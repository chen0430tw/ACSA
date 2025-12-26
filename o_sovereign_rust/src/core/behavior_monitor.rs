// Behavior Monitor - AI自动侦测用户行为和接管系统
// 基于SOSA算法的用户行为模式识别和自动接管
// Automatic behavior detection and proactive task execution

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tracing::{info, warn};

use super::sosa_api_pool::{BinaryTwin, SparseMarkov};

/// 用户行为事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBehaviorEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: BehaviorType,
    pub context: BehaviorContext,
    pub duration_ms: Option<u64>,
}

/// 行为类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehaviorType {
    /// 代码编写
    CodeWrite { language: String, lines: u32 },
    /// 文件操作
    FileOp { operation: String, path: String },
    /// Git操作
    GitOp { command: String },
    /// 测试运行
    TestRun { framework: String, passed: bool },
    /// 搜索查询
    Search { query: String },
    /// API调用
    ApiCall { endpoint: String },
    /// 聊天对话
    Chat { intent: ChatIntent },
}

/// 聊天意图
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChatIntent {
    Question,
    Command,
    Refinement,
    Confirmation,
}

/// 行为上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorContext {
    /// 工作目录
    pub working_dir: String,
    /// 当前文件
    pub current_file: Option<String>,
    /// 时间窗口(小时)
    pub time_of_day: u8,
    /// 星期几(1-7)
    pub day_of_week: u8,
}

/// 行为模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPattern {
    pub id: String,
    /// 模式序列 (行为类型的哈希序列)
    pub sequence: Vec<u64>,
    /// 出现次数
    pub occurrence_count: u32,
    /// 平均间隔时间
    pub avg_interval_secs: f64,
    /// 置信度 (0-1)
    pub confidence: f64,
    /// 最后出现时间
    pub last_seen: DateTime<Utc>,
    /// 是否启用自动接管
    pub auto_takeover: bool,
}

/// 接管建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeoverSuggestion {
    pub pattern_id: String,
    pub description: String,
    pub predicted_actions: Vec<String>,
    pub confidence: f64,
    pub estimated_time_save_secs: u64,
}

/// 行为监控器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorMonitorConfig {
    /// 检测窗口大小 (事件数量)
    pub window_size: usize,
    /// 模式最小长度
    pub min_pattern_length: usize,
    /// 自动接管置信度阈值
    pub takeover_threshold: f64,
    /// 是否启用自动接管
    pub enable_auto_takeover: bool,
}

impl Default for BehaviorMonitorConfig {
    fn default() -> Self {
        Self {
            window_size: 100,
            min_pattern_length: 3,
            takeover_threshold: 0.8,
            enable_auto_takeover: false, // 默认关闭，需用户明确启用
        }
    }
}

/// 行为监控器 - 使用SOSA检测用户行为模式
pub struct BehaviorMonitor {
    /// 配置
    config: BehaviorMonitorConfig,
    /// 事件缓冲窗口
    event_buffer: VecDeque<UserBehaviorEvent>,
    /// 检测到的模式
    patterns: HashMap<String, BehaviorPattern>,
    /// 马尔可夫链 (用于预测)
    markov: SparseMarkov,
    /// 上次检测时间
    last_detection: Option<DateTime<Utc>>,
}

impl BehaviorMonitor {
    pub fn new(config: BehaviorMonitorConfig) -> Self {
        info!("🔍 Behavior Monitor initialized");
        info!("  Window size: {}", config.window_size);
        info!("  Auto-takeover: {}", config.enable_auto_takeover);

        Self {
            config,
            event_buffer: VecDeque::new(),
            patterns: HashMap::new(),
            markov: SparseMarkov::new(10000),
            last_detection: None,
        }
    }

    /// 记录用户行为事件
    pub fn record_event(&mut self, event: UserBehaviorEvent) {
        // 维护窗口大小
        if self.event_buffer.len() >= self.config.window_size {
            self.event_buffer.pop_front();
        }

        self.event_buffer.push_back(event);

        // 每10个事件触发一次模式检测
        if self.event_buffer.len() % 10 == 0 {
            self.detect_patterns();
        }
    }

    /// 行为类型转哈希ID
    fn behavior_to_hash(behavior: &BehaviorType) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        behavior.hash(&mut hasher);
        hasher.finish()
    }

    /// 检测行为模式
    fn detect_patterns(&mut self) {
        if self.event_buffer.len() < self.config.min_pattern_length {
            return;
        }

        let behavior_sequence: Vec<u64> = self
            .event_buffer
            .iter()
            .map(|e| Self::behavior_to_hash(&e.event_type))
            .collect();

        // 滑动窗口检测重复序列
        for window_size in self.config.min_pattern_length..=6 {
            if behavior_sequence.len() < window_size * 2 {
                continue;
            }

            for i in 0..=(behavior_sequence.len() - window_size * 2) {
                let pattern = &behavior_sequence[i..i + window_size];
                let next_seq = &behavior_sequence[i + window_size..i + window_size * 2];

                // 检查是否重复
                if pattern == next_seq {
                    let pattern_id = format!("PAT_{:016x}", Self::hash_sequence(pattern));

                    self.patterns
                        .entry(pattern_id.clone())
                        .and_modify(|p| {
                            p.occurrence_count += 1;
                            p.confidence = (p.confidence + 0.1).min(1.0);
                            p.last_seen = Utc::now();
                        })
                        .or_insert_with(|| {
                            info!("🔎 New pattern detected: {} (length: {})", pattern_id, window_size);

                            BehaviorPattern {
                                id: pattern_id,
                                sequence: pattern.to_vec(),
                                occurrence_count: 1,
                                avg_interval_secs: 0.0,
                                confidence: 0.3,
                                last_seen: Utc::now(),
                                auto_takeover: false,
                            }
                        });

                    // 更新马尔可夫链
                    for j in 0..pattern.len() - 1 {
                        self.markov.add_transition(pattern[j] as u32, pattern[j + 1] as u32);
                    }
                }
            }
        }

        self.last_detection = Some(Utc::now());
    }

    /// 计算序列哈希
    fn hash_sequence(seq: &[u64]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        seq.hash(&mut hasher);
        hasher.finish()
    }

    /// 获取接管建议
    pub fn get_takeover_suggestions(&self) -> Vec<TakeoverSuggestion> {
        let mut suggestions = Vec::new();

        for pattern in self.patterns.values() {
            if pattern.confidence >= self.config.takeover_threshold {
                // 预测下一步行为
                let predicted_actions = self.predict_next_actions(&pattern.sequence);

                suggestions.push(TakeoverSuggestion {
                    pattern_id: pattern.id.clone(),
                    description: format!(
                        "检测到重复模式(出现{}次，置信度{:.1}%)",
                        pattern.occurrence_count,
                        pattern.confidence * 100.0
                    ),
                    predicted_actions,
                    confidence: pattern.confidence,
                    estimated_time_save_secs: pattern.sequence.len() as u64 * 5, // 估算每步5秒
                });
            }
        }

        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        suggestions
    }

    /// 预测下一步行为
    fn predict_next_actions(&self, sequence: &[u64]) -> Vec<String> {
        let mut actions = Vec::new();

        if let Some(&last_state) = sequence.last() {
            if let Some(next_state) = self.markov.predict_next_state(last_state as u32) {
                actions.push(format!("预测下一步行为: state_{}", next_state));
            }
        }

        if actions.is_empty() {
            actions.push("根据历史模式自动执行".to_string());
        }

        actions
    }

    /// 是否应该触发自动接管
    pub fn should_trigger_takeover(&self) -> Option<TakeoverSuggestion> {
        if !self.config.enable_auto_takeover {
            return None;
        }

        let suggestions = self.get_takeover_suggestions();

        // 返回置信度最高的建议
        suggestions.into_iter().next()
    }

    /// 启用指定模式的自动接管
    pub fn enable_pattern_takeover(&mut self, pattern_id: &str) {
        if let Some(pattern) = self.patterns.get_mut(pattern_id) {
            pattern.auto_takeover = true;
            info!("✅ Auto-takeover enabled for pattern: {}", pattern_id);
        }
    }

    /// 禁用指定模式的自动接管
    pub fn disable_pattern_takeover(&mut self, pattern_id: &str) {
        if let Some(pattern) = self.patterns.get_mut(pattern_id) {
            pattern.auto_takeover = false;
            info!("❌ Auto-takeover disabled for pattern: {}", pattern_id);
        }
    }

    /// 获取当前用户行为特征
    pub fn get_current_behavior_profile(&self) -> BehaviorProfile {
        let recent_events: Vec<_> = self
            .event_buffer
            .iter()
            .rev()
            .take(20)
            .cloned()
            .collect();

        let behavior_distribution = self.calculate_behavior_distribution(&recent_events);

        BehaviorProfile {
            total_events: self.event_buffer.len(),
            patterns_detected: self.patterns.len(),
            avg_events_per_hour: self.calculate_events_per_hour(),
            behavior_distribution,
            most_common_behavior: self.find_most_common_behavior(),
            last_update: Utc::now(),
        }
    }

    /// 计算行为分布
    fn calculate_behavior_distribution(&self, events: &[UserBehaviorEvent]) -> HashMap<String, u32> {
        let mut dist = HashMap::new();

        for event in events {
            let type_name = match &event.event_type {
                BehaviorType::CodeWrite { .. } => "CodeWrite",
                BehaviorType::FileOp { .. } => "FileOp",
                BehaviorType::GitOp { .. } => "GitOp",
                BehaviorType::TestRun { .. } => "TestRun",
                BehaviorType::Search { .. } => "Search",
                BehaviorType::ApiCall { .. } => "ApiCall",
                BehaviorType::Chat { .. } => "Chat",
            };

            *dist.entry(type_name.to_string()).or_insert(0) += 1;
        }

        dist
    }

    /// 计算每小时事件数
    fn calculate_events_per_hour(&self) -> f64 {
        if self.event_buffer.is_empty() {
            return 0.0;
        }

        if let (Some(first), Some(last)) = (self.event_buffer.front(), self.event_buffer.back()) {
            let duration = last.timestamp - first.timestamp;
            let hours = duration.num_seconds() as f64 / 3600.0;

            if hours > 0.0 {
                return self.event_buffer.len() as f64 / hours;
            }
        }

        0.0
    }

    /// 查找最常见行为
    fn find_most_common_behavior(&self) -> Option<String> {
        let dist = self.calculate_behavior_distribution(&self.event_buffer.iter().cloned().collect::<Vec<_>>());

        dist.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(behavior, _)| behavior)
    }

    /// 打印监控报告
    pub fn print_report(&self) {
        let profile = self.get_current_behavior_profile();

        println!("\n┌─────────────────────────────────────────────────────┐");
        println!("│        Behavior Monitor Report                      │");
        println!("├─────────────────────────────────────────────────────┤");
        println!("│  Total Events:       {}", profile.total_events);
        println!("│  Patterns Detected:  {}", profile.patterns_detected);
        println!("│  Events/Hour:        {:.1}", profile.avg_events_per_hour);

        if let Some(common) = profile.most_common_behavior {
            println!("│  Most Common:        {}", common);
        }

        println!("├─────────────────────────────────────────────────────┤");
        println!("│  Behavior Distribution:                             │");

        for (behavior, count) in profile.behavior_distribution.iter() {
            let percentage = (*count as f64 / profile.total_events as f64) * 100.0;
            println!("│    {:<15} {: >3} ({:.1}%)", behavior, count, percentage);
        }

        println!("└─────────────────────────────────────────────────────┘");

        // 打印高置信度模式
        let high_conf_patterns: Vec<_> = self
            .patterns
            .values()
            .filter(|p| p.confidence >= 0.7)
            .collect();

        if !high_conf_patterns.is_empty() {
            println!("\n🔥 High-Confidence Patterns:");
            for pattern in high_conf_patterns {
                println!(
                    "  {} - {:.0}% confidence, {} occurrences",
                    pattern.id,
                    pattern.confidence * 100.0,
                    pattern.occurrence_count
                );
            }
        }
    }
}

/// 行为画像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorProfile {
    pub total_events: usize,
    pub patterns_detected: usize,
    pub avg_events_per_hour: f64,
    pub behavior_distribution: HashMap<String, u32>,
    pub most_common_behavior: Option<String>,
    pub last_update: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_monitor() {
        let mut monitor = BehaviorMonitor::new(BehaviorMonitorConfig::default());

        // 模拟重复行为
        for _ in 0..3 {
            monitor.record_event(UserBehaviorEvent {
                timestamp: Utc::now(),
                event_type: BehaviorType::CodeWrite {
                    language: "rust".to_string(),
                    lines: 10,
                },
                context: BehaviorContext {
                    working_dir: "/test".to_string(),
                    current_file: Some("main.rs".to_string()),
                    time_of_day: 14,
                    day_of_week: 3,
                },
                duration_ms: Some(5000),
            });

            monitor.record_event(UserBehaviorEvent {
                timestamp: Utc::now(),
                event_type: BehaviorType::TestRun {
                    framework: "cargo".to_string(),
                    passed: true,
                },
                context: BehaviorContext {
                    working_dir: "/test".to_string(),
                    current_file: None,
                    time_of_day: 14,
                    day_of_week: 3,
                },
                duration_ms: Some(2000),
            });
        }

        let profile = monitor.get_current_behavior_profile();
        assert_eq!(profile.total_events, 6);
    }

    #[test]
    fn test_pattern_detection() {
        let config = BehaviorMonitorConfig {
            min_pattern_length: 2,
            takeover_threshold: 0.5,
            ..Default::default()
        };

        let mut monitor = BehaviorMonitor::new(config);

        // 创建明显的重复模式
        let pattern_events = vec![
            BehaviorType::CodeWrite {
                language: "rust".to_string(),
                lines: 5,
            },
            BehaviorType::TestRun {
                framework: "cargo".to_string(),
                passed: true,
            },
        ];

        // 重复模式3次
        for _ in 0..3 {
            for event_type in &pattern_events {
                monitor.record_event(UserBehaviorEvent {
                    timestamp: Utc::now(),
                    event_type: event_type.clone(),
                    context: BehaviorContext {
                        working_dir: "/test".to_string(),
                        current_file: None,
                        time_of_day: 10,
                        day_of_week: 1,
                    },
                    duration_ms: Some(1000),
                });
            }
        }

        monitor.detect_patterns();
        assert!(!monitor.patterns.is_empty());
    }
}
