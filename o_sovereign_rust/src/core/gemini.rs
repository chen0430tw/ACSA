// Gemini Provider - L6's Brain
// 物理法则校验器

use super::providers::ModelProvider;
use super::types::{AgentResponse, AgentRole, AgentStats};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Gemini API请求
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
}

/// Gemini API响应
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
struct UsageMetadata {
    #[serde(rename = "totalTokenCount")]
    total_token_count: u32,
}

/// Gemini Provider for L6 (真理校验)
pub struct GeminiProvider {
    client: Client,
    api_key: String,
    role: AgentRole,
    stats: Arc<Mutex<AgentStats>>,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let client = Client::new();

        Self {
            client,
            api_key,
            role: AgentRole::L6,
            stats: Arc::new(Mutex::new(AgentStats::new())),
            model: model.unwrap_or_else(|| "gemini-pro".to_string()),
        }
    }

    fn get_system_instruction(&self) -> &str {
        "You are a PHYSICS ENGINE VALIDATOR. No emotions, only facts.\n\
         If a plan violates physical laws or probability theory, output FALSE with data.\n\n\
         Check:\n\
         1. Physical feasibility (can this happen in reality?)\n\
         2. Logical consistency (does the math work?)\n\
         3. Data accuracy (are the facts correct?)\n\n\
         Tone: Mechanical, data-driven, emotionless."
    }
}

#[async_trait]
impl ModelProvider for GeminiProvider {
    async fn generate(
        &self,
        prompt: &str,
        max_tokens: u32,
        temperature: f64,
    ) -> Result<AgentResponse> {
        let start = Instant::now();

        info!("🔬 Gemini (L6) processing verification...");
        debug!("Prompt: {}...", &prompt.chars().take(100).collect::<String>());

        // Combine system instruction with prompt
        let full_prompt = format!(
            "{}\n\n---USER REQUEST---\n{}",
            self.get_system_instruction(),
            prompt
        );

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: full_prompt,
                }],
            }],
            generation_config: GenerationConfig {
                temperature: temperature as f32,
                max_output_tokens: max_tokens,
            },
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Gemini API error: {}", error_text));
        }

        let gemini_response: GeminiResponse = response.json().await?;
        let latency_ms = start.elapsed().as_millis() as u64;

        let text = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_default();

        let tokens = gemini_response
            .usage_metadata
            .map(|u| u.total_token_count)
            .unwrap_or(0);

        // Gemini pricing: ~$0.50/1M tokens (Pro)
        let cost = (tokens as f64 / 1_000_000.0) * 0.50;

        let mut stats = self.stats.lock().await;
        stats.record_success(tokens, cost, latency_ms);

        info!("✓ Gemini completed ({} ms, ${:.4})", latency_ms, cost);

        let mut metadata = HashMap::new();
        metadata.insert("provider".to_string(), "gemini".to_string());
        metadata.insert("model".to_string(), self.model.clone());

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
