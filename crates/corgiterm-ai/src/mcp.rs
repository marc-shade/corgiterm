//! Model Context Protocol (MCP) support
//!
//! MCP enables AI agents like Claude Code, Codex, and Gemini
//! to interact with the terminal in a structured way.

use serde::{Deserialize, Serialize};

/// MCP server for terminal integration
pub struct McpServer {
    /// Available tools
    tools: Vec<McpTool>,
    /// Resources
    resources: Vec<McpResource>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: Self::default_tools(),
            resources: Vec::new(),
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
            "tools/list" => McpResponse {
                id: request.id,
                result: Some(serde_json::to_value(&self.tools).unwrap()),
                error: None,
            },
            "tools/call" => {
                // Handle tool execution
                self.execute_tool(request.id, request.params).await
            }
            "resources/list" => McpResponse {
                id: request.id,
                result: Some(serde_json::to_value(&self.resources).unwrap()),
                error: None,
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

        // TODO: Implement actual tool execution
        McpResponse {
            id,
            result: Some(serde_json::json!({
                "content": [{
                    "type": "text",
                    "text": format!("Tool '{}' executed", tool_name)
                }]
            })),
            error: None,
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
    }
}
