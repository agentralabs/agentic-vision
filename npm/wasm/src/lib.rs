//! WASM bindings for agentic-vision.
//!
//! Exposes pure-computation operations that are compatible with WebAssembly:
//! - Cosine similarity between embedding vectors
//! - Top-k similarity search over visual observations
//! - Visual observation / memory store data types (JSON serialization)
//! - .avis binary file format reader/writer (from/to byte buffers)
//!
//! OS-dependent features (screenshot, clipboard, ONNX embedding) are NOT
//! included — they require native dependencies that cannot target wasm32.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// Core types (mirrored from agentic-vision::types, kept in sync manually)
// ---------------------------------------------------------------------------

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
    #[serde(default)]
    pub quality_score: f32,
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

/// Pixel-level diff metadata between two captures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualDiff {
    pub before_id: u64,
    pub after_id: u64,
    pub similarity: f32,
    pub changed_regions: Vec<Rect>,
    pub pixel_diff_ratio: f32,
}

/// In-memory container for all visual observations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualMemoryStore {
    pub observations: Vec<VisualObservation>,
    pub embedding_dim: u32,
    pub next_id: u64,
    pub session_count: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Default CLIP embedding dimension.
pub const EMBEDDING_DIM: u32 = 512;

// ---------------------------------------------------------------------------
// Similarity — pure math
// ---------------------------------------------------------------------------

/// Compute cosine similarity between two f32 vectors.
///
/// Accepts two `Float32Array` values from JS. Returns a scalar in [-1, 1].
#[wasm_bindgen(js_name = "cosineSimilarity")]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f64;
    let mut norm_a = 0.0f64;
    let mut norm_b = 0.0f64;

    for (x, y) in a.iter().zip(b.iter()) {
        let x = *x as f64;
        let y = *y as f64;
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        return 0.0;
    }

    (dot / denom) as f32
}

/// Find the top-k most similar observations by embedding.
///
/// `query` — Float32Array embedding vector.
/// `observations_json` — JSON string of `VisualObservation[]`.
/// `top_k` — maximum number of results.
/// `min_similarity` — minimum similarity threshold (0.0 to 1.0).
///
/// Returns a JSON string of `SimilarityMatch[]`.
#[wasm_bindgen(js_name = "findSimilar")]
pub fn find_similar(
    query: &[f32],
    observations_json: &str,
    top_k: usize,
    min_similarity: f32,
) -> Result<String, JsError> {
    let observations: Vec<VisualObservation> = serde_json::from_str(observations_json)
        .map_err(|e| JsError::new(&format!("Failed to parse observations JSON: {e}")))?;

    let mut matches: Vec<SimilarityMatch> = observations
        .iter()
        .filter(|o| !o.embedding.is_empty())
        .map(|o| SimilarityMatch {
            id: o.id,
            similarity: cosine_similarity(query, &o.embedding),
        })
        .filter(|m| m.similarity >= min_similarity)
        .collect();

    matches.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.truncate(top_k);

    serde_json::to_string(&matches)
        .map_err(|e| JsError::new(&format!("Failed to serialize matches: {e}")))
}

// ---------------------------------------------------------------------------
// .avis binary format reader/writer (from/to byte buffers)
// ---------------------------------------------------------------------------

/// Magic bytes: "AVIS"
const AVIS_MAGIC: u32 = 0x41564953;

/// Current format version.
const FORMAT_VERSION: u16 = 1;

/// Header size in bytes.
const HEADER_SIZE: usize = 64;

/// Read a `.avis` binary file from a `Uint8Array` and return JSON of the
/// `VisualMemoryStore`.
#[wasm_bindgen(js_name = "avisRead")]
pub fn avis_read(data: &[u8]) -> Result<String, JsError> {
    if data.len() < HEADER_SIZE {
        return Err(JsError::new("Data too short for .avis header"));
    }

    let magic = read_u32(&data[0..4]);
    if magic != AVIS_MAGIC {
        return Err(JsError::new(&format!(
            "Invalid magic: expected 0x{AVIS_MAGIC:08X}, got 0x{magic:08X}"
        )));
    }

    let version = read_u16(&data[4..6]);
    if version != FORMAT_VERSION {
        return Err(JsError::new(&format!(
            "Unsupported version: {version}"
        )));
    }

    let _observation_count = read_u64(&data[8..16]);
    let embedding_dim = read_u32(&data[16..20]);
    let session_count = read_u32(&data[20..24]);
    let created_at = read_u64(&data[24..32]);
    let updated_at = read_u64(&data[32..40]);
    let payload_len = read_u64(&data[40..48]) as usize;

    let payload_start = HEADER_SIZE;
    let payload_end = payload_start + payload_len;
    if data.len() < payload_end {
        return Err(JsError::new(&format!(
            "Data too short: need {payload_end} bytes, got {}",
            data.len()
        )));
    }

    let payload = &data[payload_start..payload_end];

    let deserialized: DeserializedStore = serde_json::from_slice(payload)
        .map_err(|e| JsError::new(&format!("Deserialization failed: {e}")))?;

    let store = VisualMemoryStore {
        observations: deserialized.observations,
        embedding_dim,
        next_id: deserialized.next_id,
        session_count,
        created_at,
        updated_at,
    };

    serde_json::to_string(&store)
        .map_err(|e| JsError::new(&format!("Failed to serialize store: {e}")))
}

/// Serialize a `VisualMemoryStore` (as JSON string) into `.avis` binary format,
/// returned as a `Uint8Array`.
#[wasm_bindgen(js_name = "avisWrite")]
pub fn avis_write(store_json: &str) -> Result<Vec<u8>, JsError> {
    let store: VisualMemoryStore = serde_json::from_str(store_json)
        .map_err(|e| JsError::new(&format!("Failed to parse store JSON: {e}")))?;

    let payload = serde_json::to_vec(&SerializedStore {
        observations: &store.observations,
        embedding_dim: store.embedding_dim,
        next_id: store.next_id,
        session_count: store.session_count,
        created_at: store.created_at,
        updated_at: store.updated_at,
    })
    .map_err(|e| JsError::new(&format!("Serialization failed: {e}")))?;

    let mut output = Vec::with_capacity(HEADER_SIZE + payload.len());

    // Write header
    let mut header = [0u8; HEADER_SIZE];
    write_u32(&mut header[0..4], AVIS_MAGIC);
    write_u16(&mut header[4..6], FORMAT_VERSION);
    write_u16(&mut header[6..8], 0); // flags
    write_u64(&mut header[8..16], store.observations.len() as u64);
    write_u32(&mut header[16..20], store.embedding_dim);
    write_u32(&mut header[20..24], store.session_count);
    write_u64(&mut header[24..32], store.created_at);
    write_u64(&mut header[32..40], store.updated_at);
    write_u64(&mut header[40..48], payload.len() as u64);

    output.extend_from_slice(&header);
    output.extend_from_slice(&payload);

    Ok(output)
}

// ---------------------------------------------------------------------------
// Visual observation helpers
// ---------------------------------------------------------------------------

/// Create a new empty `VisualMemoryStore` and return it as JSON.
///
/// `embedding_dim` — the embedding vector dimension (typically 512).
#[wasm_bindgen(js_name = "createMemoryStore")]
pub fn create_memory_store(embedding_dim: u32) -> String {
    let now_ms = js_sys::Date::now() as u64 / 1000;
    let store = VisualMemoryStore {
        observations: Vec::new(),
        embedding_dim,
        next_id: 1,
        session_count: 0,
        created_at: now_ms,
        updated_at: now_ms,
    };
    serde_json::to_string(&store).unwrap_or_else(|_| "{}".to_string())
}

/// Add an observation (JSON) to a store (JSON) and return the updated store JSON.
///
/// The observation `id` will be assigned automatically.
#[wasm_bindgen(js_name = "addObservation")]
pub fn add_observation(store_json: &str, observation_json: &str) -> Result<String, JsError> {
    let mut store: VisualMemoryStore = serde_json::from_str(store_json)
        .map_err(|e| JsError::new(&format!("Failed to parse store JSON: {e}")))?;

    let mut obs: VisualObservation = serde_json::from_str(observation_json)
        .map_err(|e| JsError::new(&format!("Failed to parse observation JSON: {e}")))?;

    obs.id = store.next_id;
    store.next_id += 1;
    let now_ms = js_sys::Date::now() as u64 / 1000;
    store.updated_at = now_ms;
    store.observations.push(obs);

    serde_json::to_string(&store)
        .map_err(|e| JsError::new(&format!("Failed to serialize store: {e}")))
}

/// Get the CLIP embedding dimension constant.
#[wasm_bindgen(js_name = "embeddingDim")]
pub fn embedding_dim() -> u32 {
    EMBEDDING_DIM
}

/// Return the library version string.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ---------------------------------------------------------------------------
// Internal helpers — .avis byte serialization
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct SerializedStore<'a> {
    observations: &'a [VisualObservation],
    embedding_dim: u32,
    next_id: u64,
    session_count: u32,
    created_at: u64,
    updated_at: u64,
}

#[derive(Deserialize)]
struct DeserializedStore {
    observations: Vec<VisualObservation>,
    #[allow(dead_code)]
    embedding_dim: u32,
    next_id: u64,
    #[allow(dead_code)]
    session_count: u32,
    #[allow(dead_code)]
    created_at: u64,
    #[allow(dead_code)]
    updated_at: u64,
}

fn write_u16(buf: &mut [u8], val: u16) {
    buf[..2].copy_from_slice(&val.to_le_bytes());
}
fn write_u32(buf: &mut [u8], val: u32) {
    buf[..4].copy_from_slice(&val.to_le_bytes());
}
fn write_u64(buf: &mut [u8], val: u64) {
    buf[..8].copy_from_slice(&val.to_le_bytes());
}
fn read_u16(buf: &[u8]) -> u16 {
    u16::from_le_bytes([buf[0], buf[1]])
}
fn read_u32(buf: &[u8]) -> u32 {
    u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]])
}
fn read_u64(buf: &[u8]) -> u64 {
    u64::from_le_bytes([
        buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
    ])
}
