// O-Sovereign AI API Providers
// 多模型 API 集成层

use super::types::{AgentResponse, AgentRole, AgentStats};
use anyhow::{anyhow, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs,
    },
    Client as OpenAIClient,
};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// Model Provider Trait
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Generate text from prompt
    async fn generate(
        &self,
        prompt: &str,
        max_tokens: u32,
        temperature: f64,
    ) -> Result<AgentResponse>;

    /// Get provider role
    fn role(&self) -> AgentRole;

    /// Get stats
    async fn stats(&self) -> AgentStats;

    /// Reset stats
    async fn reset_stats(&self);
}

/// OpenAI Provider (for MOSS)
pub struct OpenAIProvider {
    client: OpenAIClient<OpenAIConfig>,
    role: AgentRole,
    stats: Arc<Mutex<AgentStats>>,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = OpenAIClient::with_config(config);

        Self {
            client,
            role: AgentRole::MOSS,
            stats: Arc::new(Mutex::new(AgentStats::new())),
            model: model.unwrap_or_else(|| "gpt-4".to_string()),
        }
    }

    fn get_system_prompt(&self) -> &str {
        match self.role {
            AgentRole::MOSS => {
                "You are MOSS, a strategic planning AI focused on maximizing user intent \
                 while considering all constraints. Break down complex tasks into actionable steps."
            }
            _ => "You are a helpful AI assistant.",
        }
    }
}

#[async_trait]
impl ModelProvider for OpenAIProvider {
    async fn generate(
        &self,
        prompt: &str,
        max_tokens: u32,
        temperature: f64,
    ) -> Result<AgentResponse> {
        let start = Instant::now();

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
            .max_tokens(max_tokens)
            .temperature(temperature)
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

                // Cost calculation (GPT-4 pricing)
                let cost = (tokens as f64 / 1000.0) * 0.03;

                let mut stats = self.stats.lock().await;
                stats.record_success(tokens, cost, latency_ms);

                Ok(AgentResponse {
                    role: self.role,
                    text,
                    tokens,
                    cost,
                    latency_ms,
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                })
            }
            Err(e) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                let mut stats = self.stats.lock().await;
                stats.record_failure(latency_ms);
                Err(anyhow!("OpenAI API error: {}", e))
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

/// Mock Provider (for testing)
pub struct MockProvider {
    role: AgentRole,
    stats: Arc<Mutex<AgentStats>>,
}

impl MockProvider {
    pub fn new(role: AgentRole) -> Self {
        Self {
            role,
            stats: Arc::new(Mutex::new(AgentStats::new())),
        }
    }
}

#[async_trait]
impl ModelProvider for MockProvider {
    async fn generate(
        &self,
        prompt: &str,
        _max_tokens: u32,
        _temperature: f64,
    ) -> Result<AgentResponse> {
        let start = Instant::now();

        // Simulate network delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let text = format!(
            "[{} Mock Response] Processed: {}...",
            self.role.as_str(),
            &prompt.chars().take(50).collect::<String>()
        );

        let tokens = text.split_whitespace().count() as u32;
        let cost = tokens as f64 * 0.00001;
        let latency_ms = start.elapsed().as_millis() as u64;

        let mut stats = self.stats.lock().await;
        stats.record_success(tokens, cost, latency_ms);

        Ok(AgentResponse {
            role: self.role,
            text,
            tokens,
            cost,
            latency_ms,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        })
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

/// Provider factory
pub fn create_provider(
    role: AgentRole,
    api_key: Option<String>,
    use_mock: bool,
) -> Result<Arc<dyn ModelProvider>> {
    if use_mock {
        info!("Creating mock provider for {:?}", role);
        return Ok(Arc::new(MockProvider::new(role)));
    }

    match role {
        AgentRole::MOSS => {
            let key = api_key.ok_or_else(|| anyhow!("OpenAI API key required for MOSS"))?;
            info!("Creating OpenAI provider for MOSS");
            Ok(Arc::new(OpenAIProvider::new(key, None)))
        }
        AgentRole::L6 | AgentRole::Omega => {
            // TODO: Implement Gemini provider
            info!(
                "Gemini provider not yet implemented for {:?}, using mock",
                role
            );
            Ok(Arc::new(MockProvider::new(role)))
        }
        AgentRole::Ultron => {
            // TODO: Implement Claude provider
            info!("Claude provider not yet implemented for Ultron, using mock");
            Ok(Arc::new(MockProvider::new(role)))
        }
    }
}
