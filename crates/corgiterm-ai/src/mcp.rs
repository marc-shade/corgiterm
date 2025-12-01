//! Model Context Protocol (MCP) support
//!
//! MCP enables AI agents like Claude Code, Codex, and Gemini
//! to interact with the terminal in a structured way.
//!
//! # Architecture
//!
//! The MCP server provides JSON-RPC 2.0 interface for tools execution:
//! - Tool discovery via `tools/list`
//! - Tool execution via `tools/call`
//! - Resource listing via `resources/list`
//!
//! # Tool Integration
//!
//! Tools are currently implemented with placeholder logic. To integrate with
//! the actual terminal, implement the `TerminalBackend` trait and pass it to
//! `McpServer::with_backend()`.
//!
//! Example:
//! ```ignore
//! let backend = MyTerminalBackend::new();
//! let server = McpServer::with_backend(backend);
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Backend interface for terminal operations
///
/// Implement this trait to connect MCP tools to your terminal implementation.
#[async_trait]
pub trait TerminalBackend: Send + Sync {
    /// Execute a command in the terminal
    async fn execute_command(
        &self,
        command: &str,
        cwd: Option<&str>,
        timeout_ms: Option<u64>,
    ) -> Result<String, String>;

    /// Get recent terminal output
    async fn get_output(&self, lines: u64, session_id: Option<&str>) -> Result<String, String>;

    /// List all terminal sessions
    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, String>;

    /// Create a new terminal session
    async fn create_session(
        &self,
        name: Option<&str>,
        cwd: Option<&str>,
        project_id: Option<&str>,
    ) -> Result<SessionInfo, String>;

    /// Switch to a different session
    async fn switch_session(&self, session_id: &str) -> Result<(), String>;

    /// Search command or output history
    async fn search_history(
        &self,
        query: &str,
        search_type: &str,
    ) -> Result<Vec<HistoryEntry>, String>;
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub cwd: String,
    pub project_id: Option<String>,
    pub is_active: bool,
}

/// History entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: i64,
    pub content: String,
    pub entry_type: String, // "command" or "output"
}

/// MCP server for terminal integration
pub struct McpServer {
    /// Available tools
    tools: Vec<McpTool>,
    /// Resources
    resources: Vec<McpResource>,
    /// Optional terminal backend
    backend: Option<Arc<dyn TerminalBackend>>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: Self::default_tools(),
            resources: Vec::new(),
            backend: None,
        }
    }

    /// Create a new MCP server with a terminal backend
    pub fn with_backend(backend: Arc<dyn TerminalBackend>) -> Self {
        Self {
            tools: Self::default_tools(),
            resources: Vec::new(),
            backend: Some(backend),
        }
    }

    fn default_tools() -> Vec<McpTool> {
        vec![
            McpTool {
                name: "execute_command".to_string(),
                description: "Execute a shell command in the terminal".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to execute"
                        },
                        "cwd": {
                            "type": "string",
                            "description": "Working directory (optional)"
                        },
                        "timeout_ms": {
                            "type": "number",
                            "description": "Timeout in milliseconds (optional)"
                        }
                    },
                    "required": ["command"]
                }),
            },
            McpTool {
                name: "get_terminal_output".to_string(),
                description: "Get recent terminal output".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "lines": {
                            "type": "number",
                            "description": "Number of lines to retrieve"
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Specific session ID (optional)"
                        }
                    }
                }),
            },
            McpTool {
                name: "list_sessions".to_string(),
                description: "List all terminal sessions".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            McpTool {
                name: "create_session".to_string(),
                description: "Create a new terminal session".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Session name"
                        },
                        "cwd": {
                            "type": "string",
                            "description": "Working directory"
                        },
                        "project_id": {
                            "type": "string",
                            "description": "Project to add session to (optional)"
                        }
                    }
                }),
            },
            McpTool {
                name: "switch_session".to_string(),
                description: "Switch to a different terminal session".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session ID to switch to"
                        }
                    },
                    "required": ["session_id"]
                }),
            },
            McpTool {
                name: "search_history".to_string(),
                description: "Search command or output history".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "type": {
                            "type": "string",
                            "enum": ["commands", "output", "all"],
                            "description": "What to search"
                        }
                    },
                    "required": ["query"]
                }),
            },
        ]
    }

    /// Handle an MCP request
    pub async fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "tools/list" => match serde_json::to_value(&self.tools) {
                Ok(value) => McpResponse {
                    id: request.id,
                    result: Some(value),
                    error: None,
                },
                Err(e) => McpResponse {
                    id: request.id,
                    result: None,
                    error: Some(McpError {
                        code: -32603,
                        message: format!("Failed to serialize tools: {}", e),
                    }),
                },
            },
            "tools/call" => {
                // Handle tool execution
                self.execute_tool(request.id, request.params).await
            }
            "resources/list" => match serde_json::to_value(&self.resources) {
                Ok(value) => McpResponse {
                    id: request.id,
                    result: Some(value),
                    error: None,
                },
                Err(e) => McpResponse {
                    id: request.id,
                    result: None,
                    error: Some(McpError {
                        code: -32603,
                        message: format!("Failed to serialize resources: {}", e),
                    }),
                },
            },
            _ => McpResponse {
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: "Method not found".to_string(),
                }),
            },
        }
    }

    async fn execute_tool(&self, id: String, params: Option<serde_json::Value>) -> McpResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return McpResponse {
                    id,
                    result: None,
                    error: Some(McpError {
                        code: -32602,
                        message: "Missing parameters".to_string(),
                    }),
                }
            }
        };

        let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let tool_args = params.get("arguments").cloned();

        // Validate tool exists
        if !self.tools.iter().any(|t| t.name == tool_name) {
            return McpResponse {
                id,
                result: None,
                error: Some(McpError {
                    code: -32602,
                    message: format!("Unknown tool: {}", tool_name),
                }),
            };
        }

        // Execute the tool
        match self.execute_tool_impl(tool_name, tool_args).await {
            Ok(result) => McpResponse {
                id,
                result: Some(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": result
                    }]
                })),
                error: None,
            },
            Err(err) => McpResponse {
                id,
                result: None,
                error: Some(McpError {
                    code: -32603,
                    message: err,
                }),
            },
        }
    }

    /// Execute a specific tool implementation
    async fn execute_tool_impl(
        &self,
        tool_name: &str,
        args: Option<serde_json::Value>,
    ) -> Result<String, String> {
        match tool_name {
            "execute_command" => self.tool_execute_command(args).await,
            "get_terminal_output" => self.tool_get_terminal_output(args).await,
            "list_sessions" => self.tool_list_sessions(args).await,
            "create_session" => self.tool_create_session(args).await,
            "switch_session" => self.tool_switch_session(args).await,
            "search_history" => self.tool_search_history(args).await,
            _ => Err(format!("Tool not implemented: {}", tool_name)),
        }
    }

    /// Execute a shell command
    async fn tool_execute_command(
        &self,
        args: Option<serde_json::Value>,
    ) -> Result<String, String> {
        let args = args.ok_or("Missing arguments for execute_command")?;

        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or("Missing required parameter: command")?;

        let cwd = args.get("cwd").and_then(|v| v.as_str());
        let timeout_ms = args.get("timeout_ms").and_then(|v| v.as_u64());

        // Use backend if available, otherwise return placeholder
        if let Some(backend) = &self.backend {
            backend.execute_command(command, cwd, timeout_ms).await
        } else {
            Ok(format!(
                "Command execution requested:\nCommand: {}\nCWD: {}\nTimeout: {}ms\n\nNote: No backend configured. Use McpServer::with_backend() to connect terminal.",
                command,
                cwd.unwrap_or("<current>"),
                timeout_ms.unwrap_or(30000)
            ))
        }
    }

    /// Get recent terminal output
    async fn tool_get_terminal_output(
        &self,
        args: Option<serde_json::Value>,
    ) -> Result<String, String> {
        let args = args.unwrap_or_else(|| serde_json::json!({}));

        let lines = args.get("lines").and_then(|v| v.as_u64()).unwrap_or(100);

        let session_id = args.get("session_id").and_then(|v| v.as_str());

        if let Some(backend) = &self.backend {
            backend.get_output(lines, session_id).await
        } else {
            Ok(format!(
                "Terminal output retrieval:\nLines: {}\nSession: {}\n\nNote: No backend configured.",
                lines,
                session_id.unwrap_or("<current>")
            ))
        }
    }

    /// List all terminal sessions
    async fn tool_list_sessions(&self, _args: Option<serde_json::Value>) -> Result<String, String> {
        if let Some(backend) = &self.backend {
            let sessions = backend.list_sessions().await?;
            let json = serde_json::to_string_pretty(&sessions)
                .map_err(|e| format!("Failed to serialize sessions: {}", e))?;
            Ok(json)
        } else {
            Ok(
                "Sessions:\n1. Default session (current)\n\nNote: No backend configured."
                    .to_string(),
            )
        }
    }

    /// Create a new terminal session
    async fn tool_create_session(&self, args: Option<serde_json::Value>) -> Result<String, String> {
        let args = args.unwrap_or_else(|| serde_json::json!({}));

        let name = args.get("name").and_then(|v| v.as_str());
        let cwd = args.get("cwd").and_then(|v| v.as_str());
        let project_id = args.get("project_id").and_then(|v| v.as_str());

        if let Some(backend) = &self.backend {
            let session = backend.create_session(name, cwd, project_id).await?;
            let json = serde_json::to_string_pretty(&session)
                .map_err(|e| format!("Failed to serialize session: {}", e))?;
            Ok(format!("Created session:\n{}", json))
        } else {
            Ok(format!(
                "Session creation requested:\nName: {}\nCWD: {}\nProject: {}\n\nNote: No backend configured.",
                name.unwrap_or("<auto>"),
                cwd.unwrap_or("<current>"),
                project_id.unwrap_or("<none>")
            ))
        }
    }

    /// Switch to a different session
    async fn tool_switch_session(&self, args: Option<serde_json::Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments for switch_session")?;

        let session_id = args
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing required parameter: session_id")?;

        if let Some(backend) = &self.backend {
            backend.switch_session(session_id).await?;
            Ok(format!("Switched to session: {}", session_id))
        } else {
            Ok(format!(
                "Session switch requested:\nTarget: {}\n\nNote: No backend configured.",
                session_id
            ))
        }
    }

    /// Search command or output history
    async fn tool_search_history(&self, args: Option<serde_json::Value>) -> Result<String, String> {
        let args = args.ok_or("Missing arguments for search_history")?;

        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or("Missing required parameter: query")?;

        let search_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("all");

        if let Some(backend) = &self.backend {
            let results = backend.search_history(query, search_type).await?;
            let json = serde_json::to_string_pretty(&results)
                .map_err(|e| format!("Failed to serialize results: {}", e))?;
            Ok(format!(
                "Search results ({} matches):\n{}",
                results.len(),
                json
            ))
        } else {
            Ok(format!(
                "History search:\nQuery: {}\nType: {}\n\nNote: No backend configured.",
                query, search_type
            ))
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// An MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// An MCP resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// MCP JSON-RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// MCP JSON-RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}

/// MCP error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_creation() {
        let server = McpServer::new();
        assert!(!server.tools.is_empty());
    }

    #[test]
    fn test_default_tools() {
        let tools = McpServer::default_tools();
        assert!(tools.iter().any(|t| t.name == "execute_command"));
        assert!(tools.iter().any(|t| t.name == "get_terminal_output"));
        assert!(tools.iter().any(|t| t.name == "list_sessions"));
        assert!(tools.iter().any(|t| t.name == "create_session"));
        assert!(tools.iter().any(|t| t.name == "switch_session"));
        assert!(tools.iter().any(|t| t.name == "search_history"));
    }

    #[tokio::test]
    async fn test_list_tools() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: "test-1".to_string(),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_execute_command_tool() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: "test-2".to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "execute_command",
                "arguments": {
                    "command": "echo hello",
                    "timeout_ms": 5000
                }
            })),
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_missing_required_param() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: "test-3".to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "execute_command",
                "arguments": {}
            })),
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32603);
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: "test-4".to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "nonexistent_tool",
                "arguments": {}
            })),
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32602);
    }

    #[tokio::test]
    async fn test_get_terminal_output_tool() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: "test-5".to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "get_terminal_output",
                "arguments": {
                    "lines": 50
                }
            })),
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_list_sessions_tool() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: "test-6".to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "list_sessions",
                "arguments": {}
            })),
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_search_history_tool() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: "test-7".to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "search_history",
                "arguments": {
                    "query": "git status",
                    "type": "commands"
                }
            })),
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: "test-8".to_string(),
            method: "unknown/method".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601);
    }
}
