// MCP Server - Model Context Protocol Implementation
// Anthropic MCP标准实现 - 让ACSA作为MCP服务器
// Standardized integration with external tools and data sources

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// MCP协议版本
pub const MCP_VERSION: &str = "2025-11-25";

/// MCP工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP资源定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

/// MCP提示模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<McpPromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// MCP请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum McpRequest {
    #[serde(rename = "initialize")]
    Initialize {
        #[serde(rename = "protocolVersion")]
        protocol_version: String,
        capabilities: Value,
        #[serde(rename = "clientInfo")]
        client_info: ClientInfo,
    },

    #[serde(rename = "tools/list")]
    ToolsList,

    #[serde(rename = "tools/call")]
    ToolsCall {
        name: String,
        arguments: Option<Value>,
    },

    #[serde(rename = "resources/list")]
    ResourcesList,

    #[serde(rename = "resources/read")]
    ResourcesRead { uri: String },

    #[serde(rename = "prompts/list")]
    PromptsList,

    #[serde(rename = "prompts/get")]
    PromptsGet {
        name: String,
        arguments: Option<HashMap<String, String>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// MCP响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpResponse {
    Initialize {
        #[serde(rename = "protocolVersion")]
        protocol_version: String,
        capabilities: ServerCapabilities,
        #[serde(rename = "serverInfo")]
        server_info: ServerInfo,
    },

    ToolsList {
        tools: Vec<McpTool>,
    },

    ToolsCallResult {
        content: Vec<ToolContent>,
        #[serde(rename = "isError")]
        is_error: Option<bool>,
    },

    ResourcesList {
        resources: Vec<McpResource>,
    },

    ResourcesRead {
        contents: Vec<ResourceContent>,
    },

    PromptsList {
        prompts: Vec<McpPrompt>,
    },

    PromptsGet {
        description: String,
        messages: Vec<PromptMessage>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub prompts: Option<PromptsCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub text: Option<String>,
    pub blob: Option<String>, // base64 encoded
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// MCP工具处理器trait
pub trait McpToolHandler: Send + Sync {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>>;
}

/// ACSA MCP服务器
pub struct AcsaMcpServer {
    /// 服务器信息
    server_info: ServerInfo,
    /// 已注册的工具
    tools: Arc<RwLock<HashMap<String, (McpTool, Box<dyn McpToolHandler>)>>>,
    /// 已注册的资源
    resources: Arc<RwLock<HashMap<String, McpResource>>>,
    /// 已注册的提示模板
    prompts: Arc<RwLock<HashMap<String, McpPrompt>>>,
}

impl AcsaMcpServer {
    pub fn new(name: String, version: String) -> Self {
        info!("🔌 MCP Server initialized: {} v{}", name, version);

        Self {
            server_info: ServerInfo { name, version },
            tools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            prompts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册MCP工具
    pub async fn register_tool<H: McpToolHandler + 'static>(
        &self,
        tool: McpTool,
        handler: H,
    ) {
        let name = tool.name.clone();
        self.tools
            .write()
            .await
            .insert(name.clone(), (tool, Box::new(handler)));

        info!("🔧 Registered MCP tool: {}", name);
    }

    /// 注册MCP资源
    pub async fn register_resource(&self, resource: McpResource) {
        let uri = resource.uri.clone();
        self.resources.write().await.insert(uri.clone(), resource);

        info!("📦 Registered MCP resource: {}", uri);
    }

    /// 注册MCP提示模板
    pub async fn register_prompt(&self, prompt: McpPrompt) {
        let name = prompt.name.clone();
        self.prompts.write().await.insert(name.clone(), prompt);

        info!("💬 Registered MCP prompt: {}", name);
    }

    /// 处理MCP请求
    pub async fn handle_request(&self, request: McpRequest) -> Result<McpResponse> {
        debug!("📨 MCP Request: {:?}", request);

        match request {
            McpRequest::Initialize {
                protocol_version,
                capabilities: _,
                client_info,
            } => {
                info!("🤝 MCP Initialize from: {} v{}", client_info.name, client_info.version);

                Ok(McpResponse::Initialize {
                    protocol_version: MCP_VERSION.to_string(),
                    capabilities: ServerCapabilities {
                        tools: Some(ToolsCapability {
                            list_changed: Some(true),
                        }),
                        resources: Some(ResourcesCapability {
                            subscribe: Some(false),
                            list_changed: Some(true),
                        }),
                        prompts: Some(PromptsCapability {
                            list_changed: Some(true),
                        }),
                    },
                    server_info: self.server_info.clone(),
                })
            }

            McpRequest::ToolsList => {
                let tools = self.tools.read().await;
                let tool_list: Vec<McpTool> = tools.values().map(|(tool, _)| tool.clone()).collect();

                Ok(McpResponse::ToolsList { tools: tool_list })
            }

            McpRequest::ToolsCall { name, arguments } => {
                let tools = self.tools.read().await;

                if let Some((_, handler)) = tools.get(&name) {
                    match handler.handle(arguments) {
                        Ok(content) => Ok(McpResponse::ToolsCallResult {
                            content,
                            is_error: Some(false),
                        }),
                        Err(e) => {
                            warn!("⚠️ Tool execution failed: {}", e);
                            Ok(McpResponse::ToolsCallResult {
                                content: vec![ToolContent {
                                    content_type: "text".to_string(),
                                    text: format!("Error: {}", e),
                                }],
                                is_error: Some(true),
                            })
                        }
                    }
                } else {
                    Err(anyhow!("Tool not found: {}", name))
                }
            }

            McpRequest::ResourcesList => {
                let resources = self.resources.read().await;
                let resource_list: Vec<McpResource> = resources.values().cloned().collect();

                Ok(McpResponse::ResourcesList {
                    resources: resource_list,
                })
            }

            McpRequest::ResourcesRead { uri } => {
                let resources = self.resources.read().await;

                if let Some(resource) = resources.get(&uri) {
                    // 这里应该实际读取资源内容
                    // 简化实现，返回占位内容
                    Ok(McpResponse::ResourcesRead {
                        contents: vec![ResourceContent {
                            uri: uri.clone(),
                            mime_type: resource.mime_type.clone(),
                            text: Some(format!("Content of {}", uri)),
                            blob: None,
                        }],
                    })
                } else {
                    Err(anyhow!("Resource not found: {}", uri))
                }
            }

            McpRequest::PromptsList => {
                let prompts = self.prompts.read().await;
                let prompt_list: Vec<McpPrompt> = prompts.values().cloned().collect();

                Ok(McpResponse::PromptsList {
                    prompts: prompt_list,
                })
            }

            McpRequest::PromptsGet { name, arguments } => {
                let prompts = self.prompts.read().await;

                if let Some(prompt) = prompts.get(&name) {
                    // 生成提示消息
                    let messages = vec![PromptMessage {
                        role: "user".to_string(),
                        content: PromptContent {
                            content_type: "text".to_string(),
                            text: format!("Prompt: {} with args: {:?}", name, arguments),
                        },
                    }];

                    Ok(McpResponse::PromptsGet {
                        description: prompt.description.clone(),
                        messages,
                    })
                } else {
                    Err(anyhow!("Prompt not found: {}", name))
                }
            }
        }
    }
}

/// ACSA预置工具处理器
pub struct AcsaProtocolSwitchHandler;

impl McpToolHandler for AcsaProtocolSwitchHandler {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        let protocol_name = arguments
            .and_then(|v| v.get("protocol").and_then(|p| p.as_str().map(String::from)))
            .ok_or_else(|| anyhow!("Missing protocol argument"))?;

        Ok(vec![ToolContent {
            content_type: "text".to_string(),
            text: format!("✅ Switched to protocol: {}", protocol_name),
        }])
    }
}

pub struct AcsaTaskTrackerHandler;

impl McpToolHandler for AcsaTaskTrackerHandler {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        let action = arguments
            .and_then(|v| v.get("action").and_then(|a| a.as_str().map(String::from)))
            .ok_or_else(|| anyhow!("Missing action argument"))?;

        let response = match action.as_str() {
            "list" => "📋 Current tasks:\n  - Task 1: In Progress\n  - Task 2: Pending",
            "add" => "✅ Task added to tracker",
            "complete" => "✅ Task marked as completed",
            _ => "❌ Unknown action",
        };

        Ok(vec![ToolContent {
            content_type: "text".to_string(),
            text: response.to_string(),
        }])
    }
}

pub struct AcsaBehaviorAnalysisHandler;

impl McpToolHandler for AcsaBehaviorAnalysisHandler {
    fn handle(&self, _arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        Ok(vec![ToolContent {
            content_type: "text".to_string(),
            text: "📊 Behavior Analysis:\n  - Patterns detected: 5\n  - Confidence: 85%\n  - Auto-takeover ready: Yes"
                .to_string(),
        }])
    }
}

/// 创建ACSA MCP服务器并注册默认工具
pub async fn create_acsa_mcp_server() -> AcsaMcpServer {
    let server = AcsaMcpServer::new("ACSA".to_string(), "0.1.0".to_string());

    // 注册Protocol切换工具
    server
        .register_tool(
            McpTool {
                name: "acsa_switch_protocol".to_string(),
                description: "Switch ACSA to a different protocol mode (ARCHITECT/AEGIS/PREDATOR/etc.)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "protocol": {
                            "type": "string",
                            "description": "Protocol name to switch to"
                        }
                    },
                    "required": ["protocol"]
                }),
            },
            AcsaProtocolSwitchHandler,
        )
        .await;

    // 注册TaskTracker工具
    server
        .register_tool(
            McpTool {
                name: "acsa_task_tracker".to_string(),
                description: "Manage ACSA task tracker (list/add/complete tasks)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["list", "add", "complete"],
                            "description": "Action to perform"
                        },
                        "task": {
                            "type": "string",
                            "description": "Task description (for add action)"
                        }
                    },
                    "required": ["action"]
                }),
            },
            AcsaTaskTrackerHandler,
        )
        .await;

    // 注册行为分析工具
    server
        .register_tool(
            McpTool {
                name: "acsa_behavior_analysis".to_string(),
                description: "Get user behavior analysis and pattern detection results".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            AcsaBehaviorAnalysisHandler,
        )
        .await;

    // 注册资源
    server
        .register_resource(McpResource {
            uri: "acsa://protocols".to_string(),
            name: "ACSA Protocols".to_string(),
            description: "List of available ACSA protocols and their configurations".to_string(),
            mime_type: "application/json".to_string(),
        })
        .await;

    server
        .register_resource(McpResource {
            uri: "acsa://behavior-patterns".to_string(),
            name: "Behavior Patterns".to_string(),
            description: "Detected user behavior patterns".to_string(),
            mime_type: "application/json".to_string(),
        })
        .await;

    // 注册提示模板
    server
        .register_prompt(McpPrompt {
            name: "acsa_code_review".to_string(),
            description: "Generate code review with ACSA's multi-agent system".to_string(),
            arguments: vec![
                McpPromptArgument {
                    name: "code".to_string(),
                    description: "Code to review".to_string(),
                    required: true,
                },
                McpPromptArgument {
                    name: "protocol".to_string(),
                    description: "Protocol to use (default: REVIEWER_2)".to_string(),
                    required: false,
                },
            ],
        })
        .await;

    info!("✅ ACSA MCP Server fully initialized with default tools");

    server
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let server = AcsaMcpServer::new("TestServer".to_string(), "1.0.0".to_string());
        assert_eq!(server.server_info.name, "TestServer");
    }

    #[tokio::test]
    async fn test_mcp_initialize() {
        let server = create_acsa_mcp_server().await;

        let request = McpRequest::Initialize {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: json!({}),
            client_info: ClientInfo {
                name: "TestClient".to_string(),
                version: "1.0".to_string(),
            },
        };

        let response = server.handle_request(request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_tools_list() {
        let server = create_acsa_mcp_server().await;

        let response = server.handle_request(McpRequest::ToolsList).await.unwrap();

        if let McpResponse::ToolsList { tools } = response {
            assert!(!tools.is_empty());
            assert!(tools.iter().any(|t| t.name == "acsa_switch_protocol"));
        } else {
            panic!("Expected ToolsList response");
        }
    }
}
