// Internationalization Module
// 多语言支持系统 (中英日韩 + 模块化语言包)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// 支持的语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// 简体中文
    #[serde(rename = "zh-CN")]
    ChineseSimplified,
    /// 英语（美国）
    #[serde(rename = "en-US")]
    EnglishUS,
    /// 日语
    #[serde(rename = "ja-JP")]
    Japanese,
    /// 韩语
    #[serde(rename = "ko-KR")]
    Korean,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::ChineseSimplified => "zh-CN",
            Language::EnglishUS => "en-US",
            Language::Japanese => "ja-JP",
            Language::Korean => "ko-KR",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Language::ChineseSimplified => "简体中文",
            Language::EnglishUS => "English",
            Language::Japanese => "日本語",
            Language::Korean => "한국어",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "zh-CN" | "zh" | "chinese" => Some(Language::ChineseSimplified),
            "en-US" | "en" | "english" => Some(Language::EnglishUS),
            "ja-JP" | "ja" | "japanese" => Some(Language::Japanese),
            "ko-KR" | "ko" | "korean" => Some(Language::Korean),
            _ => None,
        }
    }
}

/// 翻译键（用于类型安全）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TranslationKey {
    // === UI通用 ===
    AppTitle,
    AppDescription,
    Execute,
    Cancel,
    Settings,
    Language,

    // === Agent角色 ===
    AgentMoss,
    AgentL6,
    AgentUltron,
    AgentOmega,
    AgentJarvis,

    // === Agent描述 ===
    MossDescription,
    L6Description,
    UltronDescription,
    OmegaDescription,
    JarvisDescription,

    // === 执行状态 ===
    StatusPending,
    StatusRunning,
    StatusSuccess,
    StatusFailed,
    StatusBlocked,

    // === Jarvis消息 ===
    JarvisBlocked,
    JarvisWarning,
    JarvisPassed,
    JarvisCheckingInput,
    JarvisCheckingPlan,

    // === 错误消息 ===
    ErrorInvalidInput,
    ErrorNetworkFailure,
    ErrorApiKeyMissing,
    ErrorTimeout,
    ErrorUnknown,

    // === 统计信息 ===
    StatsTokensUsed,
    StatsCostTotal,
    StatsIterations,
    StatsRiskScore,

    // === API管理 ===
    ApiKeyAdd,
    ApiKeyDelete,
    ApiKeyEdit,
    ApiProviderOpenAI,
    ApiProviderClaude,
    ApiProviderGemini,
    ApiProviderDeepSeek,

    // === 自定义键 (用户可添加) ===
    Custom(String),
}

impl TranslationKey {
    pub fn as_str(&self) -> &str {
        match self {
            TranslationKey::AppTitle => "app.title",
            TranslationKey::AppDescription => "app.description",
            TranslationKey::Execute => "ui.execute",
            TranslationKey::Cancel => "ui.cancel",
            TranslationKey::Settings => "ui.settings",
            TranslationKey::Language => "ui.language",

            TranslationKey::AgentMoss => "agent.moss",
            TranslationKey::AgentL6 => "agent.l6",
            TranslationKey::AgentUltron => "agent.ultron",
            TranslationKey::AgentOmega => "agent.omega",
            TranslationKey::AgentJarvis => "agent.jarvis",

            TranslationKey::MossDescription => "agent.moss.desc",
            TranslationKey::L6Description => "agent.l6.desc",
            TranslationKey::UltronDescription => "agent.ultron.desc",
            TranslationKey::OmegaDescription => "agent.omega.desc",
            TranslationKey::JarvisDescription => "agent.jarvis.desc",

            TranslationKey::StatusPending => "status.pending",
            TranslationKey::StatusRunning => "status.running",
            TranslationKey::StatusSuccess => "status.success",
            TranslationKey::StatusFailed => "status.failed",
            TranslationKey::StatusBlocked => "status.blocked",

            TranslationKey::JarvisBlocked => "jarvis.blocked",
            TranslationKey::JarvisWarning => "jarvis.warning",
            TranslationKey::JarvisPassed => "jarvis.passed",
            TranslationKey::JarvisCheckingInput => "jarvis.checking_input",
            TranslationKey::JarvisCheckingPlan => "jarvis.checking_plan",

            TranslationKey::ErrorInvalidInput => "error.invalid_input",
            TranslationKey::ErrorNetworkFailure => "error.network_failure",
            TranslationKey::ErrorApiKeyMissing => "error.api_key_missing",
            TranslationKey::ErrorTimeout => "error.timeout",
            TranslationKey::ErrorUnknown => "error.unknown",

            TranslationKey::StatsTokensUsed => "stats.tokens_used",
            TranslationKey::StatsCostTotal => "stats.cost_total",
            TranslationKey::StatsIterations => "stats.iterations",
            TranslationKey::StatsRiskScore => "stats.risk_score",

            TranslationKey::ApiKeyAdd => "api.key.add",
            TranslationKey::ApiKeyDelete => "api.key.delete",
            TranslationKey::ApiKeyEdit => "api.key.edit",
            TranslationKey::ApiProviderOpenAI => "api.provider.openai",
            TranslationKey::ApiProviderClaude => "api.provider.claude",
            TranslationKey::ApiProviderGemini => "api.provider.gemini",
            TranslationKey::ApiProviderDeepSeek => "api.provider.deepseek",

            TranslationKey::Custom(key) => key.as_str(),
        }
    }
}

/// 多语言管理器
pub struct I18n {
    /// 当前语言
    current_language: Language,
    /// 翻译映射表 (语言 -> 键 -> 文本)
    translations: HashMap<Language, HashMap<String, String>>,
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(Language::ChineseSimplified)
    }
}

impl I18n {
    pub fn new(default_language: Language) -> Self {
        let mut i18n = Self {
            current_language: default_language,
            translations: HashMap::new(),
        };

        // 初始化内置语言包
        i18n.load_builtin_languages();

        info!("🌐 I18n initialized with language: {}", default_language.name());
        i18n
    }

    /// 加载所有内置语言包
    fn load_builtin_languages(&mut self) {
        self.load_chinese_simplified();
        self.load_english_us();
        self.load_japanese();
        self.load_korean();
    }

    /// 加载简体中文语言包
    fn load_chinese_simplified(&mut self) {
        let mut zh = HashMap::new();

        // UI通用
        zh.insert("app.title".to_string(), "O-Sovereign ACSA".to_string());
        zh.insert("app.description".to_string(), "对抗约束型盲从代理系统".to_string());
        zh.insert("ui.execute".to_string(), "执行".to_string());
        zh.insert("ui.cancel".to_string(), "取消".to_string());
        zh.insert("ui.settings".to_string(), "设置".to_string());
        zh.insert("ui.language".to_string(), "语言".to_string());

        // Agent角色
        zh.insert("agent.moss".to_string(), "MOSS - 战略规划师".to_string());
        zh.insert("agent.l6".to_string(), "L6 - 真理校验器".to_string());
        zh.insert("agent.ultron".to_string(), "Ultron - 红队审计师".to_string());
        zh.insert("agent.omega".to_string(), "Omega - 绝对执行层".to_string());
        zh.insert("agent.jarvis".to_string(), "Jarvis - 安全熔断器".to_string());

        // Agent描述
        zh.insert("agent.moss.desc".to_string(), "顶级战略咨询顾问，冷静功利".to_string());
        zh.insert("agent.l6.desc".to_string(), "物理引擎验证器，无情绪无偏见".to_string());
        zh.insert("agent.ultron.desc".to_string(), "30年红队审计师+刑辩律师".to_string());
        zh.insert("agent.omega.desc".to_string(), "绝对服从的执行层".to_string());
        zh.insert("agent.jarvis.desc".to_string(), "不可绕过的最高安全层".to_string());

        // 执行状态
        zh.insert("status.pending".to_string(), "等待中".to_string());
        zh.insert("status.running".to_string(), "执行中".to_string());
        zh.insert("status.success".to_string(), "成功".to_string());
        zh.insert("status.failed".to_string(), "失败".to_string());
        zh.insert("status.blocked".to_string(), "已阻止".to_string());

        // Jarvis消息
        zh.insert("jarvis.blocked".to_string(), "🚨 Jarvis已阻止此操作".to_string());
        zh.insert("jarvis.warning".to_string(), "⚠️ Jarvis发出警告".to_string());
        zh.insert("jarvis.passed".to_string(), "✅ Jarvis安全检查通过".to_string());
        zh.insert("jarvis.checking_input".to_string(), "Jarvis正在检查用户输入...".to_string());
        zh.insert("jarvis.checking_plan".to_string(), "Jarvis正在验证MOSS计划...".to_string());

        // 错误消息
        zh.insert("error.invalid_input".to_string(), "输入无效".to_string());
        zh.insert("error.network_failure".to_string(), "网络连接失败".to_string());
        zh.insert("error.api_key_missing".to_string(), "API密钥未配置".to_string());
        zh.insert("error.timeout".to_string(), "请求超时".to_string());
        zh.insert("error.unknown".to_string(), "未知错误".to_string());

        // 统计信息
        zh.insert("stats.tokens_used".to_string(), "使用Token数".to_string());
        zh.insert("stats.cost_total".to_string(), "总花费".to_string());
        zh.insert("stats.iterations".to_string(), "迭代次数".to_string());
        zh.insert("stats.risk_score".to_string(), "风险评分".to_string());

        // API管理
        zh.insert("api.key.add".to_string(), "添加API密钥".to_string());
        zh.insert("api.key.delete".to_string(), "删除API密钥".to_string());
        zh.insert("api.key.edit".to_string(), "编辑API密钥".to_string());
        zh.insert("api.provider.openai".to_string(), "OpenAI GPT-4".to_string());
        zh.insert("api.provider.claude".to_string(), "Claude Opus".to_string());
        zh.insert("api.provider.gemini".to_string(), "Gemini Pro".to_string());
        zh.insert("api.provider.deepseek".to_string(), "DeepSeek Coder".to_string());

        self.translations.insert(Language::ChineseSimplified, zh);
    }

    /// 加载英文语言包
    fn load_english_us(&mut self) {
        let mut en = HashMap::new();

        // UI Common
        en.insert("app.title".to_string(), "O-Sovereign ACSA".to_string());
        en.insert("app.description".to_string(), "Adversarially-Constrained Sycophantic Agent".to_string());
        en.insert("ui.execute".to_string(), "Execute".to_string());
        en.insert("ui.cancel".to_string(), "Cancel".to_string());
        en.insert("ui.settings".to_string(), "Settings".to_string());
        en.insert("ui.language".to_string(), "Language".to_string());

        // Agent Roles
        en.insert("agent.moss".to_string(), "MOSS - Strategic Planner".to_string());
        en.insert("agent.l6".to_string(), "L6 - Truth Validator".to_string());
        en.insert("agent.ultron".to_string(), "Ultron - Red Team Auditor".to_string());
        en.insert("agent.omega".to_string(), "Omega - Execution Layer".to_string());
        en.insert("agent.jarvis".to_string(), "Jarvis - Safety Circuit Breaker".to_string());

        // Agent Descriptions
        en.insert("agent.moss.desc".to_string(), "Top-tier strategic consultant, cold and pragmatic".to_string());
        en.insert("agent.l6.desc".to_string(), "Physics engine validator, emotionless and unbiased".to_string());
        en.insert("agent.ultron.desc".to_string(), "30-year red team auditor + defense lawyer".to_string());
        en.insert("agent.omega.desc".to_string(), "Absolute obedience execution layer".to_string());
        en.insert("agent.jarvis.desc".to_string(), "Unbypassable highest security layer".to_string());

        // Execution Status
        en.insert("status.pending".to_string(), "Pending".to_string());
        en.insert("status.running".to_string(), "Running".to_string());
        en.insert("status.success".to_string(), "Success".to_string());
        en.insert("status.failed".to_string(), "Failed".to_string());
        en.insert("status.blocked".to_string(), "Blocked".to_string());

        // Jarvis Messages
        en.insert("jarvis.blocked".to_string(), "🚨 Jarvis BLOCKED this operation".to_string());
        en.insert("jarvis.warning".to_string(), "⚠️ Jarvis WARNING issued".to_string());
        en.insert("jarvis.passed".to_string(), "✅ Jarvis safety check PASSED".to_string());
        en.insert("jarvis.checking_input".to_string(), "Jarvis checking user input...".to_string());
        en.insert("jarvis.checking_plan".to_string(), "Jarvis verifying MOSS plan...".to_string());

        // Error Messages
        en.insert("error.invalid_input".to_string(), "Invalid input".to_string());
        en.insert("error.network_failure".to_string(), "Network connection failed".to_string());
        en.insert("error.api_key_missing".to_string(), "API key not configured".to_string());
        en.insert("error.timeout".to_string(), "Request timeout".to_string());
        en.insert("error.unknown".to_string(), "Unknown error".to_string());

        // Statistics
        en.insert("stats.tokens_used".to_string(), "Tokens Used".to_string());
        en.insert("stats.cost_total".to_string(), "Total Cost".to_string());
        en.insert("stats.iterations".to_string(), "Iterations".to_string());
        en.insert("stats.risk_score".to_string(), "Risk Score".to_string());

        // API Management
        en.insert("api.key.add".to_string(), "Add API Key".to_string());
        en.insert("api.key.delete".to_string(), "Delete API Key".to_string());
        en.insert("api.key.edit".to_string(), "Edit API Key".to_string());
        en.insert("api.provider.openai".to_string(), "OpenAI GPT-4".to_string());
        en.insert("api.provider.claude".to_string(), "Claude Opus".to_string());
        en.insert("api.provider.gemini".to_string(), "Gemini Pro".to_string());
        en.insert("api.provider.deepseek".to_string(), "DeepSeek Coder".to_string());

        self.translations.insert(Language::EnglishUS, en);
    }

    /// 加载日文语言包
    fn load_japanese(&mut self) {
        let mut ja = HashMap::new();

        // UI共通
        ja.insert("app.title".to_string(), "O-Sovereign ACSA".to_string());
        ja.insert("app.description".to_string(), "対抗制約型追従エージェントシステム".to_string());
        ja.insert("ui.execute".to_string(), "実行".to_string());
        ja.insert("ui.cancel".to_string(), "キャンセル".to_string());
        ja.insert("ui.settings".to_string(), "設定".to_string());
        ja.insert("ui.language".to_string(), "言語".to_string());

        // エージェント役割
        ja.insert("agent.moss".to_string(), "MOSS - 戦略プランナー".to_string());
        ja.insert("agent.l6".to_string(), "L6 - 真理検証器".to_string());
        ja.insert("agent.ultron".to_string(), "Ultron - レッドチーム監査人".to_string());
        ja.insert("agent.omega".to_string(), "Omega - 実行レイヤー".to_string());
        ja.insert("agent.jarvis".to_string(), "Jarvis - 安全遮断器".to_string());

        // エージェント説明
        ja.insert("agent.moss.desc".to_string(), "トップ戦略コンサルタント、冷静で実利的".to_string());
        ja.insert("agent.l6.desc".to_string(), "物理エンジン検証器、感情なし偏見なし".to_string());
        ja.insert("agent.ultron.desc".to_string(), "30年レッドチーム監査人+刑事弁護士".to_string());
        ja.insert("agent.omega.desc".to_string(), "絶対服従の実行層".to_string());
        ja.insert("agent.jarvis.desc".to_string(), "回避不可能な最高セキュリティ層".to_string());

        // 実行状態
        ja.insert("status.pending".to_string(), "待機中".to_string());
        ja.insert("status.running".to_string(), "実行中".to_string());
        ja.insert("status.success".to_string(), "成功".to_string());
        ja.insert("status.failed".to_string(), "失敗".to_string());
        ja.insert("status.blocked".to_string(), "ブロック済み".to_string());

        // Jarvisメッセージ
        ja.insert("jarvis.blocked".to_string(), "🚨 Jarvisがこの操作をブロックしました".to_string());
        ja.insert("jarvis.warning".to_string(), "⚠️ Jarvis警告発行".to_string());
        ja.insert("jarvis.passed".to_string(), "✅ Jarvis安全チェック通過".to_string());
        ja.insert("jarvis.checking_input".to_string(), "Jarvisがユーザー入力を確認中...".to_string());
        ja.insert("jarvis.checking_plan".to_string(), "JarvisがMOSS計画を検証中...".to_string());

        // エラーメッセージ
        ja.insert("error.invalid_input".to_string(), "無効な入力".to_string());
        ja.insert("error.network_failure".to_string(), "ネットワーク接続失敗".to_string());
        ja.insert("error.api_key_missing".to_string(), "APIキーが設定されていません".to_string());
        ja.insert("error.timeout".to_string(), "リクエストタイムアウト".to_string());
        ja.insert("error.unknown".to_string(), "不明なエラー".to_string());

        // 統計情報
        ja.insert("stats.tokens_used".to_string(), "使用トークン数".to_string());
        ja.insert("stats.cost_total".to_string(), "合計コスト".to_string());
        ja.insert("stats.iterations".to_string(), "反復回数".to_string());
        ja.insert("stats.risk_score".to_string(), "リスクスコア".to_string());

        // API管理
        ja.insert("api.key.add".to_string(), "APIキーを追加".to_string());
        ja.insert("api.key.delete".to_string(), "APIキーを削除".to_string());
        ja.insert("api.key.edit".to_string(), "APIキーを編集".to_string());
        ja.insert("api.provider.openai".to_string(), "OpenAI GPT-4".to_string());
        ja.insert("api.provider.claude".to_string(), "Claude Opus".to_string());
        ja.insert("api.provider.gemini".to_string(), "Gemini Pro".to_string());
        ja.insert("api.provider.deepseek".to_string(), "DeepSeek Coder".to_string());

        self.translations.insert(Language::Japanese, ja);
    }

    /// 加载韩文语言包
    fn load_korean(&mut self) {
        let mut ko = HashMap::new();

        // UI 공통
        ko.insert("app.title".to_string(), "O-Sovereign ACSA".to_string());
        ko.insert("app.description".to_string(), "대항 제약형 맹종 에이전트 시스템".to_string());
        ko.insert("ui.execute".to_string(), "실행".to_string());
        ko.insert("ui.cancel".to_string(), "취소".to_string());
        ko.insert("ui.settings".to_string(), "설정".to_string());
        ko.insert("ui.language".to_string(), "언어".to_string());

        // 에이전트 역할
        ko.insert("agent.moss".to_string(), "MOSS - 전략 기획자".to_string());
        ko.insert("agent.l6".to_string(), "L6 - 진실 검증기".to_string());
        ko.insert("agent.ultron".to_string(), "Ultron - 레드팀 감사관".to_string());
        ko.insert("agent.omega".to_string(), "Omega - 실행 레이어".to_string());
        ko.insert("agent.jarvis".to_string(), "Jarvis - 안전 차단기".to_string());

        // 에이전트 설명
        ko.insert("agent.moss.desc".to_string(), "최상급 전략 컨설턴트, 냉정하고 실용적".to_string());
        ko.insert("agent.l6.desc".to_string(), "물리 엔진 검증기, 감정없고 편견없음".to_string());
        ko.insert("agent.ultron.desc".to_string(), "30년 레드팀 감사관 + 형사 변호사".to_string());
        ko.insert("agent.omega.desc".to_string(), "절대 복종의 실행층".to_string());
        ko.insert("agent.jarvis.desc".to_string(), "우회 불가능한 최고 보안층".to_string());

        // 실행 상태
        ko.insert("status.pending".to_string(), "대기 중".to_string());
        ko.insert("status.running".to_string(), "실행 중".to_string());
        ko.insert("status.success".to_string(), "성공".to_string());
        ko.insert("status.failed".to_string(), "실패".to_string());
        ko.insert("status.blocked".to_string(), "차단됨".to_string());

        // Jarvis 메시지
        ko.insert("jarvis.blocked".to_string(), "🚨 Jarvis가 이 작업을 차단했습니다".to_string());
        ko.insert("jarvis.warning".to_string(), "⚠️ Jarvis 경고 발행".to_string());
        ko.insert("jarvis.passed".to_string(), "✅ Jarvis 안전 검사 통과".to_string());
        ko.insert("jarvis.checking_input".to_string(), "Jarvis가 사용자 입력 확인 중...".to_string());
        ko.insert("jarvis.checking_plan".to_string(), "Jarvis가 MOSS 계획 검증 중...".to_string());

        // 오류 메시지
        ko.insert("error.invalid_input".to_string(), "잘못된 입력".to_string());
        ko.insert("error.network_failure".to_string(), "네트워크 연결 실패".to_string());
        ko.insert("error.api_key_missing".to_string(), "API 키가 설정되지 않음".to_string());
        ko.insert("error.timeout".to_string(), "요청 시간 초과".to_string());
        ko.insert("error.unknown".to_string(), "알 수 없는 오류".to_string());

        // 통계 정보
        ko.insert("stats.tokens_used".to_string(), "사용된 토큰".to_string());
        ko.insert("stats.cost_total".to_string(), "총 비용".to_string());
        ko.insert("stats.iterations".to_string(), "반복 횟수".to_string());
        ko.insert("stats.risk_score".to_string(), "위험 점수".to_string());

        // API 관리
        ko.insert("api.key.add".to_string(), "API 키 추가".to_string());
        ko.insert("api.key.delete".to_string(), "API 키 삭제".to_string());
        ko.insert("api.key.edit".to_string(), "API 키 편집".to_string());
        ko.insert("api.provider.openai".to_string(), "OpenAI GPT-4".to_string());
        ko.insert("api.provider.claude".to_string(), "Claude Opus".to_string());
        ko.insert("api.provider.gemini".to_string(), "Gemini Pro".to_string());
        ko.insert("api.provider.deepseek".to_string(), "DeepSeek Coder".to_string());

        self.translations.insert(Language::Korean, ko);
    }

    /// 获取翻译文本
    pub fn t(&self, key: &TranslationKey) -> String {
        let key_str = key.as_str();

        // 尝试获取当前语言的翻译
        if let Some(lang_map) = self.translations.get(&self.current_language) {
            if let Some(text) = lang_map.get(key_str) {
                return text.clone();
            }
        }

        // 回退到英文
        if self.current_language != Language::EnglishUS {
            if let Some(lang_map) = self.translations.get(&Language::EnglishUS) {
                if let Some(text) = lang_map.get(key_str) {
                    return text.clone();
                }
            }
        }

        // 最后回退到键本身
        key_str.to_string()
    }

    /// 切换语言
    pub fn set_language(&mut self, language: Language) {
        self.current_language = language;
        info!("🌐 Language switched to: {}", language.name());
    }

    /// 获取当前语言
    pub fn current_language(&self) -> Language {
        self.current_language
    }

    /// 添加自定义翻译
    pub fn add_translation(&mut self, language: Language, key: &str, text: String) {
        self.translations
            .entry(language)
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), text);
    }

    /// 批量添加翻译（语言包）
    pub fn add_language_pack(&mut self, language: Language, pack: HashMap<String, String>) {
        self.translations
            .entry(language)
            .or_insert_with(HashMap::new)
            .extend(pack);
        info!("📦 Language pack loaded for: {}", language.name());
    }

    /// 检查语言是否已加载
    pub fn is_language_loaded(&self, language: Language) -> bool {
        self.translations.contains_key(&language)
    }

    /// 获取所有已加载的语言
    pub fn available_languages(&self) -> Vec<Language> {
        self.translations.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_code() {
        assert_eq!(Language::ChineseSimplified.code(), "zh-CN");
        assert_eq!(Language::EnglishUS.code(), "en-US");
        assert_eq!(Language::Japanese.code(), "ja-JP");
        assert_eq!(Language::Korean.code(), "ko-KR");
    }

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("zh-CN"), Some(Language::ChineseSimplified));
        assert_eq!(Language::from_code("en"), Some(Language::EnglishUS));
        assert_eq!(Language::from_code("ja"), Some(Language::Japanese));
        assert_eq!(Language::from_code("ko-KR"), Some(Language::Korean));
        assert_eq!(Language::from_code("invalid"), None);
    }

    #[test]
    fn test_translation_chinese() {
        let i18n = I18n::new(Language::ChineseSimplified);
        assert_eq!(i18n.t(&TranslationKey::AppTitle), "O-Sovereign ACSA");
        assert_eq!(i18n.t(&TranslationKey::Execute), "执行");
        assert_eq!(i18n.t(&TranslationKey::AgentMoss), "MOSS - 战略规划师");
    }

    #[test]
    fn test_translation_english() {
        let i18n = I18n::new(Language::EnglishUS);
        assert_eq!(i18n.t(&TranslationKey::AppDescription), "Adversarially-Constrained Sycophantic Agent");
        assert_eq!(i18n.t(&TranslationKey::Execute), "Execute");
        assert_eq!(i18n.t(&TranslationKey::StatusSuccess), "Success");
    }

    #[test]
    fn test_translation_japanese() {
        let i18n = I18n::new(Language::Japanese);
        assert_eq!(i18n.t(&TranslationKey::Execute), "実行");
        assert_eq!(i18n.t(&TranslationKey::Cancel), "キャンセル");
    }

    #[test]
    fn test_translation_korean() {
        let i18n = I18n::new(Language::Korean);
        assert_eq!(i18n.t(&TranslationKey::Execute), "실행");
        assert_eq!(i18n.t(&TranslationKey::Cancel), "취소");
    }

    #[test]
    fn test_language_switch() {
        let mut i18n = I18n::new(Language::ChineseSimplified);
        assert_eq!(i18n.t(&TranslationKey::Execute), "执行");

        i18n.set_language(Language::EnglishUS);
        assert_eq!(i18n.t(&TranslationKey::Execute), "Execute");

        i18n.set_language(Language::Japanese);
        assert_eq!(i18n.t(&TranslationKey::Execute), "実行");
    }

    #[test]
    fn test_custom_language_pack() {
        let mut i18n = I18n::new(Language::EnglishUS);

        let mut custom_pack = HashMap::new();
        custom_pack.insert("custom.hello".to_string(), "你好世界".to_string());

        i18n.add_language_pack(Language::ChineseSimplified, custom_pack);
        i18n.set_language(Language::ChineseSimplified);

        assert_eq!(i18n.t(&TranslationKey::Custom("custom.hello".to_string())), "你好世界");
    }

    #[test]
    fn test_available_languages() {
        let i18n = I18n::new(Language::ChineseSimplified);
        let langs = i18n.available_languages();

        assert_eq!(langs.len(), 4);
        assert!(langs.contains(&Language::ChineseSimplified));
        assert!(langs.contains(&Language::EnglishUS));
        assert!(langs.contains(&Language::Japanese));
        assert!(langs.contains(&Language::Korean));
    }
}
