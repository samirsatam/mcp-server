# MCP Server

A Model Context Protocol (MCP) server implementation in Rust that allows AI models to interact with external tools and data sources.

## What is MCP?

The Model Context Protocol (MCP) is a protocol that enables AI models to interact with external tools, data sources, and services. It follows the JSON-RPC 2.0 specification and communicates over stdin/stdout, making it suitable for integration with various AI platforms.

## How MCP Works with AI Agents

MCP serves as a bridge between AI agents and external capabilities, enabling them to:

### 1. **Extend AI Capabilities**
AI agents can use MCP servers to access:
- **File system operations** (read, write, search files)
- **Database queries** (SQL, NoSQL databases)
- **API integrations** (web services, APIs)
- **System commands** (shell commands, process management)
- **Custom tools** (domain-specific functionality)

### 2. **Integration Patterns**

#### **Direct Integration**
```python
# Example: Python AI agent using MCP server
import subprocess
import json

class MCPClient:
    def __init__(self, server_path):
        self.process = subprocess.Popen(
            [server_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
    
    def send_request(self, method, params=None, request_id=1):
        request = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params
        }
        
        # Send request
        self.process.stdin.write((json.dumps(request) + "\n").encode())
        self.process.stdin.flush()
        
        # Read response
        response_line = self.process.stdout.readline().decode().strip()
        return json.loads(response_line)
    
    def initialize(self):
        return self.send_request("initialize", {
            "clientInfo": {"name": "ai-agent", "version": "1.0.0"}
        })
    
    def list_tools(self):
        return self.send_request("tools/list")
    
    def call_tool(self, tool_name, arguments):
        return self.send_request("tools/call", {
            "name": tool_name,
            "arguments": arguments
        })

# Usage example
mcp_client = MCPClient("./target/release/mcp-server")
mcp_client.initialize()
tools = mcp_client.list_tools()
result = mcp_client.call_tool("echo", {"text": "Hello from AI agent!"})
```

#### **Framework Integration**
Many AI frameworks support MCP out of the box:

**LangChain Integration:**
```python
from langchain.agents import AgentExecutor, create_openai_tools_agent
from langchain_openai import ChatOpenAI
from langchain_core.tools import tool

# Define MCP tools as LangChain tools
@tool
def mcp_echo(text: str) -> str:
    """Echo back the input text using MCP server"""
    # Implementation would connect to MCP server
    return f"Echo: {text}"

# Create agent with MCP tools
llm = ChatOpenAI(model="gpt-4")
tools = [mcp_echo]
agent = create_openai_tools_agent(llm, tools)
agent_executor = AgentExecutor(agent=agent, tools=tools)
```

### 3. **Common Use Cases**

#### **File Operations**
```json
// AI agent requests file search
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "file_search",
    "arguments": {
      "query": "*.py",
      "directory": "/home/user/projects"
    }
  }
}
```

#### **Database Queries**
```json
// AI agent requests database query
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "sql_query",
    "arguments": {
      "query": "SELECT * FROM users WHERE age > 25",
      "database": "userdb"
    }
  }
}
```

#### **Web API Calls**
```json
// AI agent requests weather data
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "weather_api",
    "arguments": {
      "city": "New York",
      "units": "metric"
    }
  }
}
```

### 4. **AI Agent Workflow with MCP**

1. **Initialization**: AI agent starts MCP server and establishes connection
2. **Discovery**: Agent queries available tools using `tools/list`
3. **Planning**: Agent determines which tools are needed for the task
4. **Execution**: Agent calls tools sequentially or in parallel
5. **Integration**: Agent incorporates tool results into its reasoning
6. **Response**: Agent provides final response using gathered information

### 5. **Benefits for AI Agents**

- **Modularity**: Tools can be developed and deployed independently
- **Security**: Controlled access to system resources
- **Scalability**: Multiple specialized MCP servers for different domains
- **Standardization**: Consistent interface across different tools
- **Extensibility**: Easy to add new capabilities without modifying the AI agent

### 6. **Best Practices**

#### **Error Handling**
```python
def safe_mcp_call(client, tool_name, arguments):
    try:
        response = client.call_tool(tool_name, arguments)
        if "error" in response:
            return f"Tool error: {response['error']['message']}"
        return response["result"]["content"][0]["text"]
    except Exception as e:
        return f"Communication error: {str(e)}"
```

#### **Tool Validation**
```python
def validate_tool_schema(tool_schema, arguments):
    # Validate arguments against tool schema before calling
    required_fields = tool_schema.get("required", [])
    for field in required_fields:
        if field not in arguments:
            raise ValueError(f"Missing required field: {field}")
```

#### **Connection Management**
```python
class MCPConnectionManager:
    def __init__(self):
        self.connections = {}
    
    def get_connection(self, server_name):
        if server_name not in self.connections:
            self.connections[server_name] = self.create_connection(server_name)
        return self.connections[server_name]
    
    def cleanup(self):
        for conn in self.connections.values():
            conn.process.terminate()
```

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