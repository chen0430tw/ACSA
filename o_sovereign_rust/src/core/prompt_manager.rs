// Prompt Manager - Prompt管理系统
// Prompt版本控制、A/B测试和Few-shot示例管理
//
// 核心功能：
// 1. Prompt模板库
// 2. 版本控制
// 3. A/B测试
// 4. Few-shot示例管理
// 5. 动态Prompt构建
// 6. 性能追踪

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::protocol::Protocol;

/// Prompt模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// 模板ID
    pub template_id: String,
    /// 模板名称
    pub name: String,
    /// 模板内容（支持变量占位符）
    pub content: String,
    /// 变量列表
    pub variables: Vec<String>,
    /// Protocol关联
    pub protocol: Option<Protocol>,
    /// 版本号
    pub version: u32,
    /// 是否启用
    pub enabled: bool,
    /// 标签
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// Few-shot示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FewShotExample {
    /// 示例ID
    pub example_id: String,
    /// 用户输入
    pub user_input: String,
    /// 期望输出
    pub expected_output: String,
    /// 说明
    pub description: Option<String>,
    /// 权重（用于采样）
    pub weight: f64,
}

/// A/B测试组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbTestGroup {
    /// 测试ID
    pub test_id: String,
    /// 测试名称
    pub name: String,
    /// 变体A（模板ID）
    pub variant_a: String,
    /// 变体B（模板ID）
    pub variant_b: String,
    /// 流量分配（A的比例，0.0-1.0）
    pub traffic_split: f64,
    /// 是否激活
    pub active: bool,
    /// 开始时间
    pub started_at: DateTime<Utc>,
    /// 结束时间
    pub ended_at: Option<DateTime<Utc>>,
}

/// A/B测试指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbTestMetrics {
    /// 测试ID
    pub test_id: String,
    /// 变体
    pub variant: String,
    /// 请求次数
    pub request_count: u64,
    /// 成功次数
    pub success_count: u64,
    /// 平均响应时间（ms）
    pub avg_response_time_ms: f64,
    /// 平均成本
    pub avg_cost: f64,
    /// 用户满意度（1-5）
    pub avg_satisfaction: Option<f64>,
}

/// Prompt构建选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptBuildOptions {
    /// 是否包含Few-shot示例
    pub include_examples: bool,
    /// Few-shot示例数量
    pub num_examples: usize,
    /// 是否使用A/B测试
    pub use_ab_test: bool,
    /// 用户ID（用于A/B测试分组）
    pub user_id: Option<String>,
}

impl Default for PromptBuildOptions {
    fn default() -> Self {
        Self {
            include_examples: true,
            num_examples: 3,
            use_ab_test: false,
            user_id: None,
        }
    }
}

/// Prompt管理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptManagerConfig {
    /// 默认Few-shot示例数
    pub default_num_examples: usize,
    /// 是否启用版本控制
    pub enable_versioning: bool,
    /// 最大版本保留数
    pub max_versions: usize,
}

impl Default for PromptManagerConfig {
    fn default() -> Self {
        Self {
            default_num_examples: 3,
            enable_versioning: true,
            max_versions: 10,
        }
    }
}

/// Prompt管理器
pub struct PromptManager {
    config: PromptManagerConfig,
    /// 模板存储
    templates: Arc<RwLock<HashMap<String, PromptTemplate>>>,
    /// Few-shot示例存储
    examples: Arc<RwLock<HashMap<String, Vec<FewShotExample>>>>,
    /// A/B测试组
    ab_tests: Arc<RwLock<HashMap<String, AbTestGroup>>>,
    /// A/B测试指标
    ab_metrics: Arc<RwLock<HashMap<String, HashMap<String, AbTestMetrics>>>>,
}

impl PromptManager {
    /// 创建新的Prompt管理器
    pub fn new(config: PromptManagerConfig) -> Self {
        info!("📝 Initializing Prompt Manager");
        info!("    Default Examples: {}", config.default_num_examples);
        info!("    Versioning: {}", config.enable_versioning);

        Self {
            config,
            templates: Arc::new(RwLock::new(HashMap::new())),
            examples: Arc::new(RwLock::new(HashMap::new())),
            ab_tests: Arc::new(RwLock::new(HashMap::new())),
            ab_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册Prompt模板
    pub async fn register_template(&self, template: PromptTemplate) -> Result<()> {
        let mut templates = self.templates.write().await;

        if self.config.enable_versioning {
            // 检查是否已存在同名模板
            if let Some(existing) = templates.values().find(|t| t.name == template.name && t.template_id != template.template_id) {
                info!("📝 Creating new version of template: {}", template.name);
                // TODO: 实现版本管理逻辑
            }
        }

        templates.insert(template.template_id.clone(), template.clone());
        info!("✅ Registered template: {} (v{})", template.name, template.version);
        Ok(())
    }

    /// 获取Prompt模板
    pub async fn get_template(&self, template_id: &str) -> Result<PromptTemplate> {
        let templates = self.templates.read().await;
        templates
            .get(template_id)
            .cloned()
            .ok_or_else(|| anyhow!("Template not found: {}", template_id))
    }

    /// 添加Few-shot示例
    pub async fn add_example(&self, template_id: String, example: FewShotExample) -> Result<()> {
        let mut examples = self.examples.write().await;
        examples
            .entry(template_id)
            .or_insert_with(Vec::new)
            .push(example);

        info!("📚 Added few-shot example");
        Ok(())
    }

    /// 构建Prompt
    pub async fn build_prompt(
        &self,
        template_id: &str,
        variables: HashMap<String, String>,
        options: PromptBuildOptions,
    ) -> Result<String> {
        // 选择模板（考虑A/B测试）
        let template_id = if options.use_ab_test {
            self.select_ab_variant(template_id, options.user_id.as_deref())
                .await
                .unwrap_or_else(|_| template_id.to_string())
        } else {
            template_id.to_string()
        };

        // 获取模板
        let template = self.get_template(&template_id).await?;

        if !template.enabled {
            return Err(anyhow!("Template is disabled: {}", template.name));
        }

        // 替换变量
        let mut prompt = template.content.clone();
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            prompt = prompt.replace(&placeholder, &value);
        }

        // 添加Few-shot示例
        if options.include_examples {
            let examples_text = self.format_examples(&template_id, options.num_examples).await?;
            if !examples_text.is_empty() {
                prompt = format!("{}\n\n## 示例\n\n{}\n\n{}", prompt, examples_text, "现在请处理以下请求：");
            }
        }

        Ok(prompt)
    }

    /// 创建A/B测试
    pub async fn create_ab_test(&self, test: AbTestGroup) -> Result<()> {
        let mut tests = self.ab_tests.write().await;
        tests.insert(test.test_id.clone(), test.clone());

        info!("🧪 Created A/B test: {}", test.name);
        Ok(())
    }

    /// 记录A/B测试指标
    pub async fn record_ab_metrics(
        &self,
        test_id: &str,
        variant: &str,
        success: bool,
        response_time_ms: f64,
        cost: f64,
    ) -> Result<()> {
        let mut metrics_map = self.ab_metrics.write().await;
        let test_metrics = metrics_map.entry(test_id.to_string()).or_insert_with(HashMap::new);

        let metrics = test_metrics
            .entry(variant.to_string())
            .or_insert_with(|| AbTestMetrics {
                test_id: test_id.to_string(),
                variant: variant.to_string(),
                request_count: 0,
                success_count: 0,
                avg_response_time_ms: 0.0,
                avg_cost: 0.0,
                avg_satisfaction: None,
            });

        metrics.request_count += 1;
        if success {
            metrics.success_count += 1;
        }

        // 更新平均值
        let total = metrics.request_count as f64;
        metrics.avg_response_time_ms =
            (metrics.avg_response_time_ms * (total - 1.0) + response_time_ms) / total;
        metrics.avg_cost = (metrics.avg_cost * (total - 1.0) + cost) / total;

        Ok(())
    }

    /// 获取A/B测试结果
    pub async fn get_ab_test_results(&self, test_id: &str) -> Result<HashMap<String, AbTestMetrics>> {
        let metrics_map = self.ab_metrics.read().await;
        metrics_map
            .get(test_id)
            .cloned()
            .ok_or_else(|| anyhow!("No metrics found for test: {}", test_id))
    }

    /// 按Protocol获取推荐模板
    pub async fn get_templates_by_protocol(&self, protocol: &Protocol) -> Vec<PromptTemplate> {
        let templates = self.templates.read().await;
        templates
            .values()
            .filter(|t| t.protocol.as_ref() == Some(protocol) && t.enabled)
            .cloned()
            .collect()
    }

    /// 搜索模板
    pub async fn search_templates(&self, query: &str, tags: Vec<String>) -> Vec<PromptTemplate> {
        let templates = self.templates.read().await;
        let query_lower = query.to_lowercase();

        templates
            .values()
            .filter(|t| {
                let name_match = t.name.to_lowercase().contains(&query_lower);
                let tag_match = tags.is_empty() || tags.iter().any(|tag| t.tags.contains(tag));
                name_match && tag_match && t.enabled
            })
            .cloned()
            .collect()
    }

    // ===== 内部方法 =====

    /// 选择A/B测试变体
    async fn select_ab_variant(&self, template_id: &str, user_id: Option<&str>) -> Result<String> {
        let tests = self.ab_tests.read().await;

        // 查找包含该模板的激活测试
        let test = tests
            .values()
            .find(|t| t.active && (t.variant_a == template_id || t.variant_b == template_id))
            .ok_or_else(|| anyhow!("No active A/B test found for template"))?;

        // 简单的确定性分配（避免使用rand）
        let hash_value = if let Some(uid) = user_id {
            uid.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64))
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
        };

        let use_variant_a = (hash_value as f64 / u64::MAX as f64) < test.traffic_split;

        Ok(if use_variant_a {
            test.variant_a.clone()
        } else {
            test.variant_b.clone()
        })
    }

    /// 格式化Few-shot示例
    async fn format_examples(&self, template_id: &str, num: usize) -> Result<String> {
        let examples_map = self.examples.read().await;

        let examples = examples_map
            .get(template_id)
            .ok_or_else(|| anyhow!("No examples found for template"))?;

        if examples.is_empty() {
            return Ok(String::new());
        }

        // TODO: 实现加权采样
        let selected = examples.iter().take(num);

        let mut formatted = String::new();
        for (i, example) in selected.enumerate() {
            formatted.push_str(&format!(
                "### 示例 {}\n**输入**: {}\n**输出**: {}\n\n",
                i + 1,
                example.user_input,
                example.expected_output
            ));
        }

        Ok(formatted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_template_registration() {
        let manager = PromptManager::new(PromptManagerConfig::default());

        let template = PromptTemplate {
            template_id: "test1".to_string(),
            name: "Test Template".to_string(),
            content: "Hello {{name}}!".to_string(),
            variables: vec!["name".to_string()],
            protocol: Some(Protocol::Architect),
            version: 1,
            enabled: true,
            tags: vec!["greeting".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        manager.register_template(template).await.unwrap();

        let retrieved = manager.get_template("test1").await.unwrap();
        assert_eq!(retrieved.name, "Test Template");
    }

    #[tokio::test]
    async fn test_prompt_building() {
        let manager = PromptManager::new(PromptManagerConfig::default());

        let template = PromptTemplate {
            template_id: "test1".to_string(),
            name: "Test".to_string(),
            content: "Hello {{name}}!".to_string(),
            variables: vec!["name".to_string()],
            protocol: None,
            version: 1,
            enabled: true,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        manager.register_template(template).await.unwrap();

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());

        let prompt = manager
            .build_prompt("test1", vars, PromptBuildOptions {
                include_examples: false,
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(prompt, "Hello World!");
    }
}
