// O-Sovereign Core Module

pub mod aegis;
pub mod cognitive_cleaner;
pub mod providers;
pub mod router;
pub mod types;

pub use aegis::{AegisModule, DefenseDocType, DefenseDocument};
pub use cognitive_cleaner::{ChunkTag, CleanedIntent, CognitiveCleaner, SemanticChunk};
pub use providers::{create_provider, ModelProvider};
pub use router::ACSARouter;
pub use types::*;
