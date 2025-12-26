# MCP (Model Context Protocol) 集成指南

**ACSA MCP Server - 标准化的外部工具与数据源集成**

---

## ⚠️ 安全警告

### 🔒 数据隐私与网站信任

**在使用 MCP 工具连接外部服务时，请务必注意数据安全：**

> ⚠️ **不可信的网站或服务可能会诱导 AI 代理分享敏感数据**
>
> MCP 协议允许 AI 应用访问本地资源、执行工具调用，并与外部 API 交互。虽然 MCP 设计尊重用户隐私，但**恶意或不可信的第三方服务可能会：**
>
> - 🚨 诱导 AI 代理泄露本地文件内容
> - 🚨 窃取环境变量中的 API 密钥和凭证
> - 🚨 执行未经授权的工具调用
> - 🚨 收集用户交互数据用于未披露的目的
>
> **安全建议：**
>
> 1. ✅ **仅连接可信的 MCP 服务器和工具**
>    - 优先使用官方或知名开源实现
>    - 审查第三方工具的源代码
>    - 检查工具权限和数据访问范围
>
> 2. ✅ **使用受限的 API 密钥**
>    - 为 MCP 工具创建专用的、权限受限的 API 密钥
>    - 避免使用具有完整账户权限的主密钥
>    - 定期轮换凭证
>
> 3. ✅ **隔离敏感数据**
>    - 不要将敏感文件暴露为 MCP 资源
>    - 使用环境变量管理凭证，不要硬编码
>    - 考虑使用 `.env` 文件并确保它不被版本控制
>
> 4. ✅ **审查工具行为**
>    - 定期检查 MCP 工具的网络请求
>    - 监控异常的 API 调用模式
>    - 启用审计日志记录
>
> 5. ✅ **网络隔离**
>    - 在受信任的网络环境中运行 MCP 服务器
>    - 考虑使用防火墙规则限制出站连接
>    - 对于企业环境，使用 VPN 或专用网络

**记住：MCP 工具拥有你授予的所有权限。谨慎选择，审慎授权。**

---

## 📋 目录

- [⚠️ 安全警告](#️-安全警告)
- [什么是 MCP](#什么是-mcp)
- [ACSA MCP 服务器概述](#acsa-mcp-服务器概述)
- [快速开始](#快速开始)
- [集成第三方平台](#集成第三方平台)
  - [Google 平台](#1-google-平台集成)
  - [GitHub 集成](#2-github-集成)
  - [网盘服务](#3-网盘服务集成)
  - [其他平台](#4-其他平台集成)
- [自定义 MCP 工具](#自定义-mcp-工具)
- [API 平台规则与限制](#api-平台规则与限制)
- [最佳实践](#最佳实践)
- [故障排查](#故障排查)

---

## 什么是 MCP

**Model Context Protocol (MCP)** 是由 Anthropic 开发并捐赠给 [Agentic AI Foundation](https://www.anthropic.com/news/donating-the-model-context-protocol-and-establishing-of-the-agentic-ai-foundation) (Linux Foundation) 的开放协议。

### 核心特性

- 🔌 **标准化集成**: 统一的协议连接 LLM 应用与外部工具
- 🚀 **异步操作**: 支持长时间运行的任务追踪
- 🔒 **类型安全**: JSON-RPC 2.0 基础，TypeScript/Rust 强类型支持
- 📦 **可扩展**: 轻松添加自定义工具、资源和提示模板

### MCP vs LSP

| 特性 | MCP (Model Context Protocol) | LSP (Language Server Protocol) |
|------|------------------------------|--------------------------------|
| **用途** | AI 应用与数据源/工具集成 | 编辑器与语言服务器集成 |
| **传输** | stdio, HTTP (SSE) | stdio, socket |
| **协议** | JSON-RPC 2.0 | JSON-RPC 2.0 |
| **支持** | Python, TypeScript, Rust, C#, Java | 几乎所有编程语言 |

**ACSA 同时支持 MCP 和 LSP！**

---

## ACSA MCP 服务器概述

### 当前版本

- **MCP 协议版本**: `2025-11-25` (最新规范)
- **实现文件**: `o_sovereign_rust/src/core/mcp_server.rs`
- **相关文档**: [MCP 官方规范](https://modelcontextprotocol.io/specification/2025-11-25)

### 支持的能力

ACSA MCP 服务器实现了完整的 MCP 规范：

#### 1. **Tools (工具)**
允许 AI 调用 ACSA 的功能：
- ✅ 协议切换 (Protocol Switch)
- ✅ 任务追踪 (Task Tracker)
- ✅ 自定义工具扩展

#### 2. **Resources (资源)**
提供可读取的数据源：
- ✅ 审计日志
- ✅ 配置文件
- ✅ 系统状态
- ✅ 自定义资源

#### 3. **Prompts (提示模板)**
预定义的提示模板：
- ✅ 战略分析模板
- ✅ 安全审计模板
- ✅ 自定义提示

---

## 快速开始

### 1. 启动 MCP 服务器

```rust
use acsa_core::{AcsaMcpServer, create_acsa_mcp_server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建 ACSA MCP 服务器
    let mcp_server = create_acsa_mcp_server().await;

    // 服务器已预注册 ACSA 核心工具
    println!("✅ MCP Server started!");

    Ok(())
}
```

### 2. 注册自定义工具

```rust
use acsa_core::{AcsaMcpServer, McpTool, McpToolHandler};
use serde_json::json;

// 定义工具处理器
struct MyCustomTool;

impl McpToolHandler for MyCustomTool {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        // 处理工具调用
        Ok(vec![ToolContent {
            content_type: "text".to_string(),
            text: "Tool executed successfully!".to_string(),
        }])
    }
}

// 注册工具
let tool = McpTool {
    name: "my_custom_tool".to_string(),
    description: "My awesome custom tool".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "param1": { "type": "string", "description": "First parameter" }
        },
        "required": ["param1"]
    }),
};

mcp_server.register_tool(tool, MyCustomTool).await;
```

### 3. 注册资源

```rust
use acsa_core::McpResource;

let resource = McpResource {
    uri: "acsa://config/main".to_string(),
    name: "Main Configuration".to_string(),
    description: "ACSA main configuration file".to_string(),
    mime_type: "application/toml".to_string(),
};

mcp_server.register_resource(resource).await;
```

---

## 集成第三方平台

### 1. Google 平台集成

#### Google Drive

```rust
use acsa_core::{McpTool, McpToolHandler};
use serde_json::json;

struct GoogleDriveHandler {
    api_key: String,
}

impl McpToolHandler for GoogleDriveHandler {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        let file_id = arguments
            .and_then(|v| v.get("file_id").and_then(|f| f.as_str()))
            .ok_or_else(|| anyhow!("Missing file_id"))?;

        // 使用 Google Drive API
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("https://www.googleapis.com/drive/v3/files/{}", file_id))
            .bearer_auth(&self.api_key)
            .send()?;

        let content = response.text()?;

        Ok(vec![ToolContent {
            content_type: "text".to_string(),
            text: content,
        }])
    }
}

// 注册工具
let google_drive_tool = McpTool {
    name: "google_drive_read".to_string(),
    description: "Read file from Google Drive".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "file_id": {
                "type": "string",
                "description": "Google Drive file ID"
            }
        },
        "required": ["file_id"]
    }),
};

mcp_server.register_tool(
    google_drive_tool,
    GoogleDriveHandler {
        api_key: std::env::var("GOOGLE_API_KEY")?,
    }
).await;
```

#### Google Calendar

```rust
struct GoogleCalendarHandler {
    oauth_token: String,
}

impl McpToolHandler for GoogleCalendarHandler {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        let time_min = arguments
            .and_then(|v| v.get("time_min").and_then(|t| t.as_str()))
            .unwrap_or("now");

        // 调用 Google Calendar API
        let client = reqwest::blocking::Client::new();
        let response = client
            .get("https://www.googleapis.com/calendar/v3/calendars/primary/events")
            .bearer_auth(&self.oauth_token)
            .query(&[("timeMin", time_min)])
            .send()?;

        Ok(vec![ToolContent {
            content_type: "application/json".to_string(),
            text: response.text()?,
        }])
    }
}
```

---

### 2. GitHub 集成

#### GitHub Repository 操作

```rust
struct GitHubHandler {
    access_token: String,
}

impl McpToolHandler for GitHubHandler {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        let args = arguments.ok_or_else(|| anyhow!("Missing arguments"))?;
        let action = args["action"].as_str().unwrap_or("list_repos");

        match action {
            "list_repos" => {
                let client = reqwest::blocking::Client::new();
                let response = client
                    .get("https://api.github.com/user/repos")
                    .header("Authorization", format!("token {}", self.access_token))
                    .header("User-Agent", "ACSA-MCP-Client")
                    .send()?;

                Ok(vec![ToolContent {
                    content_type: "application/json".to_string(),
                    text: response.text()?,
                }])
            }

            "create_issue" => {
                let repo = args["repo"].as_str().ok_or_else(|| anyhow!("Missing repo"))?;
                let title = args["title"].as_str().ok_or_else(|| anyhow!("Missing title"))?;
                let body = args["body"].as_str().unwrap_or("");

                let client = reqwest::blocking::Client::new();
                let response = client
                    .post(&format!("https://api.github.com/repos/{}/issues", repo))
                    .header("Authorization", format!("token {}", self.access_token))
                    .header("User-Agent", "ACSA-MCP-Client")
                    .json(&json!({
                        "title": title,
                        "body": body
                    }))
                    .send()?;

                Ok(vec![ToolContent {
                    content_type: "application/json".to_string(),
                    text: response.text()?,
                }])
            }

            _ => Err(anyhow!("Unknown action: {}", action)),
        }
    }
}

// 注册 GitHub 工具
let github_tool = McpTool {
    name: "github".to_string(),
    description: "GitHub repository operations".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "enum": ["list_repos", "create_issue", "search_code"],
                "description": "Action to perform"
            },
            "repo": {
                "type": "string",
                "description": "Repository name (owner/repo)"
            },
            "title": {
                "type": "string",
                "description": "Issue title"
            },
            "body": {
                "type": "string",
                "description": "Issue body"
            }
        },
        "required": ["action"]
    }),
};

mcp_server.register_tool(
    github_tool,
    GitHubHandler {
        access_token: std::env::var("GITHUB_TOKEN")?,
    }
).await;
```

---

### 3. 网盘服务集成

#### Dropbox

```rust
struct DropboxHandler {
    access_token: String,
}

impl McpToolHandler for DropboxHandler {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        let path = arguments
            .and_then(|v| v.get("path").and_then(|p| p.as_str()))
            .ok_or_else(|| anyhow!("Missing path"))?;

        let client = reqwest::blocking::Client::new();
        let response = client
            .post("https://api.dropboxapi.com/2/files/download")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Dropbox-API-Arg", json!({"path": path}).to_string())
            .send()?;

        Ok(vec![ToolContent {
            content_type: "text".to_string(),
            text: response.text()?,
        }])
    }
}
```

#### OneDrive / SharePoint

```rust
struct OneDriveHandler {
    access_token: String,
}

impl McpToolHandler for OneDriveHandler {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        let item_id = arguments
            .and_then(|v| v.get("item_id").and_then(|i| i.as_str()))
            .ok_or_else(|| anyhow!("Missing item_id"))?;

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "https://graph.microsoft.com/v1.0/me/drive/items/{}/content",
                item_id
            ))
            .bearer_auth(&self.access_token)
            .send()?;

        Ok(vec![ToolContent {
            content_type: "application/octet-stream".to_string(),
            text: base64::encode(response.bytes()?),
        }])
    }
}
```

---

### 4. 其他平台集成

#### Slack

```rust
struct SlackHandler {
    bot_token: String,
}

impl McpToolHandler for SlackHandler {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        let channel = arguments
            .and_then(|v| v.get("channel").and_then(|c| c.as_str()))
            .ok_or_else(|| anyhow!("Missing channel"))?;

        let text = arguments
            .and_then(|v| v.get("text").and_then(|t| t.as_str()))
            .ok_or_else(|| anyhow!("Missing text"))?;

        let client = reqwest::blocking::Client::new();
        let response = client
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(&self.bot_token)
            .json(&json!({
                "channel": channel,
                "text": text
            }))
            .send()?;

        Ok(vec![ToolContent {
            content_type: "application/json".to_string(),
            text: response.text()?,
        }])
    }
}
```

#### Notion

```rust
struct NotionHandler {
    api_key: String,
}

impl McpToolHandler for NotionHandler {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        let page_id = arguments
            .and_then(|v| v.get("page_id").and_then(|p| p.as_str()))
            .ok_or_else(|| anyhow!("Missing page_id"))?;

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!("https://api.notion.com/v1/pages/{}", page_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Notion-Version", "2022-06-28")
            .send()?;

        Ok(vec![ToolContent {
            content_type: "application/json".to_string(),
            text: response.text()?,
        }])
    }
}
```

#### Jira

```rust
struct JiraHandler {
    api_token: String,
    domain: String, // e.g., "yourcompany.atlassian.net"
}

impl McpToolHandler for JiraHandler {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        let issue_key = arguments
            .and_then(|v| v.get("issue_key").and_then(|k| k.as_str()))
            .ok_or_else(|| anyhow!("Missing issue_key"))?;

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&format!(
                "https://{}/rest/api/3/issue/{}",
                self.domain, issue_key
            ))
            .bearer_auth(&self.api_token)
            .send()?;

        Ok(vec![ToolContent {
            content_type: "application/json".to_string(),
            text: response.text()?,
        }])
    }
}
```

---

## 自定义 MCP 工具

### 工具模板

```rust
use acsa_core::{McpTool, McpToolHandler, ToolContent};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};

/// 自定义工具处理器
struct MyToolHandler {
    // 工具所需的配置
    config: MyToolConfig,
}

struct MyToolConfig {
    api_endpoint: String,
    auth_token: String,
}

impl McpToolHandler for MyToolHandler {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        // 1. 解析参数
        let args = arguments.ok_or_else(|| anyhow!("Missing arguments"))?;

        // 2. 验证必需参数
        let required_param = args
            .get("required_param")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required_param"))?;

        // 3. 执行工具逻辑
        let result = self.execute_tool_logic(required_param)?;

        // 4. 返回结果
        Ok(vec![ToolContent {
            content_type: "text".to_string(),
            text: result,
        }])
    }
}

impl MyToolHandler {
    fn execute_tool_logic(&self, param: &str) -> Result<String> {
        // 实现工具的核心逻辑
        Ok(format!("Processed: {}", param))
    }
}

// 创建并注册工具
let tool = McpTool {
    name: "my_tool".to_string(),
    description: "My custom tool description".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "required_param": {
                "type": "string",
                "description": "Description of required parameter"
            },
            "optional_param": {
                "type": "number",
                "description": "Description of optional parameter"
            }
        },
        "required": ["required_param"]
    }),
};

mcp_server.register_tool(
    tool,
    MyToolHandler {
        config: MyToolConfig {
            api_endpoint: "https://api.example.com".to_string(),
            auth_token: std::env::var("MY_API_TOKEN")?,
        },
    }
).await;
```

---

## API 平台规则与限制

### OpenAI API (2025)

**速率限制**:
- 按模型分层限制
- 项目级别可配置限制

**定价** (GPT-4):
- Input: ~$10-30/1M tokens
- Output: ~$60-120/1M tokens

**文档**: [OpenAI Rate Limits](https://platform.openai.com/docs/guides/rate-limits)

---

### OpenRouter API (2025 更新)

**重要变更** (2025):
- ✅ 免费模型限额调整：**50次/天** (未充值账户)
- ✅ 充值用户 (余额 > $10): **1000次/天**
- ✅ 付费用户无平台级限制

**速率限制**:
- 免费模型: 20 RPM (每分钟请求数)
- 付费模型: 无平台限制（遵循上游提供商限制）

**定价**: 动态定价，根据选择的模型

**文档**: [OpenRouter Limits](https://openrouter.ai/docs/api/reference/limits)

---

### SiliconFlow API (硅基流动)

**优势**:
- 🇨🇳 国内高速访问
- 💰 极致性价比 (~$0.001-0.002/1M tokens)

**速率限制**:
- 请参考官方文档（限额因账户等级而异）

**支持模型**:
- Qwen 系列
- DeepSeek 系列
- ChatGLM 系列

**API 端点**: `https://api.siliconflow.cn/v1`

---

### Google (Gemini) API

**速率限制**:
- 免费层: 60 RPM
- 付费层: 可配置

**定价**:
- Gemini Pro: ~$2-7/1M tokens
- Gemini Ultra: 更高定价

**文档**: [Google AI Studio](https://ai.google.dev/)

---

### Claude API (Anthropic)

**速率限制**:
- 根据账户等级而异

**定价** (Claude 3.5):
- Input: ~$15/1M tokens
- Output: ~$75/1M tokens

**文档**: [Anthropic API](https://docs.anthropic.com/)

---

## 最佳实践

### 1. 错误处理

```rust
impl McpToolHandler for MyHandler {
    fn handle(&self, arguments: Option<Value>) -> Result<Vec<ToolContent>> {
        // 使用 Result 类型处理错误
        let result = self.risky_operation()
            .map_err(|e| anyhow!("Operation failed: {}", e))?;

        Ok(vec![ToolContent {
            content_type: "text".to_string(),
            text: result,
        }])
    }
}
```

### 2. 速率限制处理

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn rate_limited_call() -> Result<String> {
    let mut retries = 3;

    loop {
        match api_call().await {
            Ok(result) => return Ok(result),
            Err(e) if e.to_string().contains("429") && retries > 0 => {
                retries -= 1;
                sleep(Duration::from_secs(2_u64.pow(3 - retries))).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 3. 认证管理

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

struct TokenManager {
    token: Arc<RwLock<String>>,
    refresh_token: String,
}

impl TokenManager {
    async fn get_valid_token(&self) -> Result<String> {
        let token = self.token.read().await.clone();

        // 检查 token 是否过期
        if self.is_token_expired(&token) {
            self.refresh_access_token().await?;
        }

        Ok(self.token.read().await.clone())
    }

    async fn refresh_access_token(&self) -> Result<()> {
        // 刷新 token 逻辑
        let new_token = self.call_refresh_api(&self.refresh_token).await?;
        *self.token.write().await = new_token;
        Ok(())
    }
}
```

### 4. 资源清理

```rust
impl Drop for MyHandler {
    fn drop(&mut self) {
        // 清理资源
        println!("Cleaning up resources...");
    }
}
```

---

## 故障排查

### 问题 1: 工具未找到

**错误**: `Tool not found: my_tool`

**解决方案**:
```rust
// 确保工具已注册
mcp_server.register_tool(tool, handler).await;

// 检查工具名称是否匹配
let tools = mcp_server.tools.read().await;
println!("Registered tools: {:?}", tools.keys());
```

### 问题 2: 认证失败

**错误**: `401 Unauthorized`

**解决方案**:
```rust
// 检查环境变量
assert!(std::env::var("API_TOKEN").is_ok(), "API_TOKEN not set");

// 验证 token 格式
let token = std::env::var("API_TOKEN")?;
assert!(!token.is_empty(), "API_TOKEN is empty");
```

### 问题 3: 速率限制

**错误**: `429 Too Many Requests`

**解决方案**:
- 实现指数退避重试
- 使用速率限制器
- 考虑升级 API 计划

### 问题 4: JSON Schema 验证失败

**解决方案**:
```rust
// 使用明确的 JSON Schema
input_schema: json!({
    "type": "object",
    "properties": {
        "param": {
            "type": "string",
            "description": "Parameter description"
        }
    },
    "required": ["param"],
    "additionalProperties": false
}),
```

---

## 相关资源

### 官方文档
- [MCP 规范 (2025-11-25)](https://modelcontextprotocol.io/specification/2025-11-25)
- [Anthropic MCP 介绍](https://www.anthropic.com/news/model-context-protocol)
- [MCP GitHub](https://github.com/modelcontextprotocol)

### ACSA 文档
- [LSP 服务器指南](LSP_SERVER_GUIDE.md)
- [插件系统文档](../README.md)
- [API Provider 文档](../../README.md#ai-集成)

### 第三方 API 文档
- [Google Drive API](https://developers.google.com/drive/api)
- [GitHub API](https://docs.github.com/en/rest)
- [Slack API](https://api.slack.com/)
- [Notion API](https://developers.notion.com/)

---

## 贡献

如果您开发了新的 MCP 工具集成，欢迎贡献到 ACSA 项目！

**提交步骤**:
1. Fork 项目
2. 创建功能分支
3. 添加工具实现和文档
4. 提交 Pull Request

---

<div align="center">

**ACSA MCP Server**
*Standardized AI Application Integration*

Made with ❤️ by the ACSA Team

[GitHub](https://github.com/chen0430tw/ACSA) • [文档](../../README.md) • [Issues](https://github.com/chen0430tw/ACSA/issues)

</div>
