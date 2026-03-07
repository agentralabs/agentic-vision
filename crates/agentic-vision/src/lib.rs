//! AgenticVision — core vision library for image capture, embedding, similarity, and visual memory.

#[cfg(feature = "cli")]
pub mod cli;

pub mod bridges;
pub mod cache;
pub mod capture;
pub mod contracts;
pub mod diff;
pub mod embedding;
pub mod metrics;
pub mod perception;
pub mod query;
pub mod similarity;
pub mod storage;
pub mod types;

pub use capture::{
    capture_clipboard, capture_from_base64, capture_from_file, capture_screenshot,
    generate_thumbnail,
};
pub use diff::compute_diff;
pub use embedding::{EmbeddingEngine, EMBEDDING_DIM};
pub use perception::{
    DomSnapshot, GrammarStatus, GrammarStore, IntentCache, PerceptionIntent, PerceptionLayer,
    PerceptionRouter, SiteGrammar, TokenBudget,
};
pub use similarity::{cosine_similarity, find_similar};
pub use storage::{AvisReader, AvisStoreV2, AvisWriter};
pub use types::*;
