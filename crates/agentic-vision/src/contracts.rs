//! Contracts bridge — implements agentic-contracts v0.2.0 traits for Vision.
//!
//! This module provides `VisionSister`, a contracts-compliant wrapper
//! around the core `VisualMemoryStore` + storage. It implements:
//!
//! - `Sister` — lifecycle management
//! - `SessionManagement` — append-only sequential sessions
//! - `Grounding` — description/label-based claim verification
//! - `Queryable` — unified query interface
//! - `FileFormatReader/FileFormatWriter` — .avis file I/O
//!
//! The MCP server can use `VisionSister` instead of raw store + storage
//! to get compile-time contracts compliance.

use agentic_contracts::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::storage::{AvisReader, AvisWriter};
use crate::types::{VisionError, VisualMemoryStore, VisualObservation};

// ═══════════════════════════════════════════════════════════════════
// ERROR BRIDGE: VisionError → SisterError
// ═══════════════════════════════════════════════════════════════════

impl From<VisionError> for SisterError {
    fn from(e: VisionError) -> Self {
        match &e {
            VisionError::CaptureNotFound(id) => SisterError::not_found(format!("capture {}", id)),
            VisionError::InvalidInput(msg) => {
                SisterError::new(ErrorCode::InvalidInput, msg.clone())
            }
            VisionError::Storage(msg) => SisterError::new(ErrorCode::StorageError, msg.clone()),
            VisionError::Io(io_err) => {
                SisterError::new(ErrorCode::StorageError, format!("I/O error: {}", io_err))
            }
            VisionError::Embedding(msg) => {
                SisterError::new(ErrorCode::VisionError, format!("Embedding error: {}", msg))
            }
            VisionError::Image(img_err) => {
                SisterError::new(ErrorCode::VisionError, format!("Image error: {}", img_err))
            }
            VisionError::Capture(msg) => {
                SisterError::new(ErrorCode::VisionError, format!("Capture error: {}", msg))
            }
            VisionError::ModelNotAvailable(msg) => SisterError::new(
                ErrorCode::VisionError,
                format!("Model not available: {}", msg),
            ),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// SESSION STATE
// ═══════════════════════════════════════════════════════════════════

/// Session record for tracking sessions in VisionSister.
#[derive(Debug, Clone)]
struct SessionRecord {
    id: ContextId,
    session_id: u32,
    name: String,
    created_at: chrono::DateTime<chrono::Utc>,
    capture_count_at_start: usize,
}

// ═══════════════════════════════════════════════════════════════════
// VISION SISTER — The contracts-compliant facade
// ═══════════════════════════════════════════════════════════════════

/// Contracts-compliant Vision sister.
///
/// Wraps `VisualMemoryStore` and implements all v0.2.0 traits.
/// This is the canonical "Vision as a sister" interface.
pub struct VisionSister {
    store: VisualMemoryStore,
    file_path: Option<PathBuf>,
    start_time: Instant,

    // Session state
    current_session: Option<SessionRecord>,
    sessions: Vec<SessionRecord>,
    next_session_id: u32,
}

impl VisionSister {
    /// Create from an existing store (for migration from existing code).
    pub fn from_store(store: VisualMemoryStore, file_path: Option<PathBuf>) -> Self {
        Self {
            store,
            file_path,
            start_time: Instant::now(),
            current_session: None,
            sessions: vec![],
            next_session_id: 1,
        }
    }

    /// Get a reference to the underlying store.
    pub fn store(&self) -> &VisualMemoryStore {
        &self.store
    }

    /// Get a mutable reference to the underlying store.
    pub fn store_mut(&mut self) -> &mut VisualMemoryStore {
        &mut self.store
    }

    /// Get the current u32 session ID (for interop with existing code).
    pub fn current_session_id(&self) -> Option<u32> {
        self.current_session.as_ref().map(|s| s.session_id)
    }
}

// ═══════════════════════════════════════════════════════════════════
// SISTER TRAIT
// ═══════════════════════════════════════════════════════════════════

impl Sister for VisionSister {
    const SISTER_TYPE: SisterType = SisterType::Vision;
    const FILE_EXTENSION: &'static str = "avis";

    fn init(config: SisterConfig) -> SisterResult<Self>
    where
        Self: Sized,
    {
        let embedding_dim = config.get_option::<u32>("embedding_dim").unwrap_or(512);

        let file_path = config.data_path.clone();

        let store = if let Some(ref path) = file_path {
            if path.exists() {
                AvisReader::read_from_file(path).map_err(SisterError::from)?
            } else if config.create_if_missing {
                VisualMemoryStore::new(embedding_dim)
            } else {
                return Err(SisterError::new(
                    ErrorCode::NotFound,
                    format!("Vision file not found: {}", path.display()),
                ));
            }
        } else {
            VisualMemoryStore::new(embedding_dim)
        };

        Ok(Self::from_store(store, file_path))
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            status: Status::Ready,
            uptime: self.start_time.elapsed(),
            resources: ResourceUsage {
                memory_bytes: self.store.count() * 1024, // rough estimate per capture
                disk_bytes: 0,
                open_handles: if self.file_path.is_some() { 1 } else { 0 },
            },
            warnings: vec![],
            last_error: None,
        }
    }

    fn version(&self) -> Version {
        Version::new(0, 3, 0) // matches agentic-vision crate version
    }

    fn shutdown(&mut self) -> SisterResult<()> {
        // End current session if active
        if self.current_session.is_some() {
            let _ = SessionManagement::end_session(self);
        }

        // Save to file if path is set
        if let Some(ref path) = self.file_path {
            AvisWriter::write_to_file(&self.store, path).map_err(SisterError::from)?;
        }

        Ok(())
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::new(
                "vision_capture",
                "Capture an image and store in visual memory",
            ),
            Capability::new("vision_query", "Search visual memory by filters"),
            Capability::new("vision_compare", "Compare two captures for similarity"),
            Capability::new(
                "vision_ground",
                "Verify visual claims against stored captures",
            ),
            Capability::new(
                "vision_evidence",
                "Get detailed evidence for a visual query",
            ),
            Capability::new(
                "vision_suggest",
                "Find similar captures when exact match fails",
            ),
            Capability::new(
                "vision_similar",
                "Find visually similar captures by embedding",
            ),
            Capability::new("vision_diff", "Get pixel-level diff between two captures"),
            Capability::new("vision_track", "Configure tracking for a UI region"),
            Capability::new("observation_log", "Log observation intent and context"),
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════
// SESSION MANAGEMENT
// ═══════════════════════════════════════════════════════════════════

impl SessionManagement for VisionSister {
    fn start_session(&mut self, name: &str) -> SisterResult<ContextId> {
        // End current session if active
        if self.current_session.is_some() {
            self.end_session()?;
        }

        let session_id = self.next_session_id;
        self.next_session_id += 1;
        let context_id = ContextId::new();

        let record = SessionRecord {
            id: context_id,
            session_id,
            name: name.to_string(),
            created_at: chrono::Utc::now(),
            capture_count_at_start: self.store.count(),
        };

        self.current_session = Some(record.clone());
        self.sessions.push(record);

        Ok(context_id)
    }

    fn end_session(&mut self) -> SisterResult<()> {
        if self.current_session.is_none() {
            return Err(SisterError::new(
                ErrorCode::InvalidState,
                "No active session to end",
            ));
        }
        self.current_session = None;
        Ok(())
    }

    fn current_session(&self) -> Option<ContextId> {
        self.current_session.as_ref().map(|s| s.id)
    }

    fn current_session_info(&self) -> SisterResult<ContextInfo> {
        let session = self
            .current_session
            .as_ref()
            .ok_or_else(|| SisterError::new(ErrorCode::InvalidState, "No active session"))?;

        let captures_in_session = self.store.count() - session.capture_count_at_start;

        Ok(ContextInfo {
            id: session.id,
            name: session.name.clone(),
            created_at: session.created_at,
            updated_at: chrono::Utc::now(),
            item_count: captures_in_session,
            size_bytes: captures_in_session * 1024,
            metadata: Metadata::new(),
        })
    }

    fn list_sessions(&self) -> SisterResult<Vec<ContextSummary>> {
        Ok(self
            .sessions
            .iter()
            .rev() // most recent first
            .map(|s| ContextSummary {
                id: s.id,
                name: s.name.clone(),
                created_at: s.created_at,
                updated_at: s.created_at, // approximate
                item_count: 0,            // would need per-session tracking
                size_bytes: 0,
            })
            .collect())
    }

    fn export_session(&self, id: ContextId) -> SisterResult<ContextSnapshot> {
        let session = self
            .sessions
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| SisterError::context_not_found(id.to_string()))?;

        // Export all captures from this session
        let session_captures: Vec<&VisualObservation> = self
            .store
            .observations
            .iter()
            .filter(|o| o.session_id == session.session_id)
            .collect();

        let data = serde_json::to_vec(&session_captures)
            .map_err(|e| SisterError::new(ErrorCode::Internal, e.to_string()))?;
        let checksum = *blake3::hash(&data).as_bytes();

        Ok(ContextSnapshot {
            sister_type: SisterType::Vision,
            version: Version::new(0, 3, 0),
            context_info: ContextInfo {
                id,
                name: session.name.clone(),
                created_at: session.created_at,
                updated_at: chrono::Utc::now(),
                item_count: session_captures.len(),
                size_bytes: data.len(),
                metadata: Metadata::new(),
            },
            data,
            checksum,
            snapshot_at: chrono::Utc::now(),
        })
    }

    fn import_session(&mut self, snapshot: ContextSnapshot) -> SisterResult<ContextId> {
        if !snapshot.verify() {
            return Err(SisterError::new(
                ErrorCode::ChecksumMismatch,
                "Session snapshot checksum verification failed",
            ));
        }

        // Start a new session for the imported data
        let context_id = self.start_session(&snapshot.context_info.name)?;

        // Deserialize and ingest the captures
        let captures: Vec<VisualObservation> = serde_json::from_slice(&snapshot.data)
            .map_err(|e| SisterError::new(ErrorCode::InvalidInput, e.to_string()))?;

        let session_id = self.current_session_id().unwrap_or(0);
        for mut capture in captures {
            capture.session_id = session_id;
            self.store.add(capture);
        }

        Ok(context_id)
    }
}

// ═══════════════════════════════════════════════════════════════════
// GROUNDING
// ═══════════════════════════════════════════════════════════════════

impl Grounding for VisionSister {
    fn ground(&self, claim: &str) -> SisterResult<GroundingResult> {
        let claim_lower = claim.to_lowercase();
        let claim_words: Vec<&str> = claim_lower.split_whitespace().collect();

        if claim_words.is_empty() {
            return Ok(GroundingResult::ungrounded(claim, "Empty claim"));
        }

        // Score each capture by word overlap on description + labels
        let mut scored: Vec<(f64, &VisualObservation)> = self
            .store
            .observations
            .iter()
            .map(|obs| {
                let mut text = String::new();
                if let Some(ref desc) = obs.metadata.description {
                    text.push_str(&desc.to_lowercase());
                    text.push(' ');
                }
                for label in &obs.metadata.labels {
                    text.push_str(&label.to_lowercase());
                    text.push(' ');
                }

                let matched = claim_words.iter().filter(|w| text.contains(**w)).count();
                let score = matched as f64 / claim_words.len() as f64;
                (score, obs)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        if scored.is_empty() {
            return Ok(
                GroundingResult::ungrounded(claim, "No matching captures found").with_suggestions(
                    self.store
                        .observations
                        .iter()
                        .rev()
                        .take(3)
                        .filter_map(|o| o.metadata.description.clone())
                        .collect(),
                ),
            );
        }

        let best_score = scored[0].0;
        let evidence: Vec<GroundingEvidence> = scored
            .iter()
            .take(10)
            .map(|(score, obs)| {
                let desc = obs
                    .metadata
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("capture_{}", obs.id));
                GroundingEvidence::new(
                    "visual_capture",
                    format!("capture_{}", obs.id),
                    *score,
                    &desc,
                )
                .with_data("labels", obs.metadata.labels.clone())
                .with_data("session_id", obs.session_id)
                .with_data("quality_score", obs.metadata.quality_score)
            })
            .collect();

        if best_score > 0.5 {
            Ok(GroundingResult::verified(claim, best_score)
                .with_evidence(evidence)
                .with_reason("Found matching captures via description/label search"))
        } else {
            Ok(GroundingResult::partial(claim, best_score)
                .with_evidence(evidence)
                .with_reason("Some evidence found but low relevance"))
        }
    }

    fn evidence(&self, query: &str, max_results: usize) -> SisterResult<Vec<EvidenceDetail>> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(f64, &VisualObservation)> = self
            .store
            .observations
            .iter()
            .map(|obs| {
                let mut text = String::new();
                if let Some(ref desc) = obs.metadata.description {
                    text.push_str(&desc.to_lowercase());
                    text.push(' ');
                }
                for label in &obs.metadata.labels {
                    text.push_str(&label.to_lowercase());
                    text.push(' ');
                }

                let matched = query_words.iter().filter(|w| text.contains(**w)).count();
                let score = if query_words.is_empty() {
                    0.0
                } else {
                    matched as f64 / query_words.len() as f64
                };
                (score, obs)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored
            .into_iter()
            .take(max_results)
            .map(|(score, obs)| {
                let created_at =
                    chrono::DateTime::from_timestamp(obs.timestamp as i64, 0).unwrap_or_default();

                EvidenceDetail {
                    evidence_type: "visual_capture".to_string(),
                    id: format!("capture_{}", obs.id),
                    score,
                    created_at,
                    source_sister: SisterType::Vision,
                    content: obs
                        .metadata
                        .description
                        .clone()
                        .unwrap_or_else(|| format!("capture_{}", obs.id)),
                    data: {
                        let mut meta = Metadata::new();
                        if let Ok(v) = serde_json::to_value(&obs.metadata.labels) {
                            meta.insert("labels".to_string(), v);
                        }
                        if let Ok(v) = serde_json::to_value(obs.session_id) {
                            meta.insert("session_id".to_string(), v);
                        }
                        if let Ok(v) = serde_json::to_value(obs.metadata.quality_score) {
                            meta.insert("quality_score".to_string(), v);
                        }
                        meta
                    },
                }
            })
            .collect())
    }

    fn suggest(&self, query: &str, limit: usize) -> SisterResult<Vec<GroundingSuggestion>> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(f64, &VisualObservation)> = self
            .store
            .observations
            .iter()
            .map(|obs| {
                let mut text = String::new();
                if let Some(ref desc) = obs.metadata.description {
                    text.push_str(&desc.to_lowercase());
                    text.push(' ');
                }
                for label in &obs.metadata.labels {
                    text.push_str(&label.to_lowercase());
                    text.push(' ');
                }

                let matched = query_words.iter().filter(|w| text.contains(**w)).count();
                let score = if query_words.is_empty() {
                    0.0
                } else {
                    matched as f64 / query_words.len() as f64
                };
                (score, obs)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored
            .into_iter()
            .take(limit)
            .map(|(score, obs)| GroundingSuggestion {
                item_type: "visual_capture".to_string(),
                id: format!("capture_{}", obs.id),
                relevance_score: score,
                description: obs
                    .metadata
                    .description
                    .clone()
                    .unwrap_or_else(|| format!("capture_{}", obs.id)),
                data: Metadata::new(),
            })
            .collect())
    }
}

// ═══════════════════════════════════════════════════════════════════
// QUERYABLE
// ═══════════════════════════════════════════════════════════════════

impl Queryable for VisionSister {
    fn query(&self, query: Query) -> SisterResult<QueryResult> {
        let start = Instant::now();

        let results: Vec<serde_json::Value> = match query.query_type.as_str() {
            "list" => {
                let limit = query.limit.unwrap_or(50);
                let offset = query.offset.unwrap_or(0);
                self.store
                    .observations
                    .iter()
                    .skip(offset)
                    .take(limit)
                    .map(|o| {
                        serde_json::json!({
                            "id": o.id,
                            "timestamp": o.timestamp,
                            "session_id": o.session_id,
                            "description": o.metadata.description,
                            "labels": o.metadata.labels,
                            "quality_score": o.metadata.quality_score,
                            "width": o.metadata.width,
                            "height": o.metadata.height,
                        })
                    })
                    .collect()
            }
            "search" => {
                let text = query.get_string("text").unwrap_or_default().to_lowercase();
                let max = query.limit.unwrap_or(20);

                self.store
                    .observations
                    .iter()
                    .filter(|o| {
                        let desc = o
                            .metadata
                            .description
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase();
                        let labels_text = o.metadata.labels.join(" ").to_lowercase();
                        desc.contains(&text) || labels_text.contains(&text)
                    })
                    .take(max)
                    .map(|o| {
                        serde_json::json!({
                            "id": o.id,
                            "description": o.metadata.description,
                            "labels": o.metadata.labels,
                            "timestamp": o.timestamp,
                        })
                    })
                    .collect()
            }
            "recent" => {
                let count = query.limit.unwrap_or(10);
                self.store
                    .recent(count)
                    .into_iter()
                    .map(|o| {
                        serde_json::json!({
                            "id": o.id,
                            "timestamp": o.timestamp,
                            "session_id": o.session_id,
                            "description": o.metadata.description,
                            "labels": o.metadata.labels,
                            "quality_score": o.metadata.quality_score,
                        })
                    })
                    .collect()
            }
            "get" => {
                let id_str = query.get_string("id").unwrap_or_default();
                let id: u64 = id_str.parse().unwrap_or(0);
                if let Some(o) = self.store.get(id) {
                    vec![serde_json::json!({
                        "id": o.id,
                        "timestamp": o.timestamp,
                        "session_id": o.session_id,
                        "description": o.metadata.description,
                        "labels": o.metadata.labels,
                        "quality_score": o.metadata.quality_score,
                        "width": o.metadata.width,
                        "height": o.metadata.height,
                        "original_width": o.metadata.original_width,
                        "original_height": o.metadata.original_height,
                        "memory_link": o.memory_link,
                    })]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        };

        let total = self.store.count();
        let has_more = results.len() < total;

        Ok(QueryResult::new(query, results, start.elapsed()).with_pagination(total, has_more))
    }

    fn supports_query(&self, query_type: &str) -> bool {
        matches!(query_type, "list" | "search" | "recent" | "get")
    }

    fn query_types(&self) -> Vec<QueryTypeInfo> {
        vec![
            QueryTypeInfo::new("list", "List all visual captures with pagination")
                .optional(vec!["limit", "offset"]),
            QueryTypeInfo::new("search", "Search captures by description/labels")
                .required(vec!["text"])
                .optional(vec!["limit"]),
            QueryTypeInfo::new("recent", "Get most recent captures").optional(vec!["limit"]),
            QueryTypeInfo::new("get", "Get a specific capture by ID").required(vec!["id"]),
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════
// FILE FORMAT
// ═══════════════════════════════════════════════════════════════════

impl FileFormatReader for VisionSister {
    fn read_file(path: &Path) -> SisterResult<Self> {
        let store = AvisReader::read_from_file(path).map_err(SisterError::from)?;
        Ok(Self::from_store(store, Some(path.to_path_buf())))
    }

    fn can_read(path: &Path) -> SisterResult<FileInfo> {
        // Read just the header to check format validity
        let data = std::fs::read(path)
            .map_err(|e| SisterError::new(ErrorCode::StorageError, e.to_string()))?;
        if data.len() < 64 {
            return Err(SisterError::new(
                ErrorCode::StorageError,
                "File too small for .avis format",
            ));
        }

        // Check AVIS magic bytes (0x41564953 little-endian)
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != 0x41564953 {
            return Err(SisterError::new(
                ErrorCode::VersionMismatch,
                format!("Invalid magic: expected AVIS, got 0x{:08X}", magic),
            ));
        }

        let version = u16::from_le_bytes([data[4], data[5]]);
        let metadata = std::fs::metadata(path)
            .map_err(|e| SisterError::new(ErrorCode::StorageError, e.to_string()))?;

        Ok(FileInfo {
            sister_type: SisterType::Vision,
            version: Version::new(version as u8, 0, 0),
            created_at: chrono::Utc::now(),
            updated_at: chrono::DateTime::from(
                metadata
                    .modified()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            ),
            content_length: metadata.len(),
            needs_migration: version < 1,
            format_id: "AVIS".to_string(),
        })
    }

    fn file_version(path: &Path) -> SisterResult<Version> {
        let data = std::fs::read(path)
            .map_err(|e| SisterError::new(ErrorCode::StorageError, e.to_string()))?;
        if data.len() < 6 {
            return Err(SisterError::new(
                ErrorCode::StorageError,
                "File too small for .avis format",
            ));
        }
        let version = u16::from_le_bytes([data[4], data[5]]);
        Ok(Version::new(version as u8, 0, 0))
    }

    fn migrate(_data: &[u8], _from_version: Version) -> SisterResult<Vec<u8>> {
        Err(SisterError::new(
            ErrorCode::NotImplemented,
            "No migration path available (only v1 exists)",
        ))
    }
}

impl FileFormatWriter for VisionSister {
    fn write_file(&self, path: &Path) -> SisterResult<()> {
        AvisWriter::write_to_file(&self.store, path).map_err(SisterError::from)
    }

    fn to_bytes(&self) -> SisterResult<Vec<u8>> {
        let mut buffer = Vec::new();
        AvisWriter::write_to(&self.store, &mut buffer).map_err(SisterError::from)?;
        Ok(buffer)
    }
}

// ═══════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CaptureSource, ObservationMeta};

    fn make_test_sister() -> VisionSister {
        let config = SisterConfig::stateless().option("embedding_dim", 512u32);
        VisionSister::init(config).unwrap()
    }

    fn make_test_observation(
        session_id: u32,
        description: &str,
        labels: Vec<&str>,
    ) -> VisualObservation {
        VisualObservation {
            id: 0, // will be assigned by store
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            session_id,
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
                labels: labels.into_iter().map(String::from).collect(),
                description: Some(description.to_string()),
                quality_score: 0.85,
            },
            memory_link: None,
        }
    }

    fn add_test_captures(sister: &mut VisionSister) {
        let session_id = sister.current_session_id().unwrap_or(0);
        sister.store_mut().add(make_test_observation(
            session_id,
            "Screenshot of the login page with blue header",
            vec!["ui", "login", "screenshot"],
        ));
        sister.store_mut().add(make_test_observation(
            session_id,
            "Dark mode toggle button in settings panel",
            vec!["ui", "settings", "dark-mode"],
        ));
        sister.store_mut().add(make_test_observation(
            session_id,
            "Error dialog showing network timeout",
            vec!["error", "network", "dialog"],
        ));
    }

    #[test]
    fn test_sister_trait() {
        let sister = make_test_sister();
        assert_eq!(sister.sister_type(), SisterType::Vision);
        assert_eq!(sister.file_extension(), "avis");
        assert_eq!(sister.mcp_prefix(), "vision");
        assert!(sister.is_healthy());
        assert_eq!(sister.version(), Version::new(0, 3, 0));
        assert!(!sister.capabilities().is_empty());
    }

    #[test]
    fn test_session_management() {
        let mut sister = make_test_sister();

        // No session initially
        assert!(sister.current_session().is_none());
        assert!(sister.current_session_info().is_err());

        // Start session
        let sid = sister.start_session("test_session").unwrap();
        assert!(sister.current_session().is_some());
        assert_eq!(sister.current_session().unwrap(), sid);

        // Session info
        let info = sister.current_session_info().unwrap();
        assert_eq!(info.name, "test_session");

        // List sessions
        let sessions = sister.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "test_session");

        // End session
        sister.end_session().unwrap();
        assert!(sister.current_session().is_none());

        // Can't end twice
        assert!(sister.end_session().is_err());
    }

    #[test]
    fn test_grounding() {
        let mut sister = make_test_sister();
        sister.start_session("grounding_test").unwrap();
        add_test_captures(&mut sister);

        // Ground a claim that should match
        let result = sister.ground("login page blue").unwrap();
        assert!(
            result.status == GroundingStatus::Verified || result.status == GroundingStatus::Partial,
            "Expected verified or partial, got {:?}",
            result.status
        );
        assert!(!result.evidence.is_empty());

        // Ground a claim that should NOT match
        let result = sister.ground("cats teleporting mars").unwrap();
        assert_eq!(result.status, GroundingStatus::Ungrounded);
    }

    #[test]
    fn test_evidence() {
        let mut sister = make_test_sister();
        sister.start_session("evidence_test").unwrap();
        add_test_captures(&mut sister);

        let evidence = sister.evidence("settings dark mode", 10).unwrap();
        assert!(
            !evidence.is_empty(),
            "Expected evidence for 'settings dark mode'"
        );
        assert_eq!(evidence[0].source_sister, SisterType::Vision);
    }

    #[test]
    fn test_suggest() {
        let mut sister = make_test_sister();
        sister.start_session("suggest_test").unwrap();
        add_test_captures(&mut sister);

        let suggestions = sister.suggest("error network", 5).unwrap();
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].relevance_score > 0.0);
    }

    #[test]
    fn test_queryable_list() {
        let mut sister = make_test_sister();
        sister.start_session("query_test").unwrap();
        add_test_captures(&mut sister);

        let result = sister.query(Query::list().limit(2)).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.has_more);
    }

    #[test]
    fn test_queryable_search() {
        let mut sister = make_test_sister();
        sister.start_session("search_test").unwrap();
        add_test_captures(&mut sister);

        let result = sister.search("login").unwrap();
        assert!(!result.is_empty(), "Expected search results for 'login'");
    }

    #[test]
    fn test_queryable_types() {
        let sister = make_test_sister();
        assert!(sister.supports_query("list"));
        assert!(sister.supports_query("search"));
        assert!(sister.supports_query("recent"));
        assert!(sister.supports_query("get"));
        assert!(!sister.supports_query("nonexistent"));

        let types = sister.query_types();
        assert_eq!(types.len(), 4);
    }

    #[test]
    fn test_error_bridge() {
        let err = VisionError::CaptureNotFound(42);
        let sister_err: SisterError = err.into();
        assert_eq!(sister_err.code, ErrorCode::NotFound);
        assert!(sister_err.message.contains("42"));

        let err2 = VisionError::InvalidInput("bad input".to_string());
        let sister_err2: SisterError = err2.into();
        assert_eq!(sister_err2.code, ErrorCode::InvalidInput);

        let err3 = VisionError::Embedding("model failed".to_string());
        let sister_err3: SisterError = err3.into();
        assert_eq!(sister_err3.code, ErrorCode::VisionError);
    }

    #[test]
    fn test_session_export_import() {
        let mut sister = make_test_sister();
        let sid = sister.start_session("export_test").unwrap();
        add_test_captures(&mut sister);

        // Export
        let snapshot = sister.export_session(sid).unwrap();
        assert!(snapshot.verify());
        assert_eq!(snapshot.sister_type, SisterType::Vision);

        // Import into fresh sister
        let mut sister2 = make_test_sister();
        let _imported_sid = sister2.import_session(snapshot).unwrap();
        assert!(sister2.current_session().is_some());
        assert!(sister2.store().count() > 0);
    }

    #[test]
    fn test_config_patterns() {
        // Single path config
        let config = SisterConfig::new("/tmp/test.avis");
        let sister = VisionSister::init(config).unwrap();
        assert!(sister.is_healthy());

        // Stateless config
        let config2 = SisterConfig::stateless();
        let sister2 = VisionSister::init(config2).unwrap();
        assert!(sister2.is_healthy());
    }

    #[test]
    fn test_shutdown() {
        let mut sister = make_test_sister();
        sister.start_session("shutdown_test").unwrap();
        sister.shutdown().unwrap();
        // Session should be ended after shutdown
        assert!(sister.current_session().is_none());
    }

    #[test]
    fn test_file_format() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.avis");

        let mut sister = make_test_sister();
        sister.start_session("file_test").unwrap();
        add_test_captures(&mut sister);

        // Write
        sister.write_file(&path).unwrap();

        // Read back
        let sister2 = VisionSister::read_file(&path).unwrap();
        assert_eq!(sister2.store().count(), 3);

        // Can read check
        let info = VisionSister::can_read(&path).unwrap();
        assert_eq!(info.sister_type, SisterType::Vision);
        assert_eq!(info.format_id, "AVIS");
    }
}
