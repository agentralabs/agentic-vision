//! .avis binary file format reader/writer for visual memory.

use std::io::{Read, Write};
use std::path::Path;

use crate::perception::cache::IntentCache;
use crate::perception::drift::DriftHistory;
use crate::perception::grammar::GrammarStore;
use crate::types::{VisionError, VisionResult, VisualMemoryStore, VisualObservation};

/// Magic bytes: "AVIS"
const AVIS_MAGIC: u32 = 0x41564953;

/// Current format version (v2: adds grammar store, intent cache, drift history).
const FORMAT_VERSION: u16 = 2;

/// Minimum version we can read (backward compatible with v1).
const MIN_READABLE_VERSION: u16 = 1;

/// Header size in bytes.
const HEADER_SIZE: usize = 64;

/// Writer for .avis files.
pub struct AvisWriter;

/// Reader for .avis files.
pub struct AvisReader;

/// Extended store container for v2 format.
pub struct AvisStoreV2 {
    pub store: VisualMemoryStore,
    pub grammar_store: GrammarStore,
    pub intent_cache: IntentCache,
    pub drift_history: DriftHistory,
}

impl AvisStoreV2 {
    /// Create a new empty v2 store.
    pub fn new(embedding_dim: u32) -> Self {
        Self {
            store: VisualMemoryStore::new(embedding_dim),
            grammar_store: GrammarStore::new(),
            intent_cache: IntentCache::new(),
            drift_history: DriftHistory::new(),
        }
    }

    /// Create from an existing v1 store (migration).
    pub fn from_v1(store: VisualMemoryStore) -> Self {
        Self {
            store,
            grammar_store: GrammarStore::new(),
            intent_cache: IntentCache::new(),
            drift_history: DriftHistory::new(),
        }
    }
}

impl AvisWriter {
    /// Write a visual memory store to a file.
    pub fn write_to_file(store: &VisualMemoryStore, path: &Path) -> VisionResult<()> {
        let v2 = AvisStoreV2 {
            store: store.clone(),
            grammar_store: GrammarStore::new(),
            intent_cache: IntentCache::new(),
            drift_history: DriftHistory::new(),
        };
        Self::write_v2_to_file(&v2, path)
    }

    /// Write a v2 store to a file.
    pub fn write_v2_to_file(store: &AvisStoreV2, path: &Path) -> VisionResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::File::create(path)?;
        Self::write_v2_to(store, &mut file)
    }

    /// Write a visual memory store to any writer (v1 compatibility).
    pub fn write_to<W: Write>(store: &VisualMemoryStore, writer: &mut W) -> VisionResult<()> {
        let v2 = AvisStoreV2 {
            store: store.clone(),
            grammar_store: GrammarStore::new(),
            intent_cache: IntentCache::new(),
            drift_history: DriftHistory::new(),
        };
        Self::write_v2_to(&v2, writer)
    }

    /// Write a v2 store to any writer.
    pub fn write_v2_to<W: Write>(store: &AvisStoreV2, writer: &mut W) -> VisionResult<()> {
        let payload = serde_json::to_vec(&SerializedStoreV2 {
            observations: &store.store.observations,
            embedding_dim: store.store.embedding_dim,
            next_id: store.store.next_id,
            session_count: store.store.session_count,
            created_at: store.store.created_at,
            updated_at: store.store.updated_at,
            grammar_store: &store.grammar_store,
            intent_cache: &store.intent_cache,
            drift_history: &store.drift_history,
        })
        .map_err(|e| VisionError::Storage(format!("Serialization failed: {e}")))?;

        // Write header
        let mut header = [0u8; HEADER_SIZE];
        write_u32(&mut header[0..4], AVIS_MAGIC);
        write_u16(&mut header[4..6], FORMAT_VERSION);
        write_u16(&mut header[6..8], 0); // flags
        write_u64(&mut header[8..16], store.store.observations.len() as u64);
        write_u32(&mut header[16..20], store.store.embedding_dim);
        write_u32(&mut header[20..24], store.store.session_count);
        write_u64(&mut header[24..32], store.store.created_at);
        write_u64(&mut header[32..40], store.store.updated_at);
        write_u64(&mut header[40..48], payload.len() as u64);

        writer.write_all(&header)?;
        writer.write_all(&payload)?;

        Ok(())
    }
}

impl AvisReader {
    /// Read a visual memory store from a file (returns v1 VisualMemoryStore for compatibility).
    pub fn read_from_file(path: &Path) -> VisionResult<VisualMemoryStore> {
        let v2 = Self::read_v2_from_file(path)?;
        Ok(v2.store)
    }

    /// Read a v2 store from a file.
    pub fn read_v2_from_file(path: &Path) -> VisionResult<AvisStoreV2> {
        let mut file = std::fs::File::open(path)?;
        Self::read_v2_from(&mut file)
    }

    /// Read a visual memory store from any reader (v1 compatibility).
    pub fn read_from<R: Read>(reader: &mut R) -> VisionResult<VisualMemoryStore> {
        let v2 = Self::read_v2_from(reader)?;
        Ok(v2.store)
    }

    /// Read a v2 store from any reader (handles v1 migration automatically).
    pub fn read_v2_from<R: Read>(reader: &mut R) -> VisionResult<AvisStoreV2> {
        // Read header
        let mut header = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header)?;

        let magic = read_u32(&header[0..4]);
        if magic != AVIS_MAGIC {
            return Err(VisionError::Storage(format!(
                "Invalid magic: expected 0x{AVIS_MAGIC:08X}, got 0x{magic:08X}"
            )));
        }

        let version = read_u16(&header[4..6]);
        if !(MIN_READABLE_VERSION..=FORMAT_VERSION).contains(&version) {
            return Err(VisionError::Storage(format!(
                "Unsupported version: {version} (supported: {MIN_READABLE_VERSION}-{FORMAT_VERSION})"
            )));
        }

        let _observation_count = read_u64(&header[8..16]);
        let embedding_dim = read_u32(&header[16..20]);
        let session_count = read_u32(&header[20..24]);
        let created_at = read_u64(&header[24..32]);
        let updated_at = read_u64(&header[32..40]);
        let payload_len = read_u64(&header[40..48]) as usize;

        // Read payload
        let mut payload = vec![0u8; payload_len];
        reader.read_exact(&mut payload)?;

        if version == 1 {
            // v1 format: only observations, no grammar/cache/drift
            let serialized: DeserializedStore = serde_json::from_slice(&payload)
                .map_err(|e| VisionError::Storage(format!("v1 deserialization failed: {e}")))?;

            let store = VisualMemoryStore {
                observations: serialized.observations,
                embedding_dim,
                next_id: serialized.next_id,
                session_count,
                created_at,
                updated_at,
            };

            Ok(AvisStoreV2::from_v1(store))
        } else {
            // v2 format: full store with grammar, cache, drift
            let serialized: DeserializedStoreV2 = serde_json::from_slice(&payload)
                .map_err(|e| VisionError::Storage(format!("v2 deserialization failed: {e}")))?;

            let store = VisualMemoryStore {
                observations: serialized.observations,
                embedding_dim,
                next_id: serialized.next_id,
                session_count,
                created_at,
                updated_at,
            };

            Ok(AvisStoreV2 {
                store,
                grammar_store: serialized.grammar_store.unwrap_or_default(),
                intent_cache: serialized.intent_cache.unwrap_or_default(),
                drift_history: serialized.drift_history.unwrap_or_default(),
            })
        }
    }
}

// ── v1 serialization (for reading old files) ──

#[derive(serde::Deserialize)]
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

// ── v2 serialization (current format) ──

#[derive(serde::Serialize)]
struct SerializedStoreV2<'a> {
    observations: &'a [VisualObservation],
    embedding_dim: u32,
    next_id: u64,
    session_count: u32,
    created_at: u64,
    updated_at: u64,
    grammar_store: &'a GrammarStore,
    intent_cache: &'a IntentCache,
    drift_history: &'a DriftHistory,
}

#[derive(serde::Deserialize)]
struct DeserializedStoreV2 {
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
    grammar_store: Option<GrammarStore>,
    intent_cache: Option<IntentCache>,
    drift_history: Option<DriftHistory>,
}

// Little-endian byte helpers
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perception::grammar::SiteGrammar;
    use crate::types::{CaptureSource, ObservationMeta};

    fn make_test_observation(id: u64) -> VisualObservation {
        VisualObservation {
            id,
            timestamp: 1708345678,
            session_id: 1,
            source: CaptureSource::File {
                path: "/test/image.png".to_string(),
            },
            embedding: vec![0.1, 0.2, 0.3],
            thumbnail: vec![0xFF, 0xD8, 0xFF],
            metadata: ObservationMeta {
                width: 512,
                height: 512,
                original_width: 1920,
                original_height: 1080,
                labels: vec!["test".to_string()],
                description: Some("Test observation".to_string()),
                quality_score: 0.85,
            },
            memory_link: None,
        }
    }

    #[test]
    fn test_roundtrip_empty() {
        let store = VisualMemoryStore::new(512);
        let mut buf = Vec::new();
        AvisWriter::write_to(&store, &mut buf).unwrap();

        let loaded = AvisReader::read_from(&mut &buf[..]).unwrap();
        assert_eq!(loaded.count(), 0);
        assert_eq!(loaded.embedding_dim, 512);
    }

    #[test]
    fn test_roundtrip_with_observations() {
        let mut store = VisualMemoryStore::new(512);
        store.add(make_test_observation(0));
        store.add(make_test_observation(0));

        let mut buf = Vec::new();
        AvisWriter::write_to(&store, &mut buf).unwrap();

        let loaded = AvisReader::read_from(&mut &buf[..]).unwrap();
        assert_eq!(loaded.count(), 2);
        assert_eq!(loaded.observations[0].id, 1);
        assert_eq!(loaded.observations[1].id, 2);
    }

    #[test]
    fn test_invalid_magic() {
        let mut buf = [0u8; HEADER_SIZE + 10];
        buf[0..4].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        let result = AvisReader::read_from(&mut &buf[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.avis");

        let mut store = VisualMemoryStore::new(512);
        store.add(make_test_observation(0));

        AvisWriter::write_to_file(&store, &path).unwrap();
        let loaded = AvisReader::read_from_file(&path).unwrap();
        assert_eq!(loaded.count(), 1);
    }

    #[test]
    fn test_v2_roundtrip_with_grammars() {
        let mut v2 = AvisStoreV2::new(512);
        v2.store.add(make_test_observation(0));

        // Add a grammar
        let mut grammar = SiteGrammar::new("amazon.com");
        grammar.add_content("product_price", ".a-price-whole");
        grammar.add_content("product_title", "#productTitle");
        grammar.add_intent_route("find_price", vec!["product_price".into()], None);
        v2.grammar_store.insert(grammar);

        let mut buf = Vec::new();
        AvisWriter::write_v2_to(&v2, &mut buf).unwrap();

        let loaded = AvisReader::read_v2_from(&mut &buf[..]).unwrap();
        assert_eq!(loaded.store.count(), 1);
        assert!(loaded.grammar_store.has("amazon.com"));
        let g = loaded.grammar_store.get("amazon.com").unwrap();
        assert_eq!(g.content_map.len(), 2);
        assert_eq!(g.intent_routes.len(), 1);
    }

    #[test]
    fn test_v2_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_v2.avis");

        let mut v2 = AvisStoreV2::new(512);
        let mut grammar = SiteGrammar::new("github.com");
        grammar.add_content("repo_files", "[role=gridcell]");
        v2.grammar_store.insert(grammar);

        AvisWriter::write_v2_to_file(&v2, &path).unwrap();
        let loaded = AvisReader::read_v2_from_file(&path).unwrap();
        assert!(loaded.grammar_store.has("github.com"));
    }
}
