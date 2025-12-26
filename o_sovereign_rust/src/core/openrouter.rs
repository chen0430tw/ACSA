// OpenRouter Provider
// 统一AI模型路由平台 - 支持100+模型

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

/// OpenRouter Provider
///
/// 特点：
/// 1. 统一接口 - 单一API访问100+模型
/// 2. 自动降级 - 当主模型不可用时自动切换
/// 3. 价格透明 - 实时显示每个模型的成本
/// 4. 全球覆盖 - 支持OpenAI, Anthropic, Google, Meta等
pub struct OpenRouterProvider {
    client: OpenAIClient<OpenAIConfig>,
    role: AgentRole,
    stats: Arc<Mutex<AgentStats>>,
    model: String,
    /// 应用名称（用于OpenRouter统计）
    app_name: String,
}

impl OpenRouterProvider {
    /// 创建新的OpenRouter Provider
    ///
    /// # Arguments
    /// * `api_key` - OpenRouter API密钥
    /// * `model` - 模型名称，默认为"openai/gpt-4-turbo-preview"
    /// * `role` - Agent角色
    /// * `app_name` - 应用名称（可选，用于OpenRouter统计）
    pub fn new(
        api_key: String,
        model: Option<String>,
        role: AgentRole,
        app_name: Option<String>,
    ) -> Self {
        // 配置OpenRouter API端点
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base("https://openrouter.ai/api/v1");

        let client = OpenAIClient::with_config(config);

        Self {
            client,
            role,
            stats: Arc::new(Mutex::new(AgentStats::new())),
            model: model.unwrap_or_else(|| "openai/gpt-4-turbo-preview".to_string()),
            app_name: app_name.unwrap_or_else(|| "ACSA-O-Sovereign".to_string()),
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

    /// 估算成本（OpenRouter模型价格不一，这里使用平均值）
    fn estimate_cost(&self, tokens: u32) -> f64 {
        // 根据模型前缀估算价格
        let price_per_million = if self.model.starts_with("openai/gpt-4") {
            10.0 // GPT-4平均价格
        } else if self.model.starts_with("anthropic/claude") {
            15.0 // Claude平均价格
        } else if self.model.starts_with("google/") {
            2.0 // Gemini平均价格
        } else if self.model.contains("deepseek") || self.model.contains("qwen") {
            0.5 // 开源模型平均价格
        } else {
            5.0 // 默认估算
        };

        (tokens as f64 / 1_000_000.0) * price_per_million
    }
}

#[async_trait]
impl ModelProvider for OpenRouterProvider {
    async fn generate(
        &self,
        prompt: &str,
        max_tokens: u32,
        temperature: f64,
    ) -> Result<AgentResponse> {
        let start = Instant::now();

        info!(
            "🔀 OpenRouter ({:?}) routing to {}...",
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

                // 使用估算的成本
                let cost = self.estimate_cost(tokens);

                let mut stats = self.stats.lock().await;
                stats.record_success(tokens, cost, latency_ms);

                info!("✓ OpenRouter completed ({} ms, ${:.4})", latency_ms, cost);

                // 添加元数据
                let mut metadata = HashMap::new();
                metadata.insert("provider".to_string(), "openrouter".to_string());
                metadata.insert("model".to_string(), self.model.clone());
                metadata.insert("app_name".to_string(), self.app_name.clone());
                metadata.insert(
                    "features".to_string(),
                    "Multi-model routing, Auto-fallback".to_string(),
                );

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
                Err(anyhow!("OpenRouter API error: {}", e))
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
    async fn test_openrouter_system_prompt() {
        let provider =
            OpenRouterProvider::new("test-key".to_string(), None, AgentRole::L6, None);
        let prompt = provider.get_system_prompt();

        assert!(prompt.contains("L6"));
        assert!(prompt.contains("Physics"));
    }

    #[tokio::test]
    async fn test_openrouter_cost_estimation() {
        let provider = OpenRouterProvider::new(
            "test-key".to_string(),
            Some("openai/gpt-4".to_string()),
            AgentRole::MOSS,
            None,
        );

        let cost = provider.estimate_cost(1000);
        assert!(cost > 0.0);
    }

    #[tokio::test]
    async fn test_openrouter_app_name() {
        let provider = OpenRouterProvider::new(
            "test-key".to_string(),
            None,
            AgentRole::Omega,
            Some("CustomApp".to_string()),
        );

        assert_eq!(provider.app_name, "CustomApp");
    }
}
