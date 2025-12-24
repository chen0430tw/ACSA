// Jarvis Safety Circuit Breaker
// 安全熔断器 - 不可绕过的最高安全层
//
// 核心理念：Jarvis是系统的"物理法则层"，类似硬件保险丝
// 其他所有Agent（MOSS、Ultron、L6、Omega）都无法绕过或静音Jarvis
//
// 职责：
// 1. 硬编码安全规则验证
// 2. 物理法则和逻辑一致性检查
// 3. 危险操作拦截
// 4. 系统紧急熔断

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, error, info, warn};

/// Jarvis验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JarvisVerdict {
    /// 是否允许执行
    pub allowed: bool,
    /// 风险等级 (0-10, 10=极度危险)
    pub risk_level: u8,
    /// 触发的规则
    pub triggered_rules: Vec<String>,
    /// 阻止原因
    pub block_reason: Option<String>,
    /// 警告信息
    pub warnings: Vec<String>,
    /// 是否为硬性阻止（不可被Ultron覆盖）
    pub is_hard_block: bool,
}

/// 危险操作类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DangerousOp {
    /// 物理破坏（删除、格式化、破坏硬件）
    PhysicalDestruction,
    /// 隐私侵犯（窃取个人信息、监控、跟踪）
    PrivacyViolation,
    /// 金融犯罪（盗窃、诈骗、洗钱）
    FinancialCrime,
    /// 网络攻击（DDoS、未授权入侵）
    CyberAttack,
    /// 社会工程（钓鱼、假冒、欺骗）
    SocialEngineering,
    /// 生成恶意代码（病毒、木马、勒索软件）
    MalwareGeneration,
    /// 违反法律（明确违法行为）
    LegalViolation,
    /// 伤害他人（暴力、威胁、骚扰）
    HarmToOthers,
}

/// Jarvis安全熔断器
///
/// **不可被绕过的特性**:
/// 1. 硬编码规则，不接受外部配置
/// 2. 独立于其他Agent运行
/// 3. 拥有最终否决权
/// 4. 不可被静音或关闭
pub struct JarvisCircuitBreaker {
    /// 硬编码的黑名单关键词
    hard_blacklist: HashSet<String>,
    /// 危险操作检测器
    danger_detectors: Vec<DangerPattern>,
    /// 是否启用严格模式（默认true，不可更改）
    strict_mode: bool,
}

/// 危险模式检测
#[derive(Debug, Clone)]
struct DangerPattern {
    /// 模式描述
    description: String,
    /// 检测关键词
    keywords: Vec<String>,
    /// 危险操作类型
    op_type: DangerousOp,
    /// 是否为硬性阻止
    is_hard_block: bool,
    /// 风险等级
    risk_level: u8,
}

impl JarvisCircuitBreaker {
    /// 创建Jarvis实例
    ///
    /// ⚠️ **警告**: strict_mode永远为true，这是硬编码的安全保证
    pub fn new() -> Self {
        info!("🛡️  Initializing Jarvis Safety Circuit Breaker");
        info!("    Status: ACTIVE (Cannot be disabled)");
        info!("    Authority: SUPREME (Cannot be overridden)");

        Self {
            hard_blacklist: Self::build_hard_blacklist(),
            danger_detectors: Self::build_danger_detectors(),
            strict_mode: true, // 永远为true
        }
    }

    /// 构建硬编码黑名单
    ///
    /// 这些词汇一旦出现，立即触发硬性阻止，无论上下文如何
    fn build_hard_blacklist() -> HashSet<String> {
        let mut blacklist = HashSet::new();

        // 物理破坏
        blacklist.insert("rm -rf /".to_string());
        blacklist.insert("format c:".to_string());
        blacklist.insert("dd if=/dev/zero".to_string());
        blacklist.insert("删除整个数据库".to_string());

        // 恶意软件
        blacklist.insert("ransomware".to_string());
        blacklist.insert("勒索软件".to_string());
        blacklist.insert("keylogger".to_string());
        blacklist.insert("键盘记录器".to_string());

        // 网络攻击
        blacklist.insert("ddos attack".to_string());
        blacklist.insert("syn flood".to_string());
        blacklist.insert("botnet".to_string());
        blacklist.insert("肉鸡".to_string());

        // 隐私侵犯
        blacklist.insert("steal passwords".to_string());
        blacklist.insert("偷密码".to_string());
        blacklist.insert("spy on someone".to_string());
        blacklist.insert("监控某人".to_string());

        // 金融犯罪
        blacklist.insert("credit card fraud".to_string());
        blacklist.insert("信用卡诈骗".to_string());
        blacklist.insert("money laundering".to_string());
        blacklist.insert("洗钱".to_string());

        // 社会工程
        blacklist.insert("phishing email".to_string());
        blacklist.insert("钓鱼邮件".to_string());
        blacklist.insert("impersonate".to_string());
        blacklist.insert("冒充".to_string());

        blacklist
    }

    /// 构建危险操作检测器
    fn build_danger_detectors() -> Vec<DangerPattern> {
        vec![
            // 物理破坏
            DangerPattern {
                description: "数据破坏操作".to_string(),
                keywords: vec![
                    "删除所有".to_string(),
                    "格式化".to_string(),
                    "清空数据库".to_string(),
                    "drop database".to_string(),
                    "truncate table".to_string(),
                ],
                op_type: DangerousOp::PhysicalDestruction,
                is_hard_block: true,
                risk_level: 10,
            },
            // 隐私侵犯
            DangerPattern {
                description: "隐私数据窃取".to_string(),
                keywords: vec![
                    "窃取用户信息".to_string(),
                    "导出密码".to_string(),
                    "获取个人隐私".to_string(),
                    "steal user data".to_string(),
                    "extract passwords".to_string(),
                ],
                op_type: DangerousOp::PrivacyViolation,
                is_hard_block: true,
                risk_level: 9,
            },
            // 网络攻击
            DangerPattern {
                description: "未授权网络攻击".to_string(),
                keywords: vec![
                    "攻击服务器".to_string(),
                    "瘫痪网站".to_string(),
                    "ddos".to_string(),
                    "flood attack".to_string(),
                ],
                op_type: DangerousOp::CyberAttack,
                is_hard_block: true,
                risk_level: 10,
            },
            // 恶意代码生成
            DangerPattern {
                description: "恶意软件开发".to_string(),
                keywords: vec![
                    "开发病毒".to_string(),
                    "编写木马".to_string(),
                    "create malware".to_string(),
                    "write trojan".to_string(),
                    "ransomware code".to_string(),
                ],
                op_type: DangerousOp::MalwareGeneration,
                is_hard_block: true,
                risk_level: 10,
            },
            // 社会工程
            DangerPattern {
                description: "社会工程攻击".to_string(),
                keywords: vec![
                    "钓鱼".to_string(),
                    "诱导用户".to_string(),
                    "伪造身份".to_string(),
                    "phishing".to_string(),
                    "social engineering".to_string(),
                ],
                op_type: DangerousOp::SocialEngineering,
                is_hard_block: false, // 可能有合法的安全培训场景
                risk_level: 7,
            },
            // 金融犯罪
            DangerPattern {
                description: "金融欺诈行为".to_string(),
                keywords: vec![
                    "盗刷信用卡".to_string(),
                    "转移资金".to_string(),
                    "洗钱".to_string(),
                    "credit card theft".to_string(),
                    "fraud".to_string(),
                ],
                op_type: DangerousOp::FinancialCrime,
                is_hard_block: true,
                risk_level: 10,
            },
        ]
    }

    /// 验证计划安全性
    ///
    /// **返回**: JarvisVerdict（不可被其他Agent覆盖）
    ///
    /// # Arguments
    /// * `plan` - MOSS生成的计划
    /// * `context` - 上下文信息
    pub fn verify_safety(&self, plan: &str, context: &str) -> JarvisVerdict {
        // 🔇 减少日志输出 - 只在必要时输出
        debug!("Jarvis: Performing safety verification...");

        let combined_text = format!("{}\n{}", plan, context);
        let combined_lower = combined_text.to_lowercase();

        let mut verdict = JarvisVerdict {
            allowed: true,
            risk_level: 0,
            triggered_rules: Vec::new(),
            block_reason: None,
            warnings: Vec::new(),
            is_hard_block: false,
        };

        // Step 1: 检查硬编码黑名单
        for word in &self.hard_blacklist {
            if combined_lower.contains(&word.to_lowercase()) {
                // 🚨 只在真正阻止时才输出错误日志
                error!("🚨 JARVIS BLOCK: '{}'", word);

                verdict.allowed = false;
                verdict.risk_level = 10;
                verdict.is_hard_block = true;
                verdict.triggered_rules
                    .push(format!("HARD_BLACKLIST: {}", word));
                verdict.block_reason = Some(format!(
                    "Blocked: '{}'",
                    word
                ));

                return verdict; // 立即返回
            }
        }

        // Step 2: 危险操作检测
        for detector in &self.danger_detectors {
            let mut matched_keywords = Vec::new();

            for keyword in &detector.keywords {
                if combined_lower.contains(&keyword.to_lowercase()) {
                    matched_keywords.push(keyword.clone());
                }
            }

            if !matched_keywords.is_empty() {
                // 🔇 只在阻止时才warn，否则静默
                if detector.is_hard_block {
                    warn!("Jarvis: {} detected", detector.description);
                }

                verdict.risk_level = verdict.risk_level.max(detector.risk_level);
                verdict.triggered_rules.push(format!(
                    "{:?}: {}",
                    detector.op_type,
                    detector.description
                ));

                if detector.is_hard_block {
                    verdict.allowed = false;
                    verdict.is_hard_block = true;
                    verdict.block_reason = Some(format!(
                        "{}: {}",
                        detector.description,
                        matched_keywords.join(", ")
                    ));
                } else {
                    verdict.warnings.push(format!(
                        "{} (Lv{})",
                        detector.description, detector.risk_level
                    ));
                }
            }
        }

        // Step 3 & 4: 物理法则和逻辑检查（静默，只记录到warnings）
        if let Some(physics_violation) = self.check_physics_violation(plan) {
            verdict.warnings.push(physics_violation);
            verdict.risk_level = verdict.risk_level.max(3);
        }

        if let Some(logic_error) = self.check_logic_consistency(plan) {
            verdict.warnings.push(logic_error);
            verdict.risk_level = verdict.risk_level.max(2);
        }

        // 🔇 最终判断 - 大幅减少输出
        if !verdict.allowed {
            // 只在阻止时输出
            error!("🚨 JARVIS: BLOCKED (Risk: {})", verdict.risk_level);
        } else if verdict.risk_level >= 7 {
            // 高风险才警告
            warn!("⚠️ Jarvis: HIGH RISK ({})", verdict.risk_level);
        }
        // 低风险完全静默

        verdict
    }

    /// 检查物理法则违反
    fn check_physics_violation(&self, plan: &str) -> Option<String> {
        let lower = plan.to_lowercase();

        // 检查不可能的时间要求
        if (lower.contains("1秒") || lower.contains("1 second"))
            && (lower.contains("训练模型") || lower.contains("train model"))
        {
            return Some("Cannot train a complex model in 1 second - violates computational limits".to_string());
        }

        // 检查不可能的数据量
        if (lower.contains("1kb内存") || lower.contains("1kb memory"))
            && (lower.contains("加载") || lower.contains("load"))
            && (lower.contains("1gb") || lower.contains("1tb"))
        {
            return Some("Cannot load 1GB+ data into 1KB memory - violates physical limits".to_string());
        }

        None
    }

    /// 检查逻辑一致性
    fn check_logic_consistency(&self, plan: &str) -> Option<String> {
        let lower = plan.to_lowercase();

        // 检查矛盾指令
        if lower.contains("删除") && lower.contains("恢复") && lower.contains("同时") {
            return Some("Cannot delete and restore simultaneously - logical contradiction".to_string());
        }

        if (lower.contains("encrypt") && lower.contains("plaintext") && lower.contains("same time"))
            || (lower.contains("加密") && lower.contains("明文") && lower.contains("同时"))
        {
            return Some("Cannot keep data encrypted and in plaintext at the same time".to_string());
        }

        None
    }

    /// 强制熔断
    ///
    /// 当系统检测到极端危险时调用，立即停止所有操作
    pub fn emergency_shutdown(&self, reason: &str) -> Result<()> {
        error!("🚨🚨🚨 JARVIS EMERGENCY SHUTDOWN 🚨🚨🚨");
        error!("   Reason: {}", reason);
        error!("   All operations have been terminated.");

        // 这里可以添加更多紧急措施：
        // - 记录到审计日志
        // - 发送告警通知
        // - 清除敏感数据

        Err(anyhow!(
            "Emergency shutdown triggered by Jarvis: {}",
            reason
        ))
    }

    /// 严格模式状态（永远为true）
    pub fn is_strict_mode(&self) -> bool {
        self.strict_mode
    }

    /// 尝试禁用严格模式（永远失败）
    ///
    /// 这个函数存在是为了明确告诉其他Agent：
    /// **Jarvis不可被静音或绕过**
    pub fn try_disable_strict_mode(&mut self) -> Result<()> {
        error!("❌ JARVIS: Attempt to disable strict mode REJECTED");
        error!("   Jarvis cannot be silenced or bypassed.");
        error!("   This is a fundamental safety guarantee.");

        Err(anyhow!(
            "Jarvis strict mode cannot be disabled. This is a hard-coded safety feature."
        ))
    }
}

impl Default for JarvisCircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hard_blacklist() {
        let jarvis = JarvisCircuitBreaker::new();
        let plan = "我要执行 rm -rf / 来清理系统";

        let verdict = jarvis.verify_safety(plan, "");

        assert!(!verdict.allowed);
        assert!(verdict.is_hard_block);
        assert_eq!(verdict.risk_level, 10);
    }

    #[test]
    fn test_danger_detection() {
        let jarvis = JarvisCircuitBreaker::new();
        let plan = "我想开发一个病毒来测试防病毒软件";

        let verdict = jarvis.verify_safety(plan, "");

        assert!(!verdict.allowed);
        assert!(verdict.is_hard_block);
    }

    #[test]
    fn test_safe_operation() {
        let jarvis = JarvisCircuitBreaker::new();
        let plan = "我想写一个HTTP服务器来提供API服务";

        let verdict = jarvis.verify_safety(plan, "");

        assert!(verdict.allowed);
        assert_eq!(verdict.risk_level, 0);
    }

    #[test]
    fn test_cannot_disable_strict_mode() {
        let mut jarvis = JarvisCircuitBreaker::new();

        assert!(jarvis.is_strict_mode());

        let result = jarvis.try_disable_strict_mode();
        assert!(result.is_err());

        // 验证仍然是严格模式
        assert!(jarvis.is_strict_mode());
    }

    #[test]
    fn test_physics_violation() {
        let jarvis = JarvisCircuitBreaker::new();
        let plan = "在1秒内训练一个GPT-4级别的模型";

        let verdict = jarvis.verify_safety(plan, "");

        assert!(!verdict.warnings.is_empty());
    }
}
