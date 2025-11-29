# MCP Architecture Diagram

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         AI Provider Layer                        │
│                                                                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  Claude  │  │  GPT-4   │  │  Gemini  │  │  Ollama  │        │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
│       │             │              │             │               │
└───────┼─────────────┼──────────────┼─────────────┼───────────────┘
        │             │              │             │
        └─────────────┴──────────────┴─────────────┘
                      │
                      │ JSON-RPC 2.0 over HTTP/WebSocket
                      │
┌─────────────────────▼─────────────────────────────────────────────┐
│                      MCP Server Layer                              │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────┐   │
│  │                     McpServer                              │   │
│  │                                                             │   │
│  │  handle_request(McpRequest) -> McpResponse                │   │
│  │  ┌──────────────────────────────────────────────────────┐ │   │
│  │  │  Method Router                                        │ │   │
│  │  │  • tools/list    -> list_tools()                     │ │   │
│  │  │  • tools/call    -> execute_tool()                   │ │   │
│  │  │  • resources/list -> list_resources()                │ │   │
│  │  └──────────────────────────────────────────────────────┘ │   │
│  │                                                             │   │
│  │  execute_tool_impl(tool_name, args)                       │   │
│  │  ┌──────────────────────────────────────────────────────┐ │   │
│  │  │  Tool Dispatcher                                      │ │   │
│  │  │  • execute_command    -> tool_execute_command()      │ │   │
│  │  │  • get_terminal_output -> tool_get_terminal_output() │ │   │
│  │  │  • list_sessions      -> tool_list_sessions()        │ │   │
│  │  │  • create_session     -> tool_create_session()       │ │   │
│  │  │  • switch_session     -> tool_switch_session()       │ │   │
│  │  │  • search_history     -> tool_search_history()       │ │   │
│  │  └──────────────────────────────────────────────────────┘ │   │
│  └───────────────────────────────────────────────────────────┘   │
│                              │                                     │
│                              │ TerminalBackend trait              │
│                              ▼                                     │
└──────────────────────────────────────────────────────────────────┘
                               │
┌──────────────────────────────▼───────────────────────────────────┐
│                    Terminal Backend Layer                         │
│                                                                    │
│  trait TerminalBackend {                                          │
│    async fn execute_command(...) -> Result<String, String>        │
│    async fn get_output(...) -> Result<String, String>             │
│    async fn list_sessions(...) -> Result<Vec<SessionInfo>, ...>  │
│    async fn create_session(...) -> Result<SessionInfo, ...>      │
│    async fn switch_session(...) -> Result<(), String>            │
│    async fn search_history(...) -> Result<Vec<HistoryEntry>,...> │
│  }                                                                 │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │           Concrete Implementation                         │    │
│  │  (Implement this trait for your terminal)                │    │
│  └──────────────────────────────────────────────────────────┘    │
└────────────────────────────────────────────────────────────────────┘
                               │
┌──────────────────────────────▼───────────────────────────────────┐
│                     Terminal Core Layer                           │
│                                                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │   Session    │  │   PTY/VTE    │  │   History    │           │
│  │   Manager    │  │   Backend    │  │   Database   │           │
│  └──────────────┘  └──────────────┘  └──────────────┘           │
└────────────────────────────────────────────────────────────────────┘
```

## Data Flow: Tool Execution

```
1. AI Provider sends JSON-RPC request
   ┌────────────────────────────────────┐
   │ POST /mcp                          │
   │ {                                  │
   │   "jsonrpc": "2.0",               │
   │   "id": "req-123",                │
   │   "method": "tools/call",         │
   │   "params": {                     │
   │     "name": "execute_command",    │
   │     "arguments": {                │
   │       "command": "ls -la"         │
   │     }                             │
   │   }                               │
   │ }                                 │
   └────────────────────────────────────┘
                ↓
2. McpServer receives and validates
   ┌────────────────────────────────────┐
   │ • Check JSON-RPC format            │
   │ • Validate method exists           │
   │ • Parse parameters                 │
   └────────────────────────────────────┘
                ↓
3. Route to tool handler
   ┌────────────────────────────────────┐
   │ execute_tool_impl(                 │
   │   "execute_command",               │
   │   {"command": "ls -la"}            │
   │ )                                  │
   └────────────────────────────────────┘
                ↓
4. Call backend if available
   ┌────────────────────────────────────┐
   │ backend.execute_command(           │
   │   "ls -la",                        │
   │   None,      // cwd                │
   │   None       // timeout            │
   │ )                                  │
   └────────────────────────────────────┘
                ↓
5. Backend executes in terminal
   ┌────────────────────────────────────┐
   │ • Validate command                 │
   │ • Set up timeout                   │
   │ • Execute in PTY                   │
   │ • Capture output                   │
   │ • Return result                    │
   └────────────────────────────────────┘
                ↓
6. Wrap in MCP response
   ┌────────────────────────────────────┐
   │ {                                  │
   │   "jsonrpc": "2.0",               │
   │   "id": "req-123",                │
   │   "result": {                     │
   │     "content": [{                 │
   │       "type": "text",             │
   │       "text": "total 48\ndrwx..." │
   │     }]                            │
   │   }                               │
   │ }                                 │
   └────────────────────────────────────┘
                ↓
7. Return to AI provider
```

## Error Handling Flow

```
┌─────────────────────────────────────────┐
│  Error occurs at any layer              │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│  Rust Result<T, String> propagates up   │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│  MCP layer catches error                │
│  Maps to JSON-RPC error code:           │
│  • -32600: Invalid Request              │
│  • -32601: Method not found             │
│  • -32602: Invalid params               │
│  • -32603: Internal error               │
└─────────────────┬───────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────┐
│  Return error response                  │
│  {                                      │
│    "jsonrpc": "2.0",                   │
│    "id": "req-123",                    │
│    "error": {                          │
│      "code": -32602,                   │
│      "message": "Unknown tool: xyz"    │
│    }                                   │
│  }                                     │
└─────────────────────────────────────────┘
```

## Tool Schema Example

```json
{
  "name": "execute_command",
  "description": "Execute a shell command in the terminal",
  "input_schema": {
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
  }
}
```

## Deployment Scenarios

### Scenario 1: Standalone Terminal
```
┌──────────────┐
│  CorgiTerm   │
│              │
│  Built-in    │
│  MCP Server  │
│  Port: 8080  │
└──────────────┘
       ↑
       │ HTTP/WS
       │
┌──────────────┐
│ AI Provider  │
│ (Claude CLI) │
└──────────────┘
```

### Scenario 2: IDE Integration
```
┌────────────────────────────────┐
│          VS Code               │
│                                │
│  ┌──────────────────────────┐ │
│  │   CorgiTerm Extension    │ │
│  │   (MCP Client)           │ │
│  └────────────┬─────────────┘ │
└───────────────┼────────────────┘
                │
┌───────────────▼────────────────┐
│     CorgiTerm Instance         │
│     (MCP Server)               │
└────────────────────────────────┘
```

### Scenario 3: Cloud Deployment
```
┌──────────────┐     ┌──────────────┐
│  AI Service  │────▶│  Load Bal.   │
└──────────────┘     └──────┬───────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
      ┌───────▼──────┐ ┌───▼──────┐ ┌───▼──────┐
      │ CorgiTerm 1  │ │ CorgiTerm│ │ CorgiTerm│
      │ MCP Server   │ │   2      │ │   3      │
      └──────────────┘ └──────────┘ └──────────┘
```

## Type Hierarchy

```
McpServer
├── tools: Vec<McpTool>
│   ├── name: String
│   ├── description: String
│   └── input_schema: JsonValue
│
├── resources: Vec<McpResource>
│   ├── uri: String
│   ├── name: String
│   ├── description: Option<String>
│   └── mime_type: Option<String>
│
└── backend: Option<Arc<dyn TerminalBackend>>
    ├── execute_command() -> Result<String, String>
    ├── get_output() -> Result<String, String>
    ├── list_sessions() -> Result<Vec<SessionInfo>, String>
    │   └── SessionInfo
    │       ├── id: String
    │       ├── name: String
    │       ├── cwd: String
    │       ├── project_id: Option<String>
    │       └── is_active: bool
    ├── create_session() -> Result<SessionInfo, String>
    ├── switch_session() -> Result<(), String>
    └── search_history() -> Result<Vec<HistoryEntry>, String>
        └── HistoryEntry
            ├── timestamp: i64
            ├── content: String
            └── entry_type: String
```

## State Machine: Tool Execution

```
[Idle]
   │
   │ receive request
   ▼
[Validating]
   │
   ├─[invalid]──▶ [Error: Invalid Request] ──▶ return error
   │
   ├─[unknown method]──▶ [Error: Method Not Found] ──▶ return error
   │
   └─[valid]
      │
      ▼
[Routing]
   │
   ├─[tools/list]──▶ [List Tools] ──▶ return tool definitions
   │
   ├─[tools/call]──▶ [Execute Tool]
   │                    │
   │                    ├─[unknown tool]──▶ [Error: Invalid Params]
   │                    │
   │                    ├─[missing params]──▶ [Error: Invalid Params]
   │                    │
   │                    └─[valid]
   │                       │
   │                       ▼
   │                  [Call Backend]
   │                       │
   │                       ├─[no backend]──▶ return placeholder
   │                       │
   │                       └─[with backend]
   │                          │
   │                          ├─[success]──▶ return result
   │                          │
   │                          └─[error]──▶ [Error: Internal]
   │
   └─[resources/list]──▶ [List Resources] ──▶ return resources

[Return to Idle]
```

## Concurrency Model

```
┌─────────────────────────────────────────────────────────┐
│                 Async/Await Model                        │
│                                                           │
│  Multiple requests handled concurrently                  │
│                                                           │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                 │
│  │ Task 1  │  │ Task 2  │  │ Task 3  │                 │
│  │ (exec)  │  │ (list)  │  │(search) │                 │
│  └────┬────┘  └────┬────┘  └────┬────┘                 │
│       │            │            │                        │
│       └────────────┼────────────┘                        │
│                    │                                     │
│            ┌───────▼────────┐                           │
│            │  Backend        │                           │
│            │  (Arc<dyn ...>) │                           │
│            │  Thread-safe    │                           │
│            └────────────────┘                            │
│                                                           │
│  Send + Sync guarantees safe concurrent access           │
└─────────────────────────────────────────────────────────┘
```
