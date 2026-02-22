//! Core data types for visual observations and memory.

use serde::{Deserialize, Serialize};

fn default_quality_score() -> f32 {
    0.0
}

/// A captured visual observation stored in visual memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualObservation {
    pub id: u64,
    pub timestamp: u64,
    pub session_id: u32,
    pub source: CaptureSource,
    pub embedding: Vec<f32>,
    pub thumbnail: Vec<u8>,
    pub metadata: ObservationMeta,
    pub memory_link: Option<u64>,
}

/// How the image was captured.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CaptureSource {
    File { path: String },
    Base64 { mime: String },
    Screenshot { region: Option<Rect> },
    Clipboard,
}

/// Metadata about a visual observation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationMeta {
    pub width: u32,
    pub height: u32,
    pub original_width: u32,
    pub original_height: u32,
    pub labels: Vec<String>,
    pub description: Option<String>,
    /// Signal quality score in [0.0, 1.0] (resolution, metadata richness, model quality).
    #[serde(default = "default_quality_score")]
    pub quality_score: f32,
}

/// Pixel-level diff between two captures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualDiff {
    pub before_id: u64,
    pub after_id: u64,
    pub similarity: f32,
    pub changed_regions: Vec<Rect>,
    pub pixel_diff_ratio: f32,
}

/// A rectangle region.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// A similarity match result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityMatch {
    pub id: u64,
    pub similarity: f32,
}

/// In-memory container for all visual observations.
#[derive(Debug, Clone)]
pub struct VisualMemoryStore {
    pub observations: Vec<VisualObservation>,
    pub embedding_dim: u32,
    pub next_id: u64,
    pub session_count: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

impl VisualMemoryStore {
    /// Create a new empty store.
    pub fn new(embedding_dim: u32) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            observations: Vec::new(),
            embedding_dim,
            next_id: 1,
            session_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get an observation by ID.
    pub fn get(&self, id: u64) -> Option<&VisualObservation> {
        self.observations.iter().find(|o| o.id == id)
    }

    /// Get a mutable observation by ID.
    pub fn get_mut(&mut self, id: u64) -> Option<&mut VisualObservation> {
        self.observations.iter_mut().find(|o| o.id == id)
    }

    /// Add an observation and return its assigned ID.
    pub fn add(&mut self, mut obs: VisualObservation) -> u64 {
        let id = self.next_id;
        obs.id = id;
        self.next_id += 1;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.observations.push(obs);
        id
    }

    /// Return the number of observations.
    pub fn count(&self) -> usize {
        self.observations.len()
    }

    /// Get observations filtered by session ID.
    pub fn by_session(&self, session_id: u32) -> Vec<&VisualObservation> {
        self.observations
            .iter()
            .filter(|o| o.session_id == session_id)
            .collect()
    }

    /// Get observations in a timestamp range.
    pub fn in_time_range(&self, start: u64, end: u64) -> Vec<&VisualObservation> {
        self.observations
            .iter()
            .filter(|o| o.timestamp >= start && o.timestamp <= end)
            .collect()
    }

    /// Get the most recent observations.
    pub fn recent(&self, limit: usize) -> Vec<&VisualObservation> {
        let mut sorted: Vec<_> = self.observations.iter().collect();
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted.truncate(limit);
        sorted
    }
}

/// Errors that can occur in the vision library.
#[derive(thiserror::Error, Debug)]
pub enum VisionError {
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Capture not found: {0}")]
    CaptureNotFound(u64),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Capture error: {0}")]
    Capture(String),

    #[error("Model not available: {0}")]
    ModelNotAvailable(String),
}

/// Convenience result type.
pub type VisionResult<T> = Result<T, VisionError>;
