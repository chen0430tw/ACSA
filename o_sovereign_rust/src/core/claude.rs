// Claude Provider - Ultron's Brain
// 红队审计专家

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

/// Claude API请求
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    system: Option<String>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Claude API响应
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Claude Provider for Ultron (红队审计)
pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    role: AgentRole,
    stats: Arc<Mutex<AgentStats>>,
    model: String,
}

impl ClaudeProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let client = Client::new();

        Self {
            client,
            api_key,
            role: AgentRole::Ultron,
            stats: Arc::new(Mutex::new(AgentStats::new())),
            model: model.unwrap_or_else(|| "claude-3-opus-20240229".to_string()),
        }
    }

    fn get_system_prompt(&self) -> &str {
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
}

#[async_trait]
impl ModelProvider for ClaudeProvider {
    async fn generate(
        &self,
        prompt: &str,
        max_tokens: u32,
        temperature: f64,
    ) -> Result<AgentResponse> {
        let start = Instant::now();

        info!("🤖 Claude (Ultron) processing audit...");
        debug!("Prompt: {}...", &prompt.chars().take(100).collect::<String>());

        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            system: Some(self.get_system_prompt().to_string()),
            temperature: temperature as f32,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Claude API error: {}", error_text));
        }

        let claude_response: ClaudeResponse = response.json().await?;
        let latency_ms = start.elapsed().as_millis() as u64;

        let text = claude_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        let total_tokens = claude_response.usage.input_tokens + claude_response.usage.output_tokens;

        // Claude pricing: ~$15/1M input tokens, ~$75/1M output tokens (Opus)
        let cost = (claude_response.usage.input_tokens as f64 / 1_000_000.0) * 15.0
            + (claude_response.usage.output_tokens as f64 / 1_000_000.0) * 75.0;

        let mut stats = self.stats.lock().await;
        stats.record_success(total_tokens, cost, latency_ms);

        info!("✓ Claude completed ({} ms, ${:.4})", latency_ms, cost);

        let mut metadata = HashMap::new();
        metadata.insert("provider".to_string(), "claude".to_string());
        metadata.insert("model".to_string(), self.model.clone());

        Ok(AgentResponse {
            role: self.role,
            text,
            tokens: total_tokens,
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
