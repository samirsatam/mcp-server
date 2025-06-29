# MCP Server

A Model Context Protocol (MCP) server implementation in Rust that allows AI models to interact with external tools and data sources.

## What is MCP?

The Model Context Protocol (MCP) is a protocol that enables AI models to interact with external tools, data sources, and services. It follows the JSON-RPC 2.0 specification and communicates over stdin/stdout, making it suitable for integration with various AI platforms.

## Architecture Overview

This MCP server is built with the following key components:

### Core Data Structures

- **`McpRequest`**: JSON-RPC request with method and parameters
- **`McpResponse`**: JSON-RPC response with result or error
- **`McpError`**: Error information with code and message
- **`Tool`**: Tool definition with name, description, and input schema
- **`McpServer`**: Main server that manages tools and handles requests

### Communication Protocol

The server operates as a **line-oriented JSON-RPC** server:

1. **Input**: Reads JSON-RPC requests from stdin, one per line
2. **Processing**: Parses each request and routes it to the appropriate handler
3. **Output**: Writes JSON-RPC responses to stdout, one per line
4. **Error Handling**: Returns proper JSON-RPC error codes for invalid requests

## Request Handling

The server handles three main MCP methods:

### 1. `initialize`
- Called when a client first connects
- Returns protocol version, server capabilities, and server information
- Establishes the connection and negotiates protocol features

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "clientInfo": {
      "name": "test-client",
      "version": "1.0.0"
    }
  }
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {}
    },
    "serverInfo": {
      "name": "mcp-server",
      "version": "0.1.0"
    }
  }
}
```

### 2. `tools/list`
- Returns a list of all available tools with their schemas
- Allows clients to discover what tools are available
- Each tool includes its name, description, and input schema

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list",
  "params": null
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "echo",
        "description": "Echo back the input text",
        "input_schema": {
          "type": "object",
          "properties": {
            "text": {
              "type": "string",
              "description": "Text to echo back"
            }
          },
          "required": ["text"]
        }
      }
    ]
  }
}
```

### 3. `tools/call`
- Executes a specific tool with provided arguments
- Validates the tool name and parameters
- Returns the tool's result in a standardized format

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "echo",
    "arguments": {
      "text": "Hello, World!"
    }
  }
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Echo: Hello, World!"
      }
    ]
  }
}
```

## Tool System

The server maintains a registry of available tools. Each tool has:

- **Name**: Unique identifier for the tool
- **Description**: Human-readable description of what the tool does
- **Input Schema**: JSON Schema defining the expected parameters

### Example Tool: Echo

The server includes a simple "echo" tool that:
- Takes a `text` parameter
- Returns the text prefixed with "Echo: "
- Demonstrates the basic tool execution pattern

## Error Handling

The server implements proper JSON-RPC error handling with standard error codes:

- **-32601**: Method not found
- **-32602**: Invalid parameters
- Custom error messages for specific failures

**Example Error Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": null,
  "error": {
    "code": -32601,
    "message": "Method not found",
    "data": null
  }
}
```

## Usage

### Building the Server

```bash
cargo build --release
```

### Running the Server

```bash
cargo run
```

The server will start and wait for JSON-RPC requests on stdin. Each request should be a complete JSON object on a single line.

### Testing

Run the comprehensive test suite:

```bash
cargo test
```

The tests verify:
- JSON serialization/deserialization
- Server initialization
- Tool listing
- Tool execution
- Error conditions

## Integration with AI Models

To integrate this MCP server with an AI model:

1. **Start the server** as a subprocess
2. **Connect to stdin/stdout** for communication
3. **Send initialization request** to establish connection
4. **Discover available tools** using `tools/list`
5. **Execute tools** using `tools/call` with appropriate parameters
6. **Process responses** and integrate results into the AI model's context

## Extending the Server

To add new tools to the server:

1. **Define the tool** in the `register_example_tool()` method or create a new registration method
2. **Add a handler** in the `handle_tools_call()` method
3. **Implement the tool logic** with proper parameter validation
4. **Return results** in the standardized content format

## Dependencies

- **tokio**: Async runtime for I/O operations
- **serde**: Serialization/deserialization
- **serde_json**: JSON handling
- **uuid**: Unique identifier generation
- **anyhow**: Error handling
- **async-trait**: Async trait support

## License

This project is open source and available under the MIT License. 