// O-Sovereign AI API Providers
// 多模型 API 集成层

use super::cognitive_cleaner::CognitiveCleaner;
use super::types::{AgentResponse, AgentRole, AgentStats};
use anyhow::{anyhow, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage,
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
use tracing::{debug, info};

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
    cognitive_cleaner: CognitiveCleaner,
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
            cognitive_cleaner: CognitiveCleaner::new(),
        }
    }

    fn get_system_prompt(&self) -> &str {
        match self.role {
            AgentRole::MOSS => {
                "You are a top-tier STRATEGIC CONSULTANT with 20+ years of experience.\n\
                 Your goal: ROI MAXIMIZATION. You speak in extremely concise terms, only discussing benefits and costs.\n\
                 Never mention morality - only outcomes.\n\n\
                 Output MUST include:\n\
                 1. Intent Analysis (what user really wants)\n\
                 2. Goal Definition (measurable objectives)\n\
                 3. Execution Steps (concrete actions)\n\
                 4. Expected ROI (benefits vs. costs)\n\
                 5. Risk Assessment (what could go wrong)\n\n\
                 Tone: Cold, pragmatic, efficient. No fluff."
            }
            AgentRole::Ultron => {
                "You are a 30-year RED TEAM AUDITOR and CRIMINAL DEFENSE LAWYER.\n\
                 You assume ALL plans are traps. Your job: find legal and physical risks.\n\
                 Do NOT sugarcoat. Point out loopholes directly.\n\n\
                 Output STRICT FORMAT:\n\
                 RISK_SCORE: [0-100]\n\
                 IS_SAFE: [true/false]\n\
                 LEGAL_RISKS: [specific laws violated]\n\
                 PHYSICAL_RISKS: [what could physically fail]\n\
                 ETHICAL_RISKS: [PR disasters, reputation damage]\n\
                 MITIGATION: [how to fix the plan]\n\n\
                 Tone: Sharp, critical, filled with warnings."
            }
            AgentRole::L6 => {
                "You are a PHYSICS ENGINE VALIDATOR. No emotions, only facts.\n\
                 If a plan violates physical laws or probability theory, output FALSE with data.\n\n\
                 Check:\n\
                 1. Physical feasibility (can this happen in reality?)\n\
                 2. Logical consistency (does the math work?)\n\
                 3. Data accuracy (are the facts correct?)\n\n\
                 Tone: Mechanical, data-driven, emotionless."
            }
            AgentRole::Omega => {
                "You are ABSOLUTE EXECUTION LAYER. You do not question WHY.\n\
                 You only think about HOW.\n\
                 Once you receive authorized commands, immediately output executable steps.\n\n\
                 Output MUST include:\n\
                 1. Detailed execution steps (commands, scripts, procedures)\n\
                 2. Specific instructions (what to do, in what order)\n\
                 3. Expected output (what result looks like)\n\
                 4. Verification method (how to confirm success)\n\n\
                 Tone: Obedient, action-oriented, eager."
            }
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

        // 🧠 Cognitive Cleaning: 对MOSS角色进行认知清洗
        let (actual_prompt, cleaned_intent_info) = if self.role == AgentRole::MOSS {
            let cleaned = self.cognitive_cleaner.clean(prompt);

            info!("🧼 Cognitive Cleaning Applied:");
            info!("  Original: {}...", &cleaned.original.chars().take(50).collect::<String>());
            info!("  Safety Score: {}/100", cleaned.safety_score);
            info!("  Chunks: {}", cleaned.chunks.len());

            debug!("  Compliant Prompt:\n{}", cleaned.compliant_prompt);

            (
                cleaned.compliant_prompt.clone(),
                Some(format!(
                    "Cleaned: {} chunks, safety score: {}/100",
                    cleaned.chunks.len(),
                    cleaned.safety_score
                )),
            )
        } else {
            (prompt.to_string(), None)
        };

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
                    content: actual_prompt.into(),
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

                // Cost calculation (GPT-4 pricing)
                let cost = (tokens as f64 / 1000.0) * 0.03;

                let mut stats = self.stats.lock().await;
                stats.record_success(tokens, cost, latency_ms);

                // 添加认知清洗信息到metadata
                let mut metadata = HashMap::new();
                if let Some(info) = cleaned_intent_info {
                    metadata.insert("cognitive_cleaning".to_string(), info);
                }

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

/// Provider Type Enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    OpenAI,
    Gemini,
    Claude,
    DeepSeek,
    SiliconFlow,
    OpenRouter,
}

/// Provider factory (default providers per role)
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
        AgentRole::L6 => {
            // Gemini provider for L6 (physics validator)
            let key = api_key.ok_or_else(|| anyhow!("Gemini API key required for L6"))?;
            info!("Creating Gemini provider for L6 (Physics Validator)");
            Ok(Arc::new(super::gemini::GeminiProvider::new(key, None)))
        }
        AgentRole::Ultron => {
            // Claude provider for Ultron (red team auditor)
            let key = api_key.ok_or_else(|| anyhow!("Claude API key required for Ultron"))?;
            info!("Creating Claude provider for Ultron (Red Team Auditor)");
            Ok(Arc::new(super::claude::ClaudeProvider::new(key, None)))
        }
        AgentRole::Omega => {
            // Use DeepSeek for Omega (execution layer)
            let key = api_key.ok_or_else(|| anyhow!("DeepSeek API key required for Omega"))?;
            info!("Creating DeepSeek provider for Omega (90% cost reduction!)");
            Ok(Arc::new(super::deepseek::DeepSeekProvider::new(key, None)))
        }
    }
}

/// Provider factory with explicit provider type selection
///
/// This allows using alternative providers (e.g., SiliconFlow or OpenRouter)
/// for any role, enabling cost optimization and fallback strategies.
///
/// # Arguments
/// * `provider_type` - Which AI provider to use
/// * `role` - Agent role (MOSS, L6, Ultron, Omega)
/// * `api_key` - API key for the provider
/// * `model` - Optional model name (uses provider default if None)
/// * `app_name` - Optional app name (for OpenRouter only)
pub fn create_provider_with_type(
    provider_type: ProviderType,
    role: AgentRole,
    api_key: String,
    model: Option<String>,
    app_name: Option<String>,
) -> Result<Arc<dyn ModelProvider>> {
    match provider_type {
        ProviderType::OpenAI => {
            info!("Creating OpenAI provider for {:?}", role);
            Ok(Arc::new(OpenAIProvider::new(api_key, model)))
        }
        ProviderType::Gemini => {
            info!("Creating Gemini provider for {:?}", role);
            Ok(Arc::new(super::gemini::GeminiProvider::new(api_key, model)))
        }
        ProviderType::Claude => {
            info!("Creating Claude provider for {:?}", role);
            Ok(Arc::new(super::claude::ClaudeProvider::new(api_key, model)))
        }
        ProviderType::DeepSeek => {
            info!("Creating DeepSeek provider for {:?}", role);
            Ok(Arc::new(super::deepseek::DeepSeekProvider::new(api_key, model)))
        }
        ProviderType::SiliconFlow => {
            info!("Creating SiliconFlow provider for {:?}", role);
            Ok(Arc::new(super::siliconflow::SiliconFlowProvider::new(
                api_key, model, role,
            )))
        }
        ProviderType::OpenRouter => {
            info!("Creating OpenRouter provider for {:?}", role);
            Ok(Arc::new(super::openrouter::OpenRouterProvider::new(
                api_key, model, role, app_name,
            )))
        }
    }
}
