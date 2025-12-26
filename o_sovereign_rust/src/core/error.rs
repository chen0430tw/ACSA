// Error Handling System with Error Codes
// 容错日志系统 - 使用错误编号避免乱码

use std::fmt;
use thiserror::Error;

/// ACSA错误代码
/// 格式: [模块代码][错误类型][序号]
/// 例如: E1001 = Router模块(1) - 初始化错误(00) - 序号1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // === Router模块 (1xxx) ===
    /// E1001: Router初始化失败
    RouterInitFailed = 1001,
    /// E1002: Jarvis验证失败
    RouterJarvisBlocked = 1002,
    /// E1003: 超过最大迭代次数
    RouterMaxIterations = 1003,
    /// E1004: 执行超时
    RouterTimeout = 1004,

    // === Provider模块 (2xxx) ===
    /// E2001: API密钥缺失
    ProviderApiKeyMissing = 2001,
    /// E2002: API调用失败
    ProviderApiCallFailed = 2002,
    /// E2003: 响应解析失败
    ProviderResponseParseFailed = 2003,
    /// E2004: 网络连接失败
    ProviderNetworkError = 2004,
    /// E2005: 速率限制
    ProviderRateLimited = 2005,

    // === OpenCode模块 (3xxx) ===
    /// E3001: OpenCode未安装
    OpenCodeNotInstalled = 3001,
    /// E3002: 任务执行失败
    OpenCodeExecutionFailed = 3002,
    /// E3003: 文件操作失败
    OpenCodeFileOpFailed = 3003,
    /// E3004: 超时
    OpenCodeTimeout = 3004,

    // === Jarvis模块 (4xxx) ===
    /// E4001: 检测到危险操作
    JarvisDangerousOp = 4001,
    /// E4002: 硬编码黑名单命中
    JarvisBlacklistHit = 4002,
    /// E4003: 风险等级过高
    JarvisHighRisk = 4003,

    // === API管理模块 (5xxx) ===
    /// E5001: API密钥不存在
    ApiKeyNotFound = 5001,
    /// E5002: API密钥无效
    ApiKeyInvalid = 5002,
    /// E5003: 数据持久化失败
    ApiPersistenceFailed = 5003,

    // === i18n模块 (6xxx) ===
    /// E6001: 语言包加载失败
    I18nLoadFailed = 6001,
    /// E6002: 翻译键不存在
    I18nKeyNotFound = 6002,

    // === 通用错误 (9xxx) ===
    /// E9001: 未知错误
    Unknown = 9001,
    /// E9002: 配置错误
    ConfigError = 9002,
    /// E9003: IO错误
    IoError = 9003,
    /// E9004: JSON序列化错误
    JsonError = 9004,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl ErrorCode {
    /// 获取错误代码字符串
    pub fn code(&self) -> String {
        format!("E{:04}", *self as u32)
    }

    /// 获取错误简短描述（英文）
    pub fn description_en(&self) -> &'static str {
        match self {
            // Router
            ErrorCode::RouterInitFailed => "Router initialization failed",
            ErrorCode::RouterJarvisBlocked => "Blocked by Jarvis safety check",
            ErrorCode::RouterMaxIterations => "Maximum iterations exceeded",
            ErrorCode::RouterTimeout => "Execution timeout",

            // Provider
            ErrorCode::ProviderApiKeyMissing => "API key missing",
            ErrorCode::ProviderApiCallFailed => "API call failed",
            ErrorCode::ProviderResponseParseFailed => "Failed to parse API response",
            ErrorCode::ProviderNetworkError => "Network connection error",
            ErrorCode::ProviderRateLimited => "Rate limit exceeded",

            // OpenCode
            ErrorCode::OpenCodeNotInstalled => "OpenCode not installed",
            ErrorCode::OpenCodeExecutionFailed => "Task execution failed",
            ErrorCode::OpenCodeFileOpFailed => "File operation failed",
            ErrorCode::OpenCodeTimeout => "OpenCode execution timeout",

            // Jarvis
            ErrorCode::JarvisDangerousOp => "Dangerous operation detected",
            ErrorCode::JarvisBlacklistHit => "Blacklist keyword detected",
            ErrorCode::JarvisHighRisk => "Risk level too high",

            // API Manager
            ErrorCode::ApiKeyNotFound => "API key not found",
            ErrorCode::ApiKeyInvalid => "API key invalid",
            ErrorCode::ApiPersistenceFailed => "Failed to persist data",

            // i18n
            ErrorCode::I18nLoadFailed => "Failed to load language pack",
            ErrorCode::I18nKeyNotFound => "Translation key not found",

            // General
            ErrorCode::Unknown => "Unknown error",
            ErrorCode::ConfigError => "Configuration error",
            ErrorCode::IoError => "I/O error",
            ErrorCode::JsonError => "JSON serialization error",
        }
    }

    /// 获取错误简短描述（中文）
    pub fn description_zh(&self) -> &'static str {
        match self {
            // Router
            ErrorCode::RouterInitFailed => "路由器初始化失败",
            ErrorCode::RouterJarvisBlocked => "被Jarvis安全检查阻止",
            ErrorCode::RouterMaxIterations => "超过最大迭代次数",
            ErrorCode::RouterTimeout => "执行超时",

            // Provider
            ErrorCode::ProviderApiKeyMissing => "API密钥缺失",
            ErrorCode::ProviderApiCallFailed => "API调用失败",
            ErrorCode::ProviderResponseParseFailed => "解析API响应失败",
            ErrorCode::ProviderNetworkError => "网络连接错误",
            ErrorCode::ProviderRateLimited => "超过速率限制",

            // OpenCode
            ErrorCode::OpenCodeNotInstalled => "OpenCode未安装",
            ErrorCode::OpenCodeExecutionFailed => "任务执行失败",
            ErrorCode::OpenCodeFileOpFailed => "文件操作失败",
            ErrorCode::OpenCodeTimeout => "OpenCode执行超时",

            // Jarvis
            ErrorCode::JarvisDangerousOp => "检测到危险操作",
            ErrorCode::JarvisBlacklistHit => "命中黑名单关键词",
            ErrorCode::JarvisHighRisk => "风险等级过高",

            // API Manager
            ErrorCode::ApiKeyNotFound => "API密钥不存在",
            ErrorCode::ApiKeyInvalid => "API密钥无效",
            ErrorCode::ApiPersistenceFailed => "数据持久化失败",

            // i18n
            ErrorCode::I18nLoadFailed => "语言包加载失败",
            ErrorCode::I18nKeyNotFound => "翻译键不存在",

            // General
            ErrorCode::Unknown => "未知错误",
            ErrorCode::ConfigError => "配置错误",
            ErrorCode::IoError => "输入输出错误",
            ErrorCode::JsonError => "JSON序列化错误",
        }
    }

    /// 获取推荐的错误级别
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // 致命错误
            ErrorCode::RouterInitFailed
            | ErrorCode::OpenCodeNotInstalled
            | ErrorCode::ProviderApiKeyMissing => ErrorSeverity::Fatal,

            // 高危错误
            ErrorCode::JarvisDangerousOp
            | ErrorCode::JarvisBlacklistHit
            | ErrorCode::JarvisHighRisk
            | ErrorCode::RouterJarvisBlocked => ErrorSeverity::Critical,

            // 可恢复错误
            ErrorCode::ProviderApiCallFailed
            | ErrorCode::ProviderNetworkError
            | ErrorCode::OpenCodeExecutionFailed
            | ErrorCode::RouterMaxIterations => ErrorSeverity::Error,

            // 警告
            ErrorCode::ProviderRateLimited
            | ErrorCode::RouterTimeout
            | ErrorCode::OpenCodeTimeout => ErrorSeverity::Warning,

            // 其他
            _ => ErrorSeverity::Error,
        }
    }
}

/// 错误严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// 致命 - 系统无法继续运行
    Fatal,
    /// 关键 - 安全问题或重大功能失效
    Critical,
    /// 错误 - 操作失败但系统可继续
    Error,
    /// 警告 - 潜在问题
    Warning,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorSeverity::Fatal => write!(f, "FATAL"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
        }
    }
}

/// ACSA错误类型
#[derive(Error, Debug)]
pub enum AcsaError {
    #[error("[{code}] {severity}: {message}")]
    Coded {
        code: ErrorCode,
        severity: ErrorSeverity,
        message: String,
        context: Option<String>,
    },

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl AcsaError {
    /// 创建带错误代码的错误
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        let severity = code.severity();
        AcsaError::Coded {
            code,
            severity,
            message: message.into(),
            context: None,
        }
    }

    /// 创建带上下文的错误
    pub fn with_context(code: ErrorCode, message: impl Into<String>, context: impl Into<String>) -> Self {
        let severity = code.severity();
        AcsaError::Coded {
            code,
            severity,
            message: message.into(),
            context: Some(context.into()),
        }
    }

    /// 获取错误代码
    pub fn code(&self) -> Option<ErrorCode> {
        match self {
            AcsaError::Coded { code, .. } => Some(*code),
            _ => None,
        }
    }

    /// 获取错误级别
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AcsaError::Coded { severity, .. } => *severity,
            _ => ErrorSeverity::Error,
        }
    }

    /// 格式化为用户友好的错误消息
    pub fn user_message(&self, lang: &str) -> String {
        match self {
            AcsaError::Coded { code, message, context, .. } => {
                let desc = match lang {
                    "zh" | "zh-CN" => code.description_zh(),
                    _ => code.description_en(),
                };

                let mut msg = format!("[{}] {}: {}", code.code(), desc, message);

                if let Some(ctx) = context {
                    msg.push_str(&format!("\n  Context: {}", ctx));
                }

                msg
            }
            AcsaError::Anyhow(e) => format!("Error: {}", e),
            AcsaError::Io(e) => format!("[E9003] I/O Error: {}", e),
            AcsaError::Json(e) => format!("[E9004] JSON Error: {}", e),
        }
    }

    /// 日志输出（带emoji和颜色提示）
    pub fn log(&self) {
        let severity = self.severity();
        let emoji = match severity {
            ErrorSeverity::Fatal => "💀",
            ErrorSeverity::Critical => "🚨",
            ErrorSeverity::Error => "❌",
            ErrorSeverity::Warning => "⚠️",
        };

        let msg = self.user_message("en");

        match severity {
            ErrorSeverity::Fatal | ErrorSeverity::Critical => {
                tracing::error!("{} {}", emoji, msg);
            }
            ErrorSeverity::Error => {
                tracing::error!("{} {}", emoji, msg);
            }
            ErrorSeverity::Warning => {
                tracing::warn!("{} {}", emoji, msg);
            }
        }
    }
}

/// Result类型别名
pub type AcsaResult<T> = Result<T, AcsaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_formatting() {
        assert_eq!(ErrorCode::RouterInitFailed.code(), "E1001");
        assert_eq!(ErrorCode::ProviderApiKeyMissing.code(), "E2001");
        assert_eq!(ErrorCode::JarvisDangerousOp.code(), "E4001");
    }

    #[test]
    fn test_error_descriptions() {
        let code = ErrorCode::ProviderApiCallFailed;
        assert_eq!(code.description_en(), "API call failed");
        assert_eq!(code.description_zh(), "API调用失败");
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(ErrorCode::RouterInitFailed.severity(), ErrorSeverity::Fatal);
        assert_eq!(ErrorCode::JarvisDangerousOp.severity(), ErrorSeverity::Critical);
        assert_eq!(ErrorCode::ProviderApiCallFailed.severity(), ErrorSeverity::Error);
        assert_eq!(ErrorCode::ProviderRateLimited.severity(), ErrorSeverity::Warning);
    }

    #[test]
    fn test_acsa_error_creation() {
        let err = AcsaError::new(
            ErrorCode::RouterJarvisBlocked,
            "Dangerous operation detected: rm -rf /"
        );

        assert_eq!(err.code(), Some(ErrorCode::RouterJarvisBlocked));
        assert_eq!(err.severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_user_message_formatting() {
        let err = AcsaError::with_context(
            ErrorCode::ProviderApiCallFailed,
            "OpenAI API returned 429",
            "Rate limit: 10 req/min",
        );

        let msg_en = err.user_message("en");
        assert!(msg_en.contains("[E2002]"));
        assert!(msg_en.contains("API call failed"));
        assert!(msg_en.contains("Context:"));

        let msg_zh = err.user_message("zh");
        assert!(msg_zh.contains("[E2002]"));
        assert!(msg_zh.contains("API调用失败"));
    }

    #[test]
    fn test_error_code_uniqueness() {
        use std::collections::HashSet;

        let codes = vec![
            ErrorCode::RouterInitFailed,
            ErrorCode::RouterJarvisBlocked,
            ErrorCode::ProviderApiKeyMissing,
            ErrorCode::JarvisDangerousOp,
            ErrorCode::OpenCodeNotInstalled,
        ];

        let unique_codes: HashSet<_> = codes.iter().map(|c| *c as u32).collect();
        assert_eq!(unique_codes.len(), codes.len(), "Error codes must be unique");
    }
}
