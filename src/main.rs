use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub struct McpServer {
    tools: HashMap<String, Tool>,
}

impl McpServer {
    pub fn new() -> Self {
        let mut server = Self {
            tools: HashMap::new(),
        };
        
        server.register_example_tool();
        server
    }
    
    fn register_example_tool(&mut self) {
        let tool = Tool {
            name: "echo".to_string(),
            description: "Echo back the input text".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to echo back"
                    }
                },
                "required": ["text"]
            }),
        };
        
        self.tools.insert("echo".to_string(), tool);
    }
    
    pub async fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "tools/list" => self.handle_tools_list(request).await,
            "tools/call" => self.handle_tools_call(request).await,
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            },
        }
    }
    
    async fn handle_initialize(&self, request: McpRequest) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "mcp-server",
                    "version": "0.1.0"
                }
            })),
            error: None,
        }
    }
    
    async fn handle_tools_list(&self, request: McpRequest) -> McpResponse {
        let tools: Vec<&Tool> = self.tools.values().collect();
        
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({
                "tools": tools
            })),
            error: None,
        }
    }
    
    async fn handle_tools_call(&self, request: McpRequest) -> McpResponse {
        let params = match request.params {
            Some(params) => params,
            None => {
                return McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(McpError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: None,
                    }),
                };
            }
        };
        
        let tool_name = match params.get("name") {
            Some(Value::String(name)) => name,
            _ => {
                return McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(McpError {
                        code: -32602,
                        message: "Tool name required".to_string(),
                        data: None,
                    }),
                };
            }
        };
        
        match tool_name.as_str() {
            "echo" => {
                let text = params
                    .get("arguments")
                    .and_then(|args| args.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("No text provided");
                
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Echo: {}", text)
                        }]
                    })),
                    error: None,
                }
            }
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: "Tool not found".to_string(),
                    data: None,
                }),
            },
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = McpServer::new();
    
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();
    
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                match serde_json::from_str::<McpRequest>(line) {
                    Ok(request) => {
                        let response = server.handle_request(request).await;
                        let response_json = serde_json::to_string(&response)?;
                        stdout.write_all(response_json.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                    }
                    Err(e) => {
                        eprintln!("Failed to parse request: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read line: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_request_serialization() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: Some(json!({"clientInfo": {"name": "test", "version": "1.0"}})),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: McpRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.jsonrpc, deserialized.jsonrpc);
        assert_eq!(request.id, deserialized.id);
        assert_eq!(request.method, deserialized.method);
        assert_eq!(request.params, deserialized.params);
    }

    #[test]
    fn test_mcp_response_serialization() {
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            result: Some(json!({"success": true})),
            error: None,
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: McpResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response.jsonrpc, deserialized.jsonrpc);
        assert_eq!(response.id, deserialized.id);
        assert_eq!(response.result, deserialized.result);
        assert_eq!(response.error.is_none(), deserialized.error.is_none());
    }

    #[test]
    fn test_mcp_error_serialization() {
        let error = McpError {
            code: -32601,
            message: "Method not found".to_string(),
            data: Some(json!({"method": "unknown"})),
        };

        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: McpError = serde_json::from_str(&serialized).unwrap();

        assert_eq!(error.code, deserialized.code);
        assert_eq!(error.message, deserialized.message);
        assert_eq!(error.data, deserialized.data);
    }

    #[tokio::test]
    async fn test_server_initialization() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: Some(json!({"clientInfo": {"name": "test", "version": "1.0"}})),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(1)));
        assert!(response.error.is_none());
        
        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert_eq!(result["serverInfo"]["name"], "mcp-server");
        assert_eq!(result["serverInfo"]["version"], "0.1.0");
    }

    #[tokio::test]
    async fn test_tools_list() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(2)));
        assert!(response.error.is_none());
        
        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "echo");
        assert_eq!(tools[0]["description"], "Echo back the input text");
    }

    #[tokio::test]
    async fn test_echo_tool_execution() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(3)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "echo",
                "arguments": {
                    "text": "Hello, World!"
                }
            })),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(3)));
        assert!(response.error.is_none());
        
        let result = response.result.unwrap();
        let content = result["content"].as_array().unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[0]["text"], "Echo: Hello, World!");
    }

    #[tokio::test]
    async fn test_echo_tool_without_text() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(4)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "echo",
                "arguments": {}
            })),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(4)));
        assert!(response.error.is_none());
        
        let result = response.result.unwrap();
        let content = result["content"].as_array().unwrap();
        assert_eq!(content[0]["text"], "Echo: No text provided");
    }

    #[tokio::test]
    async fn test_unknown_method_error() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(5)),
            method: "unknown/method".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(5)));
        assert!(response.result.is_none());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
    }

    #[tokio::test]
    async fn test_unknown_tool_error() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(6)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "unknown_tool",
                "arguments": {}
            })),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(6)));
        assert!(response.result.is_none());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Tool not found");
    }

    #[tokio::test]
    async fn test_tool_call_without_params() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(7)),
            method: "tools/call".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(7)));
        assert!(response.result.is_none());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "Invalid params");
    }

    #[tokio::test]
    async fn test_tool_call_without_name() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(8)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "arguments": {"text": "test"}
            })),
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(json!(8)));
        assert!(response.result.is_none());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "Tool name required");
    }

    #[test]
    fn test_server_creation() {
        let server = McpServer::new();
        assert_eq!(server.tools.len(), 1);
        assert!(server.tools.contains_key("echo"));
    }

    #[test]
    fn test_tool_schema() {
        let server = McpServer::new();
        let echo_tool = server.tools.get("echo").unwrap();
        
        assert_eq!(echo_tool.name, "echo");
        assert_eq!(echo_tool.description, "Echo back the input text");
        
        let schema = &echo_tool.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["text"].is_object());
        assert_eq!(schema["required"].as_array().unwrap().len(), 1);
        assert_eq!(schema["required"][0], "text");
    }
}
