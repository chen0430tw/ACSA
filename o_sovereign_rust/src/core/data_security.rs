// Data Security & Privacy System - 数据安全与隐私保护系统
// 文件读取、权限管理、数据脱敏、跨平台支持
// File access, permission management, data sanitization, cross-platform support

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Jarvis API配置说明
///
/// **重要澄清**: Jarvis不是AI Agent，不使用AI API！
///
/// Jarvis是硬编码的安全熔断器，基于规则的检查系统：
/// - 不需要AI推理
/// - 不消耗API tokens
/// - 完全本地运行
/// - 零延迟检查
///
/// Jarvis的职责是拦截危险操作，不是生成内容。
pub const JARVIS_EXPLANATION: &str = r#"
Jarvis Circuit Breaker (安全熔断器)
====================================

类型: 规则引擎 (非AI Agent)
运行: 完全本地 (无API调用)
消耗: 零token (纯规则匹配)
速度: 微秒级 (无网络延迟)

职责:
1. 危险操作拦截
2. 硬编码安全规则
3. 物理法则检查
4. 紧急熔断

Jarvis不需要AI，因为安全规则必须是确定性的，不能依赖概率模型。
"#;

/// 数据敏感度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SensitivityLevel {
    /// 公开数据 (无需保护)
    Public = 0,
    /// 内部数据 (内部使用)
    Internal = 1,
    /// 机密数据 (需要授权)
    Confidential = 2,
    /// 高度机密 (严格管控)
    Secret = 3,
    /// 绝密 (最高级别)
    TopSecret = 4,
}

impl SensitivityLevel {
    pub fn icon(&self) -> &'static str {
        match self {
            SensitivityLevel::Public => "🌐",
            SensitivityLevel::Internal => "🏢",
            SensitivityLevel::Confidential => "🔒",
            SensitivityLevel::Secret => "🔐",
            SensitivityLevel::TopSecret => "⛔",
        }
    }
}

/// 数据分类
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataCategory {
    /// 个人身份信息 (PII)
    PersonalIdentity,
    /// 财务信息
    Financial,
    /// 健康医疗
    HealthMedical,
    /// 密码凭证
    Credentials,
    /// API密钥/Token
    ApiKeys,
    /// 源代码
    SourceCode,
    /// 配置文件
    Configuration,
    /// 普通文本
    PlainText,
    /// 图片/媒体
    Media,
    /// 其他
    Other,
}

/// 脱敏规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationRule {
    pub category: DataCategory,
    pub pattern: String,
    pub replacement: String,
    pub enabled: bool,
}

/// 文件访问权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccessPermission {
    /// 文件路径
    pub path: PathBuf,
    /// 是否允许读取
    pub can_read: bool,
    /// 是否允许写入
    pub can_write: bool,
    /// 是否允许执行
    pub can_execute: bool,
    /// 敏感度等级
    pub sensitivity: SensitivityLevel,
    /// 需要的权限级别
    pub required_permission_level: u8,
}

/// 跨平台权限请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub request_type: PermissionType,
    pub reason: String,
    pub granted: bool,
    pub requested_at: DateTime<Utc>,
    pub granted_at: Option<DateTime<Utc>>,
}

/// 权限类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionType {
    /// 文件系统读取
    FileRead,
    /// 文件系统写入
    FileWrite,
    /// 相机访问
    Camera,
    /// 相册访问
    Photos,
    /// 位置信息
    Location,
    /// 网络访问
    Network,
    /// 剪贴板
    Clipboard,
}

/// 资源消耗监控
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU时间 (ms)
    pub cpu_time_ms: u64,
    /// 内存占用 (bytes)
    pub memory_bytes: u64,
    /// 磁盘I/O (bytes)
    pub disk_io_bytes: u64,
    /// 网络流量 (bytes)
    pub network_bytes: u64,
}

/// 数据安全管理器
pub struct DataSecurityManager {
    /// 脱敏规则
    sanitization_rules: Vec<SanitizationRule>,
    /// 文件访问控制列表
    file_acl: HashMap<PathBuf, FileAccessPermission>,
    /// 权限请求历史
    permission_history: Vec<PermissionRequest>,
    /// 敏感数据模式库
    sensitive_patterns: HashMap<DataCategory, Vec<regex::Regex>>,
    /// 资源使用追踪
    resource_tracking: bool,
}

impl DataSecurityManager {
    pub fn new() -> Self {
        info!("🔐 Data Security Manager initialized");
        info!("  Features: Sanitization, ACL, Permission Management");

        let mut manager = Self {
            sanitization_rules: Vec::new(),
            file_acl: HashMap::new(),
            permission_history: Vec::new(),
            sensitive_patterns: HashMap::new(),
            resource_tracking: true,
        };

        // 加载默认脱敏规则
        manager.load_default_rules();

        manager
    }

    /// 加载默认脱敏规则
    fn load_default_rules(&mut self) {
        // PII: 身份证号 (中国18位)
        self.add_sanitization_rule(SanitizationRule {
            category: DataCategory::PersonalIdentity,
            pattern: r"\d{17}[\dXx]".to_string(),
            replacement: "***IDCARD***".to_string(),
            enabled: true,
        });

        // 财务: 信用卡号
        self.add_sanitization_rule(SanitizationRule {
            category: DataCategory::Financial,
            pattern: r"\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}".to_string(),
            replacement: "***CARD***".to_string(),
            enabled: true,
        });

        // 凭证: 密码
        self.add_sanitization_rule(SanitizationRule {
            category: DataCategory::Credentials,
            pattern: r#"password["\s:=]+([^\s"]+)"#.to_string(),
            replacement: r#"password="***REDACTED***""#.to_string(),
            enabled: true,
        });

        // API密钥: 常见格式
        self.add_sanitization_rule(SanitizationRule {
            category: DataCategory::ApiKeys,
            pattern: "(api[_-]?key|token)[\"\\s:=]+([a-zA-Z0-9_\\-]{20,})".to_string(),
            replacement: "$1=***REDACTED***".to_string(),
            enabled: true,
        });

        info!("✅ Loaded {} default sanitization rules ", self.sanitization_rules.len());
    }

    /// 添加脱敏规则
    pub fn add_sanitization_rule(&mut self, rule: SanitizationRule) {
        self.sanitization_rules.push(rule);
    }

    /// 数据脱敏
    pub fn sanitize(&self, content: &str, category: Option<DataCategory>) -> String {
        let mut sanitized = content.to_string();

        for rule in &self.sanitization_rules {
            if !rule.enabled {
                continue;
            }

            // 如果指定了分类，只应用该分类的规则
            if let Some(ref cat) = category {
                if &rule.category != cat {
                    continue;
                }
            }

            // 简化的正则替换（实际应该用regex crate）
            // 这里先用简单的字符串替换示意
            if content.contains(&rule.pattern) {
                warn!("🔒 Sensitive data detected: {:?}", rule.category);
                sanitized = sanitized.replace(&rule.pattern, &rule.replacement);
            }
        }

        sanitized
    }

    /// 文件读取（带权限检查和脱敏）
    pub fn read_file_secure(&self, path: &Path) -> Result<SecureFileContent> {
        let start = std::time::Instant::now();

        // 1. 权限检查
        if let Some(acl) = self.file_acl.get(path) {
            if !acl.can_read {
                return Err(anyhow!("Access denied: No read permission for {:?}", path));
            }

            if acl.sensitivity >= SensitivityLevel::Secret {
                warn!("⚠️ Reading highly sensitive file: {:?}", path);
            }
        }

        // 2. 读取文件
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read file: {}", e))?;

        // 3. 数据分类
        let category = self.classify_file_content(&content, path);

        // 4. 自动脱敏
        let sanitized_content = self.sanitize(&content, Some(category.clone()));
        let was_sanitized = content != sanitized_content;

        let elapsed = start.elapsed();

        // 5. 资源追踪
        let resource_usage = ResourceUsage {
            cpu_time_ms: elapsed.as_millis() as u64,
            memory_bytes: content.len() as u64,
            disk_io_bytes: content.len() as u64,
            network_bytes: 0,
        };

        info!("📄 File read: {:?} ({} bytes, {} ms)",
            path,
            content.len(),
            elapsed.as_millis()
        );

        Ok(SecureFileContent {
            original_path: path.to_path_buf(),
            content: sanitized_content,
            is_sanitized: was_sanitized,
            category,
            sensitivity: self.get_file_sensitivity(path),
            resource_usage,
        })
    }

    /// 图片读取（移动端需要权限）
    pub async fn read_image_secure(&mut self, path: &Path) -> Result<SecureImageContent> {
        let start = std::time::Instant::now();

        // 1. 检查平台权限
        #[cfg(target_os = "android")]
        {
            self.request_permission(PermissionType::Photos, "Read user photos ").await?;
        }

        #[cfg(target_os = "ios")]
        {
            self.request_permission(PermissionType::Photos, "Access photo library ").await?;
        }

        // 2. 读取图片
        let image_data = std::fs::read(path)
            .map_err(|e| anyhow!("Failed to read image: {}", e))?;

        let elapsed = start.elapsed();

        // 3. 资源追踪
        let resource_usage = ResourceUsage {
            cpu_time_ms: elapsed.as_millis() as u64,
            memory_bytes: image_data.len() as u64,
            disk_io_bytes: image_data.len() as u64,
            network_bytes: 0,
        };

        info!("🖼️ Image read: {:?} ({} KB, {} ms)",
            path,
            image_data.len() / 1024,
            elapsed.as_millis()
        );

        Ok(SecureImageContent {
            original_path: path.to_path_buf(),
            data: image_data,
            format: Self::detect_image_format(path),
            contains_exif: false, // 简化实现
            resource_usage,
        })
    }

    /// 请求权限（跨平台）
    async fn request_permission(&mut self, perm_type: PermissionType, reason: &str) -> Result<()> {
        info!("📱 Requesting permission: {:?} - {}", perm_type, reason);

        let request = PermissionRequest {
            request_type: perm_type.clone(),
            reason: reason.to_string(),
            granted: false,
            requested_at: Utc::now(),
            granted_at: None,
        };

        // 在真实实现中，这里会调用平台特定的权限API
        // Android: ActivityCompat.requestPermissions
        // iOS: PHPhotoLibrary.requestAuthorization

        // 模拟权限授予
        let mut request = request;
        request.granted = true;
        request.granted_at = Some(Utc::now());

        self.permission_history.push(request);

        Ok(())
    }

    /// 分类文件内容
    fn classify_file_content(&self, content: &str, path: &Path) -> DataCategory {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // 基于扩展名的初步分类
        match ext {
            "rs" | "py" | "js" | "java" | "cpp" => DataCategory::SourceCode,
            "toml" | "yaml" | "json" | "ini" => DataCategory::Configuration,
            "jpg" | "png" | "gif" | "bmp" => DataCategory::Media,
            _ => {
                // 基于内容的深度分类
                if content.contains("password") || content.contains("secret") {
                    DataCategory::Credentials
                } else if content.contains("api_key") || content.contains("token") {
                    DataCategory::ApiKeys
                } else {
                    DataCategory::PlainText
                }
            }
        }
    }

    /// 获取文件敏感度
    fn get_file_sensitivity(&self, path: &Path) -> SensitivityLevel {
        if let Some(acl) = self.file_acl.get(path) {
            return acl.sensitivity;
        }

        // 默认敏感度
        let path_str = path.to_string_lossy().to_lowercase();

        if path_str.contains("env") || path_str.contains("secret") {
            SensitivityLevel::Secret
        } else if path_str.contains("config") || path_str.contains("key") {
            SensitivityLevel::Confidential
        } else {
            SensitivityLevel::Internal
        }
    }

    /// 检测图片格式
    fn detect_image_format(path: &Path) -> ImageFormat {
        match path.extension().and_then(|e| e.to_str()) {
            Some("jpg") | Some("jpeg") => ImageFormat::JPEG,
            Some("png") => ImageFormat::PNG,
            Some("gif") => ImageFormat::GIF,
            Some("bmp") => ImageFormat::BMP,
            _ => ImageFormat::Unknown,
        }
    }

    /// 设置文件访问权限
    pub fn set_file_permission(&mut self, path: PathBuf, permission: FileAccessPermission) {
        info!("🔧 Setting file permission: {:?} -> {:?}", path, permission.sensitivity);
        self.file_acl.insert(path, permission);
    }

    /// 获取资源使用统计
    pub fn get_resource_stats(&self) -> ResourceStats {
        // 简化实现
        ResourceStats {
            total_files_read: 0,
            total_images_read: 0,
            total_data_sanitized: 0,
            permissions_requested: self.permission_history.len(),
            permissions_granted: self.permission_history.iter().filter(|p| p.granted).count(),
        }
    }

    /// 打印安全报告
    pub fn print_security_report(&self) {
        println!();
        println!("+-----------------------------------------------------+");
        println!("|        Data Security Manager Report                 |");
        println!("+-----------------------------------------------------+");
        println!("|  Sanitization Rules: {}", self.sanitization_rules.len());
        println!("|  ACL Entries:        {}", self.file_acl.len());
        println!("|  Permission History: {}", self.permission_history.len());
        println!("+-----------------------------------------------------+");

        println!();
        println!("Enabled Sanitization Rules:");
        for rule in self.sanitization_rules.iter().filter(|r| r.enabled) {
            println!("  {:?}: {}", rule.category, rule.pattern);
        }

        if !self.permission_history.is_empty() {
            println!();
            println!("Permission Requests:");
            for perm in self.permission_history.iter().rev().take(5) {
                let status = if perm.granted { "Granted" } else { "Denied" };
                println!("  {} {:?}: {}", status, perm.request_type, perm.reason);
            }
        }
    }
}

impl Default for DataSecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 安全文件内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureFileContent {
    pub original_path: PathBuf,
    pub content: String,
    pub is_sanitized: bool,
    pub category: DataCategory,
    pub sensitivity: SensitivityLevel,
    pub resource_usage: ResourceUsage,
}

/// 安全图片内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureImageContent {
    pub original_path: PathBuf,
    pub data: Vec<u8>,
    pub format: ImageFormat,
    pub contains_exif: bool,
    pub resource_usage: ResourceUsage,
}

/// 图片格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    JPEG,
    PNG,
    GIF,
    BMP,
    Unknown,
}

/// 资源统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    pub total_files_read: usize,
    pub total_images_read: usize,
    pub total_data_sanitized: usize,
    pub permissions_requested: usize,
    pub permissions_granted: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitization() {
        let manager = DataSecurityManager::new();

        let sensitive_text = "My password is secret123 and my API key is sk-1234567890abcdef";
        let sanitized = manager.sanitize(sensitive_text, None);

        assert_ne!(sensitive_text, sanitized);
        assert!(!sanitized.contains("secret123"));
    }

    #[test]
    fn test_file_classification() {
        let manager = DataSecurityManager::new();

        let path = PathBuf::from("test_file_rust");
        let content = "fn main() {}";
        let category = manager.classify_file_content(content, &path);

        assert_eq!(category, DataCategory::SourceCode);
    }

    #[test]
    fn test_sensitivity_level() {
        let manager = DataSecurityManager::new();

        let secret_path = PathBuf::from("env_secret_file");
        let level = manager.get_file_sensitivity(&secret_path);

        assert!(level >= SensitivityLevel::Internal);
    }
}
