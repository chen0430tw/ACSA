// O-Sovereign Core Module

pub mod aegis;
pub mod cognitive_cleaner;
pub mod deepseek;
pub mod jarvis;
pub mod multimodal;
pub mod opencode;
pub mod providers;
pub mod router;
pub mod types;

pub use aegis::{AegisModule, DefenseDocType, DefenseDocument};
pub use cognitive_cleaner::{ChunkTag, CleanedIntent, CognitiveCleaner, SemanticChunk};
pub use deepseek::DeepSeekProvider;
pub use jarvis::{DangerousOp, JarvisCircuitBreaker, JarvisVerdict};
pub use multimodal::{ModalityType, MultimodalInput, MultimodalMetadata, MultimodalProcessor};
pub use opencode::OpenCodeExecutor;
pub use providers::{create_provider, ModelProvider};
pub use router::ACSARouter;
pub use types::*;
