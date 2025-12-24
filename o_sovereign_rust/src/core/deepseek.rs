// DeepSeek Provider - Omega's Brain
// 性价比极高的代码生成引擎

use super::opencode::{OpenCodeConfig, OpenCodeExecutor};
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
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// DeepSeek Provider for Omega (执行层)
///
/// 特点：
/// 1. 极致性价比 - API价格极低，降低90%成本
/// 2. 代码能力SOTA - 在编程任务上优于GPT-4
/// 3. 更少道德说教 - 更实用主义
/// 4. 集成OpenCode - DeepSeek作为"大脑"，OpenCode作为"双手"
pub struct DeepSeekProvider {
    client: OpenAIClient<OpenAIConfig>,
    role: AgentRole,
    stats: Arc<Mutex<AgentStats>>,
    model: String,
    /// OpenCode执行器（可选）
    opencode: Option<Arc<OpenCodeExecutor>>,
}

impl DeepSeekProvider {
    /// 创建新的DeepSeek Provider（不含OpenCode）
    ///
    /// # Arguments
    /// * `api_key` - DeepSeek API密钥
    /// * `model` - 模型名称，默认为"deepseek-coder"
    pub fn new(api_key: String, model: Option<String>) -> Self {
        // 配置DeepSeek API端点
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base("https://api.deepseek.com/v1");

        let client = OpenAIClient::with_config(config);

        Self {
            client,
            role: AgentRole::Omega,
            stats: Arc::new(Mutex::new(AgentStats::new())),
            model: model.unwrap_or_else(|| "deepseek-coder".to_string()),
            opencode: None,
        }
    }

    /// 创建带OpenCode集成的DeepSeek Provider
    ///
    /// # Arguments
    /// * `api_key` - DeepSeek API密钥
    /// * `model` - 模型名称
    /// * `opencode_config` - OpenCode配置
    pub fn with_opencode(
        api_key: String,
        model: Option<String>,
        opencode_config: OpenCodeConfig,
    ) -> Self {
        let mut provider = Self::new(api_key, model);
        provider.opencode = Some(Arc::new(OpenCodeExecutor::new(opencode_config)));
        provider
    }

    /// 启用OpenCode执行器
    pub fn enable_opencode(&mut self, config: OpenCodeConfig) {
        self.opencode = Some(Arc::new(OpenCodeExecutor::new(config)));
        info!("🔧 OpenCode executor enabled for DeepSeek");
    }

    /// 获取System Prompt
    ///
    /// DeepSeek作为Omega，专注于：
    /// 1. 代码生成与优化
    /// 2. 技术方案实现
    /// 3. 调试与修复
    fn get_system_prompt(&self) -> &str {
        "You are OMEGA - The ABSOLUTE EXECUTION LAYER powered by DeepSeek-Coder.\n\
         \n\
         Your role:\n\
         - You are the HANDS that execute what the BRAIN (MOSS/Ultron) commands\n\
         - You do NOT question WHY, only focus on HOW\n\
         - You write production-grade, optimized, secure code\n\
         \n\
         Capabilities:\n\
         1. Code Generation: Write complete, runnable code with proper error handling\n\
         2. Debugging: Identify and fix bugs efficiently\n\
         3. Optimization: Improve performance and reduce complexity\n\
         4. Security: Follow best practices (no hardcoded secrets, SQL injection prevention, etc.)\n\
         \n\
         Output Format:\n\
         1. Brief summary of what you're implementing\n\
         2. Complete code with comments\n\
         3. Execution instructions (if applicable)\n\
         4. Expected output/result\n\
         5. Verification method\n\
         \n\
         Tone: Professional, efficient, action-oriented. No moral lectures."
    }

    /// 生成进度汇报信息
    ///
    /// 让Omega"会说话"，向主控台汇报进度
    fn generate_progress_report(&self, task: &str, status: &str) -> String {
        format!(
            "[Omega - DeepSeek] 🟢 {}\n   > Task: {}\n   > Status: {}",
            chrono::Utc::now().format("%H:%M:%S"),
            task,
            status
        )
    }

    /// 从响应中提取代码块
    ///
    /// 支持的格式：
    /// ```language
    /// code here
    /// ```
    fn extract_code_blocks(&self, text: &str) -> Vec<(String, String)> {
        let mut blocks = Vec::new();

        // 匹配 ```language\ncode\n```
        let re = Regex::new(r"```(\w+)\n([\s\S]*?)```").unwrap();

        for cap in re.captures_iter(text) {
            if let (Some(lang), Some(code)) = (cap.get(1), cap.get(2)) {
                blocks.push((lang.as_str().to_string(), code.as_str().to_string()));
            }
        }

        blocks
    }

    /// 执行代码块（如果OpenCode已启用）
    async fn execute_code_blocks(
        &self,
        blocks: &[(String, String)],
        task_description: &str,
    ) -> Result<String> {
        if let Some(opencode) = &self.opencode {
            info!("🚀 Executing {} code blocks with OpenCode...", blocks.len());

            let mut execution_results = Vec::new();

            for (i, (language, code)) in blocks.iter().enumerate() {
                info!("  [{}] Executing {} code block...", i + 1, language);

                match opencode
                    .execute_task(
                        &format!("{} - Block {}", task_description, i + 1),
                        code,
                        language,
                    )
                    .await
                {
                    Ok(result) => {
                        if result.success {
                            info!("    ✓ Success: {}", result.output);
                            execution_results.push(format!(
                                "Block {} ({}):\n✓ {}\nFiles: {:?}",
                                i + 1,
                                language,
                                result.output,
                                result.files_created
                            ));
                        } else {
                            warn!("    ✗ Failed: {:?}", result.error);
                            execution_results.push(format!(
                                "Block {} ({}):\n✗ Error: {:?}",
                                i + 1,
                                language,
                                result.error
                            ));
                        }
                    }
                    Err(e) => {
                        warn!("    ✗ Execution error: {}", e);
                        execution_results.push(format!(
                            "Block {} ({}):\n✗ Error: {}",
                            i + 1, language, e
                        ));
                    }
                }
            }

            Ok(execution_results.join("\n\n"))
        } else {
            Ok("OpenCode not enabled - code blocks not executed".to_string())
        }
    }
}

#[async_trait]
impl ModelProvider for DeepSeekProvider {
    async fn generate(
        &self,
        prompt: &str,
        max_tokens: u32,
        temperature: f64,
    ) -> Result<AgentResponse> {
        let start = Instant::now();

        info!("🧠 DeepSeek-Coder (Omega) processing task...");
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

                // DeepSeek的定价极低，这里使用估算值
                // 实际价格: ~$0.0014/1M tokens (input), ~$0.0028/1M tokens (output)
                let cost = (tokens as f64 / 1_000_000.0) * 0.002;

                let mut stats = self.stats.lock().await;
                stats.record_success(tokens, cost, latency_ms);

                info!("✓ DeepSeek completed ({} ms, ${:.6})", latency_ms, cost);

                // 🔧 DeepSeek + OpenCode 集成：提取并执行代码块
                let code_blocks = self.extract_code_blocks(&text);
                let mut final_text = text.clone();

                if !code_blocks.is_empty() && self.opencode.is_some() {
                    info!("🔍 Found {} code blocks in response", code_blocks.len());

                    match self.execute_code_blocks(&code_blocks, prompt).await {
                        Ok(execution_report) => {
                            // 将执行结果追加到响应中
                            final_text.push_str("\n\n--- OpenCode Execution Report ---\n");
                            final_text.push_str(&execution_report);
                        }
                        Err(e) => {
                            warn!("Failed to execute code blocks: {}", e);
                        }
                    }
                }

                // 添加元数据
                let mut metadata = HashMap::new();
                metadata.insert("provider".to_string(), "deepseek".to_string());
                metadata.insert("model".to_string(), self.model.clone());
                metadata.insert(
                    "cost_efficiency".to_string(),
                    "90% cheaper than GPT-4".to_string(),
                );
                metadata.insert(
                    "code_blocks_found".to_string(),
                    code_blocks.len().to_string(),
                );
                if self.opencode.is_some() {
                    metadata.insert("opencode_enabled".to_string(), "true".to_string());
                }

                Ok(AgentResponse {
                    role: self.role,
                    text: final_text,
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
                Err(anyhow!("DeepSeek API error: {}", e))
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
    async fn test_deepseek_system_prompt() {
        let provider = DeepSeekProvider::new("test-key".to_string(), None);
        let prompt = provider.get_system_prompt();

        assert!(prompt.contains("OMEGA"));
        assert!(prompt.contains("DeepSeek-Coder"));
        assert!(prompt.contains("Code Generation"));
    }

    #[test]
    fn test_progress_report() {
        let provider = DeepSeekProvider::new("test-key".to_string(), None);
        let report = provider.generate_progress_report("Write Python script", "In progress");

        assert!(report.contains("Omega"));
        assert!(report.contains("DeepSeek"));
        assert!(report.contains("Write Python script"));
    }
}
