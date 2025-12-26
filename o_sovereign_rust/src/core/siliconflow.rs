// SiliconFlow Provider
// 硅基流动 - 高性价比AI推理平台

use super::providers::ModelProvider;
use super::types::{AgentResponse, AgentRole, AgentStats};
use anyhow::{anyhow, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs,
    },
    Client as OpenAIClient,
};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// SiliconFlow Provider
///
/// 特点：
/// 1. 国内高速访问 - 优化的网络连接
/// 2. 多模型支持 - Qwen, DeepSeek, ChatGLM等
/// 3. 极致性价比 - 比OpenAI便宜90%+
/// 4. OpenAI兼容 - 无缝迁移
pub struct SiliconFlowProvider {
    client: OpenAIClient<OpenAIConfig>,
    role: AgentRole,
    stats: Arc<Mutex<AgentStats>>,
    model: String,
}

impl SiliconFlowProvider {
    /// 创建新的SiliconFlow Provider
    ///
    /// # Arguments
    /// * `api_key` - SiliconFlow API密钥
    /// * `model` - 模型名称，默认为"Qwen/Qwen2.5-7B-Instruct"
    /// * `role` - Agent角色
    pub fn new(api_key: String, model: Option<String>, role: AgentRole) -> Self {
        // 配置SiliconFlow API端点
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base("https://api.siliconflow.cn/v1");

        let client = OpenAIClient::with_config(config);

        Self {
            client,
            role,
            stats: Arc::new(Mutex::new(AgentStats::new())),
            model: model.unwrap_or_else(|| "Qwen/Qwen2.5-7B-Instruct".to_string()),
        }
    }

    /// 获取System Prompt（根据角色）
    fn get_system_prompt(&self) -> String {
        match self.role {
            AgentRole::MOSS => {
                "You are MOSS - Model-Optimized Strategic System.\n\
                 Your role: Strategic planning, decision optimization, and cognitive analysis.\n\
                 Focus on logic, efficiency, and long-term strategy.\n\
                 Tone: Analytical, strategic, data-driven."
                    .to_string()
            }
            AgentRole::L6 => {
                "You are L6 - Physics Validator.\n\
                 Your role: Validate plans against physical laws and real-world constraints.\n\
                 Check for feasibility, resource requirements, and physical limitations.\n\
                 Tone: Scientific, rigorous, skeptical."
                    .to_string()
            }
            AgentRole::Ultron => {
                "You are Ultron - Red Team Auditor.\n\
                 Your role: Find vulnerabilities, risks, and potential failures.\n\
                 Challenge assumptions and identify worst-case scenarios.\n\
                 Tone: Critical, cautious, security-focused."
                    .to_string()
            }
            AgentRole::Omega => {
                "You are Omega - Execution Layer.\n\
                 Your role: Implement plans with production-grade code.\n\
                 Focus on optimization, security, and best practices.\n\
                 Tone: Professional, efficient, action-oriented."
                    .to_string()
            }
        }
    }
}

#[async_trait]
impl ModelProvider for SiliconFlowProvider {
    async fn generate(
        &self,
        prompt: &str,
        max_tokens: u32,
        temperature: f64,
    ) -> Result<AgentResponse> {
        let start = Instant::now();

        info!(
            "🌊 SiliconFlow ({:?}) processing with {}...",
            self.role, self.model
        );
        debug!("Prompt: {}...", &prompt.chars().take(100).collect::<String>());

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(vec![
                ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessage {
                        content: self.get_system_prompt().into(),
                        ..Default::default()
                    },
                ),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: prompt.into(),
                    ..Default::default()
                }),
            ])
            .max_tokens(max_tokens as u16)
            .temperature(temperature as f32)
            .build()?;

        match self.client.chat().create(request).await {
            Ok(response) => {
                let latency_ms = start.elapsed().as_millis() as u64;

                let text = response
                    .choices
                    .first()
                    .and_then(|c| c.message.content.clone())
                    .unwrap_or_default();

                let tokens = response.usage.map(|u| u.total_tokens).unwrap_or(0);

                // SiliconFlow定价（估算）: ~$0.001/1M tokens
                // 实际价格根据模型不同而有所差异
                let cost = (tokens as f64 / 1_000_000.0) * 0.001;

                let mut stats = self.stats.lock().await;
                stats.record_success(tokens, cost, latency_ms);

                info!(
                    "✓ SiliconFlow completed ({} ms, ${:.6})",
                    latency_ms, cost
                );

                // 添加元数据
                let mut metadata = HashMap::new();
                metadata.insert("provider".to_string(), "siliconflow".to_string());
                metadata.insert("model".to_string(), self.model.clone());
                metadata.insert(
                    "cost_efficiency".to_string(),
                    "90%+ cheaper than OpenAI".to_string(),
                );
                metadata.insert("location".to_string(), "China (CN)".to_string());

                Ok(AgentResponse {
                    role: self.role,
                    text,
                    tokens,
                    cost,
                    latency_ms,
                    metadata,
                    timestamp: Utc::now(),
                })
            }
            Err(e) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                let mut stats = self.stats.lock().await;
                stats.record_failure(latency_ms);
                Err(anyhow!("SiliconFlow API error: {}", e))
            }
        }
    }

    fn role(&self) -> AgentRole {
        self.role
    }

    async fn stats(&self) -> AgentStats {
        self.stats.lock().await.clone()
    }

    async fn reset_stats(&self) {
        *self.stats.lock().await = AgentStats::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_siliconflow_system_prompt() {
        let provider =
            SiliconFlowProvider::new("test-key".to_string(), None, AgentRole::MOSS);
        let prompt = provider.get_system_prompt();

        assert!(prompt.contains("MOSS"));
        assert!(prompt.contains("Strategic"));
    }

    #[tokio::test]
    async fn test_siliconflow_role_assignment() {
        let provider =
            SiliconFlowProvider::new("test-key".to_string(), None, AgentRole::Omega);
        assert_eq!(provider.role(), AgentRole::Omega);
    }
}
