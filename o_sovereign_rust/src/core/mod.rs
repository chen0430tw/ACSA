// O-Sovereign Core Module

pub mod aegis;
pub mod api_manager;
pub mod cache_manager;
pub mod claude;
pub mod cognitive_cleaner;
pub mod deepseek;
pub mod error;
pub mod gemini;
pub mod i18n;
pub mod jarvis;
pub mod multimodal;
pub mod opencode;
pub mod opencode_connector;
pub mod protocol;
pub mod providers;
pub mod router;
pub mod sosa_api_pool;
pub mod task_tracker;
pub mod types;

pub use aegis::{AegisModule, DefenseDocType, DefenseDocument};
pub use api_manager::{ApiCallRecord, ApiKeyConfig, ApiManager, ApiProvider, ProviderStats};
pub use cache_manager::{CacheManager, CacheType, CacheUsage, CleanupPolicy, CleanupStats};
pub use claude::ClaudeProvider;
pub use cognitive_cleaner::{ChunkTag, CleanedIntent, CognitiveCleaner, SemanticChunk};
pub use deepseek::DeepSeekProvider;
pub use error::{AcsaError, AcsaResult, ErrorCode, ErrorSeverity};
pub use gemini::GeminiProvider;
pub use i18n::{I18n, Language, TranslationKey};
pub use jarvis::{DangerousOp, JarvisCircuitBreaker, JarvisVerdict};
pub use multimodal::{ModalityType, MultimodalInput, MultimodalMetadata, MultimodalProcessor};
pub use opencode::OpenCodeExecutor;
pub use opencode_connector::{
    CodeStats, ExecutionReceipt, MissionPack, OpenCodeConfig, OpenCodeConnector, TestResults,
};
pub use protocol::{AgentWeights, Protocol, ProtocolConfig, ProtocolManager};
pub use providers::{create_provider, ModelProvider};
pub use router::ACSARouter;
pub use sosa_api_pool::{
    ApiCallEvent, ApiEndpoint, ApiErrorType, ApiProviderType, Attractor, BinaryTwin,
    EndpointStatus, LocalModelConfig, PoolConfig, SosaApiPool, SosaCore, SparseMarkov,
};
pub use task_tracker::{Task, TaskPriority, TaskStatus, TaskTracker};
pub use types::*;
