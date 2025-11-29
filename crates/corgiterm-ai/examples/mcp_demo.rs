//! MCP (Model Context Protocol) Demo
//!
//! This example demonstrates how to use the MCP server with AI providers.
//!
//! Run with:
//! ```bash
//! cargo run --example mcp_demo
//! ```

use corgiterm_ai::mcp::{HistoryEntry, McpRequest, McpServer, SessionInfo, TerminalBackend};
use async_trait::async_trait;
use std::sync::Arc;

/// Example terminal backend implementation
struct DemoBackend {
    sessions: Vec<SessionInfo>,
}

impl DemoBackend {
    fn new() -> Self {
        Self {
            sessions: vec![SessionInfo {
                id: "session-1".to_string(),
                name: "Default".to_string(),
                cwd: "/home/user".to_string(),
                project_id: None,
                is_active: true,
            }],
        }
    }
}

#[async_trait]
impl TerminalBackend for DemoBackend {
    async fn execute_command(
        &self,
        command: &str,
        cwd: Option<&str>,
        timeout_ms: Option<u64>,
    ) -> Result<String, String> {
        println!("Executing: {}", command);
        println!("  CWD: {}", cwd.unwrap_or("<current>"));
        println!("  Timeout: {}ms", timeout_ms.unwrap_or(30000));

        // Simulate command execution
        Ok(format!(
            "Command output for: {}\n(This is a demo - actual execution not implemented)",
            command
        ))
    }

    async fn get_output(&self, lines: u64, session_id: Option<&str>) -> Result<String, String> {
        Ok(format!(
            "Last {} lines from session: {}\n\n$ echo 'Hello from CorgiTerm'\nHello from CorgiTerm\n$ ls\nfile1.txt  file2.txt",
            lines,
            session_id.unwrap_or("current")
        ))
    }

    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, String> {
        Ok(self.sessions.clone())
    }

    async fn create_session(
        &self,
        name: Option<&str>,
        cwd: Option<&str>,
        project_id: Option<&str>,
    ) -> Result<SessionInfo, String> {
        let session = SessionInfo {
            id: format!("session-{}", uuid::Uuid::new_v4()),
            name: name.unwrap_or("New Session").to_string(),
            cwd: cwd.unwrap_or("/home/user").to_string(),
            project_id: project_id.map(String::from),
            is_active: false,
        };
        Ok(session)
    }

    async fn switch_session(&self, session_id: &str) -> Result<(), String> {
        if self.sessions.iter().any(|s| s.id == session_id) {
            Ok(())
        } else {
            Err(format!("Session not found: {}", session_id))
        }
    }

    async fn search_history(
        &self,
        query: &str,
        _search_type: &str,
    ) -> Result<Vec<HistoryEntry>, String> {
        Ok(vec![
            HistoryEntry {
                timestamp: 1638360000,
                content: format!("git commit -m '{}'", query),
                entry_type: "command".to_string(),
            },
            HistoryEntry {
                timestamp: 1638360060,
                content: format!("Searching for: {}", query),
                entry_type: "output".to_string(),
            },
        ])
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CorgiTerm MCP Demo\n");
    println!("===================\n");

    // Create backend and MCP server
    let backend = Arc::new(DemoBackend::new());
    let server = McpServer::with_backend(backend);

    // Example 1: List available tools
    println!("1. Listing available tools...\n");
    let list_request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: "demo-1".to_string(),
        method: "tools/list".to_string(),
        params: None,
    };
    let response = server.handle_request(list_request).await;
    if let Some(result) = response.result {
        println!("Available tools: {}\n", serde_json::to_string_pretty(&result)?);
    }

    // Example 2: Execute a command
    println!("2. Executing a command...\n");
    let exec_request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: "demo-2".to_string(),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "execute_command",
            "arguments": {
                "command": "echo 'Hello from CorgiTerm MCP!'",
                "timeout_ms": 5000
            }
        })),
    };
    let response = server.handle_request(exec_request).await;
    if let Some(result) = response.result {
        println!("Result: {}\n", serde_json::to_string_pretty(&result)?);
    }

    // Example 3: List sessions
    println!("3. Listing sessions...\n");
    let list_sessions_request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: "demo-3".to_string(),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "list_sessions",
            "arguments": {}
        })),
    };
    let response = server.handle_request(list_sessions_request).await;
    if let Some(result) = response.result {
        println!("Sessions: {}\n", serde_json::to_string_pretty(&result)?);
    }

    // Example 4: Get terminal output
    println!("4. Getting terminal output...\n");
    let output_request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: "demo-4".to_string(),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "get_terminal_output",
            "arguments": {
                "lines": 10
            }
        })),
    };
    let response = server.handle_request(output_request).await;
    if let Some(result) = response.result {
        println!("Output: {}\n", serde_json::to_string_pretty(&result)?);
    }

    // Example 5: Search history
    println!("5. Searching history...\n");
    let search_request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: "demo-5".to_string(),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "search_history",
            "arguments": {
                "query": "git",
                "type": "commands"
            }
        })),
    };
    let response = server.handle_request(search_request).await;
    if let Some(result) = response.result {
        println!("History: {}\n", serde_json::to_string_pretty(&result)?);
    }

    // Example 6: Error handling - unknown tool
    println!("6. Testing error handling (unknown tool)...\n");
    let error_request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: "demo-6".to_string(),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "nonexistent_tool",
            "arguments": {}
        })),
    };
    let response = server.handle_request(error_request).await;
    if let Some(error) = response.error {
        println!("Error (expected): {}\n", serde_json::to_string_pretty(&error)?);
    }

    println!("Demo complete!");
    Ok(())
}
