# MCP (Model Context Protocol) Integration Guide

## Overview

The MCP implementation in CorgiTerm provides a JSON-RPC 2.0 interface for AI agents to interact with the terminal programmatically. This enables Claude, GPT-4, Gemini, and other AI assistants to execute commands, manage sessions, and access terminal data.

## Architecture

```
┌─────────────────────┐
│   AI Provider       │
│  (Claude, GPT-4)    │
└──────────┬──────────┘
           │ JSON-RPC 2.0
           │
┌──────────▼──────────┐
│    McpServer        │
│                     │
│  - Tool Discovery   │
│  - Tool Execution   │
│  - Error Handling   │
└──────────┬──────────┘
           │ TerminalBackend trait
           │
┌──────────▼──────────┐
│  Terminal Backend   │
│                     │
│  - Session Mgmt     │
│  - Command Exec     │
│  - History Search   │
└─────────────────────┘
```

## Available Tools

### 1. execute_command
Execute a shell command in the terminal.

**Parameters:**
- `command` (required): The shell command to execute
- `cwd` (optional): Working directory for execution
- `timeout_ms` (optional): Execution timeout in milliseconds

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "tools/call",
  "params": {
    "name": "execute_command",
    "arguments": {
      "command": "ls -la",
      "cwd": "/home/user/projects",
      "timeout_ms": 5000
    }
  }
}
```

### 2. get_terminal_output
Retrieve recent terminal output.

**Parameters:**
- `lines` (optional): Number of lines to retrieve (default: 100)
- `session_id` (optional): Specific session ID to query

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "2",
  "method": "tools/call",
  "params": {
    "name": "get_terminal_output",
    "arguments": {
      "lines": 50
    }
  }
}
```

### 3. list_sessions
List all active terminal sessions.

**Parameters:** None

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "3",
  "method": "tools/call",
  "params": {
    "name": "list_sessions",
    "arguments": {}
  }
}
```

### 4. create_session
Create a new terminal session.

**Parameters:**
- `name` (optional): Session name
- `cwd` (optional): Working directory
- `project_id` (optional): Associated project ID

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "4",
  "method": "tools/call",
  "params": {
    "name": "create_session",
    "arguments": {
      "name": "dev-session",
      "cwd": "/home/user/project",
      "project_id": "my-project"
    }
  }
}
```

### 5. switch_session
Switch to a different terminal session.

**Parameters:**
- `session_id` (required): Target session ID

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "5",
  "method": "tools/call",
  "params": {
    "name": "switch_session",
    "arguments": {
      "session_id": "abc-123"
    }
  }
}
```

### 6. search_history
Search command or output history.

**Parameters:**
- `query` (required): Search query string
- `type` (optional): "commands", "output", or "all" (default: "all")

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "6",
  "method": "tools/call",
  "params": {
    "name": "search_history",
    "arguments": {
      "query": "git commit",
      "type": "commands"
    }
  }
}
```

## Implementing a Backend

To connect MCP tools to your terminal implementation, implement the `TerminalBackend` trait:

```rust
use corgiterm_ai::mcp::{TerminalBackend, SessionInfo, HistoryEntry};
use async_trait::async_trait;
use std::sync::Arc;

pub struct MyTerminalBackend {
    // Your terminal state here
}

#[async_trait]
impl TerminalBackend for MyTerminalBackend {
    async fn execute_command(
        &self,
        command: &str,
        cwd: Option<&str>,
        timeout_ms: Option<u64>,
    ) -> Result<String, String> {
        // Execute command and return output
        todo!()
    }

    async fn get_output(&self, lines: u64, session_id: Option<&str>) -> Result<String, String> {
        // Get terminal output
        todo!()
    }

    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, String> {
        // List all sessions
        todo!()
    }

    async fn create_session(
        &self,
        name: Option<&str>,
        cwd: Option<&str>,
        project_id: Option<&str>,
    ) -> Result<SessionInfo, String> {
        // Create a new session
        todo!()
    }

    async fn switch_session(&self, session_id: &str) -> Result<(), String> {
        // Switch to a session
        todo!()
    }

    async fn search_history(
        &self,
        query: &str,
        search_type: &str,
    ) -> Result<Vec<HistoryEntry>, String> {
        // Search history
        todo!()
    }
}
```

Then create the MCP server with your backend:

```rust
use corgiterm_ai::mcp::McpServer;
use std::sync::Arc;

let backend = Arc::new(MyTerminalBackend::new());
let mcp_server = McpServer::with_backend(backend);
```

## Error Handling

MCP uses standard JSON-RPC 2.0 error codes:

| Code | Meaning |
|------|---------|
| -32600 | Invalid Request |
| -32601 | Method not found |
| -32602 | Invalid params |
| -32603 | Internal error |

**Error Response Example:**
```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "error": {
    "code": -32602,
    "message": "Unknown tool: invalid_tool"
  }
}
```

## Testing

Run the MCP tests:

```bash
cargo test --package corgiterm-ai --lib mcp
```

## Integration with AI Providers

### Claude (Anthropic)
Claude has native MCP support. Configure your MCP server endpoint in Claude's configuration.

### OpenAI / GPT-4
Use function calling with the MCP tool definitions converted to OpenAI function schemas.

### Gemini
Use Gemini's function calling with adapted MCP schemas.

## Security Considerations

1. **Command Validation**: Always validate commands before execution
2. **Timeout Enforcement**: Respect timeout parameters to prevent hanging
3. **Path Sanitization**: Sanitize file paths to prevent directory traversal
4. **Output Limits**: Limit output size to prevent memory exhaustion
5. **Rate Limiting**: Consider implementing rate limits for tool calls

## Future Enhancements

- [ ] Resource support (file access, directory browsing)
- [ ] Streaming tool output for long-running commands
- [ ] Tool composition (chaining multiple tools)
- [ ] Permission system for dangerous operations
- [ ] Audit logging for all tool executions
- [ ] WebSocket transport for bidirectional communication
- [ ] Multi-session command execution
- [ ] Advanced history filtering and analysis

## References

- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [CorgiTerm AI Crate Documentation](./README.md)
