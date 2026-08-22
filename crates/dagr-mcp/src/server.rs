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

                if !self.registry.has_tool(tool_name) {
                    return Some(JsonRpcResponse::error(
                        id,
                        -32602,
                        format!("Invalid params: unknown tool '{tool_name}'"),
                    ));
                }

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

    fn call_tool(server: &McpServer, id: i64, name: &str, args: Value) -> Value {
        let resp = server.handle_request(JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(id)),
            method: "tools/call".into(),
            params: Some(json!({ "name": name, "arguments": args })),
        });
        serde_json::to_value(resp.unwrap()).unwrap()
    }

    #[test]
    fn unknown_tool_returns_invalid_params_json_rpc_error() {
        let server = McpServer::new(PathBuf::from("."));
        let v = call_tool(&server, 10, "dagr_nonexistent", json!({}));
        assert_eq!(v["error"]["code"], -32602);
        assert!(
            v["error"]["message"]
                .as_str()
                .unwrap_or_default()
                .contains("unknown tool"),
            "was: {v}"
        );
    }

    #[test]
    fn verify_architecture_missing_imports_is_error_not_silent_valid() {
        let server = McpServer::new(PathBuf::from("."));
        let v = call_tool(
            &server,
            11,
            "dagr_verify_architecture",
            json!({ "source_file": "src/x.ts" }),
        );
        assert_eq!(v["result"]["isError"], json!(true));
        let text = v["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        assert!(
            text.contains("Missing required argument 'proposed_imports'"),
            "was: {text}"
        );
    }

    #[test]
    fn verify_architecture_wrong_type_is_error() {
        let server = McpServer::new(PathBuf::from("."));
        let v = call_tool(
            &server,
            12,
            "dagr_verify_architecture",
            json!({ "source_file": "src/x.ts", "proposed_imports": "not-an-array" }),
        );
        assert_eq!(v["result"]["isError"], json!(true));
        let text = v["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        assert!(text.contains("must be an array of strings"), "was: {text}");
    }

    #[test]
    fn execute_sandboxed_without_command_fails_without_running_echo() {
        let server = McpServer::new(PathBuf::from("."));
        let v = call_tool(&server, 13, "dagr_execute_sandboxed", json!({}));
        assert_eq!(v["result"]["isError"], json!(true));
        let text = v["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        assert!(
            text.contains("Missing required argument 'command'"),
            "was: {text}"
        );
        assert!(
            v["result"]["success"].is_null(),
            "must not report sandbox success: {v}"
        );
    }

    #[test]
    fn handshake_requires_agent_id_instead_of_unknown_agent_default() {
        let server = McpServer::new(PathBuf::from("."));
        let v = call_tool(&server, 14, "dagr_a2a_handshake", json!({}));
        assert_eq!(v["result"]["isError"], json!(true));
        let text = v["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        assert!(
            text.contains("Missing required argument 'agent_id'"),
            "was: {text}"
        );
    }

    #[test]
    fn verify_architecture_schema_shaped_call_has_no_rpc_error() {
        let server = McpServer::new(PathBuf::from("."));
        let v = call_tool(
            &server,
            15,
            "dagr_verify_architecture",
            json!({
                "source_file": "packages/core/src/content-filter/text-filter.ts",
                "proposed_imports": ["node:fs"]
            }),
        );
        assert!(
            v.get("error").is_none() || v["error"].is_null(),
            "rpc error: {v}"
        );
        if v["result"]["isError"] != json!(true) {
            let text = v["result"]["content"][0]["text"]
                .as_str()
                .unwrap_or_default();
            let inner: Value = serde_json::from_str(text).unwrap();
            assert_eq!(
                inner["valid"],
                json!(true),
                "clean imports must be valid: {inner}"
            );
            assert_eq!(inner["violations_count"], json!(0));
        }
    }

    #[test]
    fn verify_architecture_reports_preset_provenance_when_rules_missing() {
        let temp = tempfile::tempdir().unwrap();
        let server = McpServer::new(temp.path().to_path_buf());
        let v = call_tool(
            &server,
            16,
            "dagr_verify_architecture",
            json!({ "source_file": "src/x.ts", "proposed_imports": ["node:fs"] }),
        );
        let text = v["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        let inner: Value = serde_json::from_str(text).unwrap();
        assert_eq!(inner["rules_source"], json!("preset"));
        assert!(inner["active_rules"].as_u64().unwrap_or(0) > 0);
        assert_eq!(inner["workspace"], temp.path().display().to_string());
    }

    #[test]
    fn verify_architecture_reports_file_provenance_and_catches_planted_violation() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join(".dagr");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("rules.yaml"),
            "version: \"1.0\"\nboundaries:\n  - name: \"UI-to-DB\"\n    from: \"src/ui/**\"\n    cannot_import:\n      - \"src/db/**\"\n",
        )
        .unwrap();
        let server = McpServer::new(temp.path().to_path_buf());
        let v = call_tool(
            &server,
            17,
            "dagr_verify_architecture",
            json!({ "source_file": "src/ui/A.ts", "proposed_imports": ["src/db/client"] }),
        );
        let text = v["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        let inner: Value = serde_json::from_str(text).unwrap();
        assert_eq!(inner["rules_source"], json!("file"));
        assert_eq!(inner["active_rules"], json!(1));
        assert_eq!(
            inner["valid"],
            json!(false),
            "planted violation must be caught"
        );
    }

    /// EC-P7: tools/list is asserted against an exact allowlist so additions
    /// and renames are deliberate, reviewed events.
    #[test]
    fn tools_list_matches_exact_allowlist() {
        let server = McpServer::new(PathBuf::from("."));
        let resp = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: Some(json!(20)),
                method: "tools/list".into(),
                params: None,
            })
            .unwrap();
        let v = serde_json::to_value(resp).unwrap();
        let mut names: Vec<&str> = v["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        names.sort_unstable();
        assert_eq!(
            names,
            vec![
                "dagr_a2a_handshake",
                "dagr_a2a_transfer_context",
                "dagr_a2a_verify_peer_patch",
                "dagr_execute_sandboxed",
                "dagr_get_context_slice",
                "dagr_get_lifetime_stats",
                "dagr_verify_architecture",
            ]
        );
    }
}
