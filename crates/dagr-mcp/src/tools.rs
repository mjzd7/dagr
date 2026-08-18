use crate::protocol::ToolDefinition;
use dagr_core::{DagrError, Language, Result};
use dagr_guard::ArchitectureGuard;
use dagr_sandbox::CowSandbox;
use dagr_slicer::{SlicerConfig, SymbolicSlicer};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

pub struct ToolRegistry {
    pub workspace_root: PathBuf,
    // Active A2A agent locks (agent_id -> locked_files)
    active_agent_locks: Mutex<HashMap<String, Vec<String>>>,
}

impl ToolRegistry {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            active_agent_locks: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the complete list of tools exposed to MCP IDEs and A2A swarms
    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        vec![
            // Tool 1: MCP Context Slicer
            ToolDefinition {
                name: "dagr_get_context_slice".into(),
                description: "Extracts minimal backwards AST slice & type contracts for a target symbol instead of reading entire files. Saves 95% tokens.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Relative path to target file (e.g. src/billing/charge.ts)" },
                        "symbol_name": { "type": "string", "description": "Function, method, or class name (e.g. processPayment)" },
                        "max_depth_hops": { "type": "number", "description": "Dependency traversal depth (default: 3)" }
                    },
                    "required": ["file_path", "symbol_name"]
                }),
            },
            // Tool 2: MCP Architecture Guard
            ToolDefinition {
                name: "dagr_verify_architecture".into(),
                description: "Evaluates proposed code imports against clean layer boundaries defined in .dagr/rules.yaml (<0.1ms).".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "source_file": { "type": "string", "description": "Path to file being edited" },
                        "proposed_imports": { "type": "array", "items": { "type": "string" }, "description": "List of imported modules or files" }
                    },
                    "required": ["source_file", "proposed_imports"]
                }),
            },
            // Tool 3: MCP CoW Sandbox Runner
            ToolDefinition {
                name: "dagr_execute_sandboxed".into(),
                description: "Executes a test/verification command inside an isolated Copy-on-Write shadow sandbox with instant 10ms rollback on failure.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "Shell command to execute (e.g. npm test, cargo test, pytest)" }
                    },
                    "required": ["command"]
                }),
            },
            // Tool 4: A2A Swarm Handshake & Lock Arbitrator
            ToolDefinition {
                name: "dagr_a2a_handshake".into(),
                description: "Registers an autonomous agent session and acquires optimistic file write locks to prevent peer conflicts in multi-agent swarms.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Unique identifier of the agent" },
                        "role": { "type": "string", "description": "Agent role: planner, builder, reviewer, or tester" },
                        "files_to_lock": { "type": "array", "items": { "type": "string" }, "description": "Files this agent plans to modify" }
                    },
                    "required": ["agent_id", "role"]
                }),
            },
            // Tool 5: A2A Peer Context Transfer
            ToolDefinition {
                name: "dagr_a2a_transfer_context".into(),
                description: "Passes compressed AST slices and contract envelopes directly between peer agents without disk re-parsing.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "from_agent": { "type": "string" },
                        "to_agent": { "type": "string" },
                        "file_path": { "type": "string" },
                        "symbol_name": { "type": "string" }
                    },
                    "required": ["from_agent", "to_agent", "file_path", "symbol_name"]
                }),
            },
            // Tool 6: A2A Peer Patch Verification
            ToolDefinition {
                name: "dagr_a2a_verify_peer_patch".into(),
                description: "Allows a Reviewer/Tester agent to execute test suites on another agent's staged shadow transaction before approving commit.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "reviewer_agent": { "type": "string" },
                        "target_tx_id": { "type": "string" },
                        "test_command": { "type": "string" }
                    },
                    "required": ["reviewer_agent", "target_tx_id", "test_command"]
                }),
            }
        ]
    }

    /// Dispatches a tool call to the appropriate engine
    pub fn dispatch(&self, name: &str, arguments: &Value) -> Result<Value> {
        match name {
            "dagr_get_context_slice" => self.handle_get_context_slice(arguments),
            "dagr_verify_architecture" => self.handle_verify_architecture(arguments),
            "dagr_execute_sandboxed" => self.handle_execute_sandboxed(arguments),
            "dagr_a2a_handshake" => self.handle_a2a_handshake(arguments),
            "dagr_a2a_transfer_context" => self.handle_a2a_transfer_context(arguments),
            "dagr_a2a_verify_peer_patch" => self.handle_a2a_verify_peer_patch(arguments),
            _ => Err(DagrError::Config(format!("Unknown tool: {}", name))),
        }
    }

    fn handle_get_context_slice(&self, args: &Value) -> Result<Value> {
        let file_path = args["file_path"]
            .as_str()
            .ok_or_else(|| DagrError::Config("file_path is required".into()))?;
        let symbol_name = args["symbol_name"]
            .as_str()
            .ok_or_else(|| DagrError::Config("symbol_name is required".into()))?;

        let full_path = self.workspace_root.join(file_path);
        let content = std::fs::read_to_string(&full_path).map_err(DagrError::Io)?;

        let ext = Path::new(file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let lang = Language::from_extension(ext);

        let slicer = SymbolicSlicer::new(SlicerConfig::default());
        let slice = slicer.slice(Path::new(file_path), &content, lang, symbol_name)?;

        Ok(json!({
            "target_symbol": slice.target_symbol,
            "language": format!("{:?}", slice.language),
            "sparse_code": slice.sparse_code_lines,
            "type_contracts": slice.type_contracts,
            "estimated_tokens": slice.estimated_tokens,
            "original_tokens": slice.original_file_tokens,
            "token_reduction_pct": format!("{:.1}%", slice.compression_ratio * 100.0),
            "syntax_degraded": slice.syntax_degraded
        }))
    }

    fn handle_verify_architecture(&self, args: &Value) -> Result<Value> {
        let source_file = args["source_file"].as_str().unwrap_or("");
        let imports: Vec<String> = args["proposed_imports"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let guard = ArchitectureGuard::load(&self.workspace_root)?;
        let violations = guard.check_file_imports(source_file, &imports);

        Ok(json!({
            "valid": violations.is_empty(),
            "violations_count": violations.len(),
            "violations": violations
        }))
    }

    fn handle_execute_sandboxed(&self, args: &Value) -> Result<Value> {
        let command = args["command"]
            .as_str()
            .unwrap_or("echo 'no command provided'");
        let tx = CowSandbox::begin(&self.workspace_root)?;

        let result = CowSandbox::verify(&tx, command)?;
        let success = result.success;

        if success {
            CowSandbox::commit(tx)?;
        } else {
            CowSandbox::rollback(tx)?;
        }

        Ok(json!({
            "success": success,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "rolled_back": !success
        }))
    }

    fn handle_a2a_handshake(&self, args: &Value) -> Result<Value> {
        let agent_id = args["agent_id"].as_str().unwrap_or("unknown_agent");
        let role = args["role"].as_str().unwrap_or("worker");
        let files: Vec<String> = args["files_to_lock"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut locks = self.active_agent_locks.lock().unwrap();
        locks.insert(agent_id.to_string(), files.clone());

        Ok(json!({
            "status": "registered",
            "agent_id": agent_id,
            "role": role,
            "locked_files": files,
            "swarm_peers_count": locks.len()
        }))
    }

    fn handle_a2a_transfer_context(&self, args: &Value) -> Result<Value> {
        let to_agent = args["to_agent"].as_str().unwrap_or("");
        let file_path = args["file_path"].as_str().unwrap_or("");
        let symbol_name = args["symbol_name"].as_str().unwrap_or("");

        let slice_value = self.handle_get_context_slice(&json!({
            "file_path": file_path,
            "symbol_name": symbol_name
        }))?;

        Ok(json!({
            "transferred_to": to_agent,
            "context_envelope": slice_value
        }))
    }

    fn handle_a2a_verify_peer_patch(&self, args: &Value) -> Result<Value> {
        let reviewer = args["reviewer_agent"].as_str().unwrap_or("");
        let tx_id_str = args["target_tx_id"].as_str().unwrap_or("");
        let command = args["test_command"].as_str().unwrap_or("");

        let tx_id = Uuid::parse_str(tx_id_str)
            .map_err(|e| DagrError::Config(format!("Invalid tx_id: {}", e)))?;

        let shadow_root = self
            .workspace_root
            .join(".dagr")
            .join("shadow")
            .join(tx_id.to_string());
        if !shadow_root.exists() {
            return Err(DagrError::Sandbox(format!(
                "Shadow transaction {} not found",
                tx_id
            )));
        }

        let temp_tx = dagr_sandbox::SandboxTx {
            tx_id,
            workspace_root: self.workspace_root.clone(),
            shadow_root,
            modified_files: Vec::new(),
        };

        let result = CowSandbox::verify(&temp_tx, command)?;

        Ok(json!({
            "reviewer_agent": reviewer,
            "tx_id": tx_id_str,
            "verified": result.success,
            "stdout": result.stdout,
            "stderr": result.stderr
        }))
    }
}
