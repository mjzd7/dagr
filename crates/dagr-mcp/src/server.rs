use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use crate::tools::ToolRegistry;
use dagr_core::{DagrError, Result};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

pub struct McpServer {
    registry: ToolRegistry,
}

impl McpServer {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            registry: ToolRegistry::new(workspace_root),
        }
    }

    /// Runs the Stdio JSON-RPC 2.0 event loop with strict stdout isolation
    pub fn run_stdio(&self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line_result in stdin.lock().lines() {
            let line = line_result.map_err(DagrError::Io)?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            match serde_json::from_str::<JsonRpcRequest>(trimmed) {
                Ok(request) => {
                    let response = self.handle_request(request);
                    if let Some(resp) = response {
                        let serialized = serde_json::to_string(&resp)?;
                        writeln!(stdout, "{}", serialized).map_err(DagrError::Io)?;
                        stdout.flush().map_err(DagrError::Io)?;
                    }
                }
                Err(e) => {
                    let error_resp =
                        JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e));
                    let serialized = serde_json::to_string(&error_resp)?;
                    writeln!(stdout, "{}", serialized).map_err(DagrError::Io)?;
                    stdout.flush().map_err(DagrError::Io)?;
                }
            }
        }

        Ok(())
    }

    /// Handles an incoming JSON-RPC 2.0 request
    pub fn handle_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = req.id.clone();

        match req.method.as_str() {
            "initialize" => {
                let init_result = json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "dagr-hypervisor",
                        "version": "0.1.0"
                    },
                    "capabilities": {
                        "tools": { "listChanged": false }
                    }
                });
                Some(JsonRpcResponse::success(id, init_result))
            }

            "notifications/initialized" => {
                // Initialized notification from client, no response required
                None
            }

            "ping" => Some(JsonRpcResponse::success(id, json!({}))),

            "tools/list" => {
                let tools = self.registry.list_tools();
                Some(JsonRpcResponse::success(id, json!({ "tools": tools })))
            }

            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let tool_name = params["name"].as_str().unwrap_or("");
                let arguments = &params["arguments"];

                match self.registry.dispatch(tool_name, arguments) {
                    Ok(tool_result) => {
                        let mcp_content = json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&tool_result).unwrap_or_default()
                            }]
                        });
                        Some(JsonRpcResponse::success(id, mcp_content))
                    }
                    Err(e) => {
                        let err_content = json!({
                            "isError": true,
                            "content": [{
                                "type": "text",
                                "text": format!("Tool execution failed: {}", e)
                            }]
                        });
                        Some(JsonRpcResponse::success(id, err_content))
                    }
                }
            }

            _ => Some(JsonRpcResponse::error(
                id,
                -32601,
                format!("Method not found: {}", req.method),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_initialize_and_tools_list() {
        let server = McpServer::new(PathBuf::from("."));

        // 1. Test initialize
        let init_req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "initialize".into(),
            params: None,
        };
        let init_resp = server.handle_request(init_req).unwrap();
        assert!(init_resp.result.is_some());
        assert_eq!(
            init_resp.result.unwrap()["serverInfo"]["name"],
            "dagr-hypervisor"
        );

        // 2. Test tools/list
        let tools_req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(2)),
            method: "tools/list".into(),
            params: None,
        };
        let tools_resp = server.handle_request(tools_req).unwrap();
        assert!(tools_resp.result.is_some());
        let tools_array = tools_resp.result.unwrap()["tools"]
            .as_array()
            .unwrap()
            .clone();
        assert!(tools_array
            .iter()
            .any(|t| t["name"] == "dagr_get_context_slice"));
        assert!(tools_array
            .iter()
            .any(|t| t["name"] == "dagr_a2a_handshake"));
    }
}
