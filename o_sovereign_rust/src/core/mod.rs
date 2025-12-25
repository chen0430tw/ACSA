// O-Sovereign Core Module

pub mod aegis;
pub mod agent_extension;
pub mod api_manager;
pub mod auto_takeover;
pub mod behavior_monitor;
pub mod cache_manager;
pub mod claude;
pub mod cognitive_cleaner;
pub mod data_security;
pub mod deepseek;
pub mod error;
pub mod gemini;
pub mod i18n;
pub mod jarvis;
pub mod mcp_server;
pub mod multimodal;
pub mod opencode;
pub mod opencode_connector;
pub mod personal_rules;
pub mod protocol;
pub mod providers;
pub mod router;
pub mod sosa_api_pool;
pub mod sosa_learning;
pub mod task_tracker;
pub mod types;

pub use aegis::{AegisModule, DefenseDocType, DefenseDocument};
pub use agent_extension::{
    AgentApiConfig, AgentCallRecord, AgentExtensionManager, AgentInfo, AgentList, AgentMetrics,
    AgentType, CustomAgent, DiminishingReturns, Recommendation,
};
pub use api_manager::{ApiCallRecord, ApiKeyConfig, ApiManager, ApiProvider, ProviderStats};
pub use auto_takeover::{AutoTakeoverEngine, TakeoverAction, TakeoverPolicy, TakeoverResult, TakeoverStats};
pub use behavior_monitor::{
    BehaviorContext, BehaviorMonitor, BehaviorMonitorConfig, BehaviorPattern, BehaviorProfile,
    BehaviorType, ChatIntent, TakeoverSuggestion, UserBehaviorEvent,
};
pub use cache_manager::{CacheManager, CacheType, CacheUsage, CleanupPolicy, CleanupStats};
pub use claude::ClaudeProvider;
pub use data_security::{
    DataCategory, DataSecurityManager, FileAccessPermission, ImageFormat, PermissionRequest,
    PermissionType, ResourceStats, ResourceUsage, SanitizationRule, SecureFileContent,
    SecureImageContent, SensitivityLevel, JARVIS_EXPLANATION,
};
pub use cognitive_cleaner::{ChunkTag, CleanedIntent, CognitiveCleaner, SemanticChunk};
pub use deepseek::DeepSeekProvider;
pub use error::{AcsaError, AcsaResult, ErrorCode, ErrorSeverity};
pub use gemini::GeminiProvider;
pub use i18n::{I18n, Language, TranslationKey};
pub use jarvis::{DangerousOp, JarvisCircuitBreaker, JarvisVerdict};
pub use mcp_server::{
    AcsaMcpServer, ClientInfo, McpPrompt, McpRequest, McpResource, McpResponse, McpTool,
    McpToolHandler, create_acsa_mcp_server,
};
pub use multimodal::{ModalityType, MultimodalInput, MultimodalMetadata, MultimodalProcessor};
pub use opencode::OpenCodeExecutor;
pub use opencode_connector::{
    CodeStats, ExecutionReceipt, MissionPack, OpenCodeConfig, OpenCodeConnector, TestResults,
};
pub use personal_rules::{PersonalRule, PersonalRulesManager, RuleConflict, RuleType, RulesStats};
pub use protocol::{AgentWeights, Protocol, ProtocolConfig, ProtocolManager};
pub use providers::{create_provider, ModelProvider};
pub use router::ACSARouter;
pub use sosa_api_pool::{
    ApiCallEvent, ApiEndpoint, ApiErrorType, ApiProviderType, Attractor, BinaryTwin,
    EndpointStatus, LocalModelConfig, PoolConfig, SosaApiPool, SosaCore, SparseMarkov,
};
pub use sosa_learning::{
    EventOutcome, KnowledgeNode, LearningConfig, LearningEvent, LearningSummary, SosaLearningEngine,
};
pub use task_tracker::{Task, TaskPriority, TaskStatus, TaskTracker};
pub use types::*;
