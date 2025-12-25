// O-Sovereign Core Module

pub mod addressing_system;
pub mod aegis;
pub mod agent_extension;
pub mod agent_state;
pub mod aipc_controller;
pub mod api_manager;
pub mod audit_log;
pub mod auth_system;
pub mod auto_takeover;
pub mod behavior_monitor;
pub mod cache_manager;
pub mod claude;
pub mod cognitive_cleaner;
pub mod concurrency;
pub mod config_manager;
pub mod data_security;
pub mod database;
pub mod distributed;
pub mod deepseek;
pub mod emergency_log;
pub mod event_bus;
pub mod error;
pub mod gemini;
pub mod http_server;
pub mod i18n;
pub mod image_generator;
pub mod jarvis;
pub mod lsp_server;
pub mod mcp_server;
pub mod metrics;
pub mod multimodal;
pub mod opencode;
pub mod opencode_connector;
pub mod performance;
pub mod personal_rules;
pub mod plugin_system;
pub mod prompt_manager;
pub mod protocol;
pub mod providers;
pub mod rag_engine;
pub mod rate_limiter;
pub mod router;
pub mod shadow_mode;
pub mod sovereignty;
pub mod sosa_api_pool;
pub mod sosa_crypto;
pub mod sosa_learning;
pub mod task_tracker;
pub mod terminal_server;
pub mod types;
pub mod voice_processor;
pub mod workflow_engine;

pub use addressing_system::{AddressingConfig, AddressingMode, AddressingStyle, AddressingSystem};
pub use aegis::{AegisModule, DefenseDocType, DefenseDocument};
pub use agent_extension::{
    AgentApiConfig, AgentCallRecord, AgentExtensionManager, AgentInfo, AgentList, AgentMetrics,
    AgentType, CustomAgent, DiminishingReturns, Recommendation,
};
pub use agent_state::{AgentStateConfig, AgentStateManager, LongTermMemory, Message, SessionState, StateSnapshot, UserPreference};
pub use aipc_controller::{AipcController, HardwareCommand, HardwareStatus, HardwareType};
pub use api_manager::{ApiCallRecord, ApiKeyConfig, ApiManager, ApiProvider, ProviderStats};
pub use audit_log::{AuditEvent, AuditEventType, AuditLogger, AuditLogConfig, AuditQuery, AuditSeverity, ComplianceReport};
pub use auth_system::{AuthConfig, AuthManager, Claims, SessionInfo, TokenPair};
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
pub use cognitive_cleaner::{ChunkTag, CleanedIntent, CognitiveCleaner, DictionaryData, DictionaryFormat, SemanticChunk};
pub use concurrency::{AsyncTask, ConcurrencyConfig, ConcurrencyManager, DistributedLock as ConcurrentLock, TaskPriority as ConcurrentTaskPriority, TaskResult};
pub use config_manager::{ConfigChange, ConfigEntry, ConfigListener, ConfigManager, ConfigManagerConfig, ConfigValue, Environment};
pub use database::{DatabaseConfig, DatabaseManager, DatabaseTransaction, DatabaseType, PoolStats, QueryBuilder, QueryRow};
pub use distributed::{ClusterManager, ClusterStats, DistributedLock as RedisLock, LockConfig, NodeRole, NodeStatus, ServiceDiscovery, ServiceDiscoveryConfig, ServiceInstance};
pub use deepseek::DeepSeekProvider;
pub use emergency_log::{EmergencyLogConfig, EmergencyLogger, LogEntry, LogEntryType};
pub use event_bus::{Event, EventBus, EventBusConfig, EventHandler, EventType, LoggingEventHandler, MetricsEventHandler};
pub use error::{AcsaError, AcsaResult, ErrorCode, ErrorSeverity};
pub use gemini::GeminiProvider;
pub use http_server::{ApiResponse, HttpServer, HttpServerConfig, ServerState};
pub use i18n::{I18n, Language, TranslationKey};
pub use image_generator::{GenerationConfig, ImageGenerator};
pub use jarvis::{DangerousOp, JarvisCircuitBreaker, JarvisVerdict};
pub use lsp_server::{AcsaLspServer, CompletionItem, CompletionItemKind, Diagnostic as LspDiagnostic, DiagnosticSeverity, Document as LspDocument, LspServerConfig, Position, Range};
pub use mcp_server::{
    AcsaMcpServer, ClientInfo, McpPrompt, McpRequest, McpResource, McpResponse, McpTool,
    McpToolHandler, create_acsa_mcp_server,
};
pub use metrics::{ApplicationMetrics, ComponentHealth, HealthCheck, HealthStatus, MetricType, MetricValue, MetricsCollector, SystemMetrics};
pub use multimodal::{ModalityType, MultimodalInput, MultimodalMetadata, MultimodalProcessor};
pub use opencode::OpenCodeExecutor;
pub use opencode_connector::{
    CodeStats, ExecutionReceipt, MissionPack, OpenCodeConfig, OpenCodeConnector, TestResults,
};
pub use performance::{CacheWarmer, EventBatcher, GLOBAL_OPTIMIZER, GLOBAL_SCHEDULER, PerformanceOptimizer, PhaseTracker, PriorityScheduler, StartupMetrics};
pub use personal_rules::{PersonalRule, PersonalRulesManager, RuleConflict, RuleType, RulesStats};
pub use plugin_system::{Plugin, PluginConfig, PluginHandler, PluginMetadata, PluginRequest, PluginResponse, PluginState, PluginStats, PluginSystem, PluginSystemConfig, PluginType, ResourceLimits};
pub use prompt_manager::{AbTestGroup, AbTestMetrics, FewShotExample, PromptBuildOptions, PromptManager, PromptManagerConfig, PromptTemplate};
pub use protocol::{AgentWeights, Protocol, ProtocolConfig, ProtocolManager};
pub use providers::{create_provider, ModelProvider};
pub use rag_engine::{ChunkingStrategy, Document as RagDocument, DocumentChunk, EmbeddingModel, RagConfig, RagEngine, RagStats, RetrievalMode, RetrievalResult};
pub use rate_limiter::{RateLimitLevel, RateLimitRecord, RateLimitResult, RateLimitRule, RateLimitStrategy, RateLimiter, RateLimiterConfig};
pub use router::ACSARouter;
pub use shadow_mode::{AccessAudit, MaskingConfig, MaskingStrategy, MaskedData, PiiDetection, PiiType, ShadowModeConfig, ShadowModeEngine};
pub use sovereignty::{
    BioActivity, CircuitBreakerConfig, DecisionEvent, DecisionType, DoseMeter, DoseStats,
    ExecCircuitBreaker, RiskLevel, SovereigntyConfig, SovereigntySystem, SOVEREIGNTY,
    generate_bio_activity_report,
};
pub use sosa_api_pool::{
    ApiCallEvent, ApiEndpoint, ApiErrorType, ApiProviderType, Attractor, BinaryTwin,
    EndpointStatus, LocalModelConfig, PoolConfig, SosaApiPool, SosaCore, SparseMarkov,
};
pub use sosa_crypto::{CryptoAlgorithm, CryptoKey, CryptoStats, EncryptedData, KeyPurpose, SosaCryptoConfig, SosaCryptoEngine};
pub use sosa_learning::{
    EventOutcome, KnowledgeNode, LearningConfig, LearningEvent, LearningSummary, SosaLearningEngine,
};
pub use task_tracker::{Task, TaskPriority, TaskStatus, TaskTracker};
pub use terminal_server::{ClientConnection, DefaultHandler, MessageHandler, ServerConfig, TerminalServer, WsMessage};
pub use types::*;
pub use voice_processor::{SttResult, VoiceConfig, VoiceProcessor};
pub use workflow_engine::{Workflow, WorkflowEngine, WorkflowStep};
