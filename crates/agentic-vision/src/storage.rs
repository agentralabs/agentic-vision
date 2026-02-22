//! .avis binary file format reader/writer for visual memory.

use std::io::{Read, Write};
use std::path::Path;

use crate::types::{VisionError, VisionResult, VisualMemoryStore, VisualObservation};

/// Magic bytes: "AVIS"
const AVIS_MAGIC: u32 = 0x41564953;

/// Current format version.
const FORMAT_VERSION: u16 = 1;

/// Header size in bytes.
const HEADER_SIZE: usize = 64;

/// Writer for .avis files.
pub struct AvisWriter;

/// Reader for .avis files.
pub struct AvisReader;

impl AvisWriter {
    /// Write a visual memory store to a file.
    pub fn write_to_file(store: &VisualMemoryStore, path: &Path) -> VisionResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::File::create(path)?;
        Self::write_to(store, &mut file)
    }

    /// Write a visual memory store to any writer.
    pub fn write_to<W: Write>(store: &VisualMemoryStore, writer: &mut W) -> VisionResult<()> {
        // Serialize observations as JSON (simple, correct, can optimize later)
        let payload = serde_json::to_vec(&SerializedStore {
            observations: &store.observations,
            embedding_dim: store.embedding_dim,
            next_id: store.next_id,
            session_count: store.session_count,
            created_at: store.created_at,
            updated_at: store.updated_at,
        })
        .map_err(|e| VisionError::Storage(format!("Serialization failed: {e}")))?;

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
        write_u64(&mut header[40..48], payload.len() as u64); // payload length

        writer.write_all(&header)?;
        writer.write_all(&payload)?;

        Ok(())
    }
}

impl AvisReader {
    /// Read a visual memory store from a file.
    pub fn read_from_file(path: &Path) -> VisionResult<VisualMemoryStore> {
        let mut file = std::fs::File::open(path)?;
        Self::read_from(&mut file)
    }

    /// Read a visual memory store from any reader.
    pub fn read_from<R: Read>(reader: &mut R) -> VisionResult<VisualMemoryStore> {
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
        if version != FORMAT_VERSION {
            return Err(VisionError::Storage(format!(
                "Unsupported version: {version}"
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

        let serialized: DeserializedStore = serde_json::from_slice(&payload)
            .map_err(|e| VisionError::Storage(format!("Deserialization failed: {e}")))?;

        let next_id = serialized.next_id;

        Ok(VisualMemoryStore {
            observations: serialized.observations,
            embedding_dim,
            next_id,
            session_count,
            created_at,
            updated_at,
        })
    }
}

#[derive(serde::Serialize)]
struct SerializedStore<'a> {
    observations: &'a [VisualObservation],
    embedding_dim: u32,
    next_id: u64,
    session_count: u32,
    created_at: u64,
    updated_at: u64,
}

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
}
