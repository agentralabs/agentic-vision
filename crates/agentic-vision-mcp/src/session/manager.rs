//! Visual memory session lifecycle, file I/O, and session tracking.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use image::GenericImageView;

use agentic_vision::{
    capture_from_base64, capture_from_file, compute_diff, cosine_similarity, find_similar,
    generate_thumbnail, AvisReader, AvisWriter, CaptureSource, EmbeddingEngine, ObservationMeta,
    Rect, SimilarityMatch, VisualDiff, VisualMemoryStore, VisualObservation, EMBEDDING_DIM,
};

use crate::types::{McpError, McpResult};

const DEFAULT_AUTO_SAVE_SECS: u64 = 30;
const DEFAULT_STORAGE_BUDGET_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const DEFAULT_STORAGE_BUDGET_HORIZON_YEARS: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StorageBudgetMode {
    AutoRollup,
    Warn,
    Off,
}

impl StorageBudgetMode {
    fn from_env(name: &str) -> Self {
        let raw = read_env_string(name).unwrap_or_else(|| "auto-rollup".to_string());
        match raw.trim().to_ascii_lowercase().as_str() {
            "warn" => Self::Warn,
            "off" | "disabled" | "none" => Self::Off,
            _ => Self::AutoRollup,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::AutoRollup => "auto-rollup",
            Self::Warn => "warn",
            Self::Off => "off",
        }
    }
}

/// Manages the visual memory lifecycle, file I/O, and session state.
pub struct VisionSessionManager {
    store: VisualMemoryStore,
    engine: EmbeddingEngine,
    file_path: PathBuf,
    current_session: u32,
    dirty: bool,
    last_save: Instant,
    auto_save_interval: Duration,
    storage_budget_mode: StorageBudgetMode,
    storage_budget_max_bytes: u64,
    storage_budget_horizon_years: u32,
    storage_budget_target_fraction: f32,
    storage_budget_rollup_count: u64,
}

impl VisionSessionManager {
    /// Open or create a vision file at the given path.
    pub fn open(path: &str, model_path: Option<&str>) -> McpResult<Self> {
        let file_path = PathBuf::from(path);

        let store = if file_path.exists() {
            tracing::info!("Opening existing vision file: {}", file_path.display());
            AvisReader::read_from_file(&file_path)
                .map_err(|e| McpError::VisionError(format!("Failed to read vision file: {e}")))?
        } else {
            tracing::info!("Creating new vision file: {}", file_path.display());
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    McpError::Io(std::io::Error::other(format!(
                        "Failed to create directory {}: {e}",
                        parent.display()
                    )))
                })?;
            }
            VisualMemoryStore::new(EMBEDDING_DIM)
        };

        let current_session = store.session_count + 1;

        let engine = EmbeddingEngine::new(model_path).map_err(|e| {
            McpError::VisionError(format!("Failed to initialize embedding engine: {e}"))
        })?;

        tracing::info!(
            "Session {} started. Store has {} observations. Embedding model: {}",
            current_session,
            store.count(),
            if engine.has_model() {
                "loaded"
            } else {
                "fallback"
            }
        );

        let storage_budget_mode = StorageBudgetMode::from_env("CORTEX_STORAGE_BUDGET_MODE");
        let storage_budget_max_bytes =
            read_env_u64("CORTEX_STORAGE_BUDGET_BYTES", DEFAULT_STORAGE_BUDGET_BYTES).max(1);
        let storage_budget_horizon_years = read_env_u32(
            "CORTEX_STORAGE_BUDGET_HORIZON_YEARS",
            DEFAULT_STORAGE_BUDGET_HORIZON_YEARS,
        )
        .max(1);
        let storage_budget_target_fraction =
            read_env_f32("CORTEX_STORAGE_BUDGET_TARGET_FRACTION", 0.85).clamp(0.50, 0.99);

        Ok(Self {
            store,
            engine,
            file_path,
            current_session,
            dirty: false,
            last_save: Instant::now(),
            auto_save_interval: Duration::from_secs(DEFAULT_AUTO_SAVE_SECS),
            storage_budget_mode,
            storage_budget_max_bytes,
            storage_budget_horizon_years,
            storage_budget_target_fraction,
            storage_budget_rollup_count: 0,
        })
    }

    /// Get the visual memory store.
    pub fn store(&self) -> &VisualMemoryStore {
        &self.store
    }

    /// Current session ID.
    pub fn current_session_id(&self) -> u32 {
        self.current_session
    }

    /// Start a new session.
    pub fn start_session(&mut self, explicit_id: Option<u32>) -> McpResult<u32> {
        let session_id = explicit_id.unwrap_or(self.current_session + 1);
        self.current_session = session_id;
        self.store.session_count = self.store.session_count.max(session_id);
        tracing::info!("Started session {session_id}");
        Ok(session_id)
    }

    /// End the current session.
    pub fn end_session(&mut self) -> McpResult<u32> {
        let session_id = self.current_session;
        self.save()?;
        self.maybe_enforce_storage_budget()?;
        tracing::info!("Ended session {session_id}");
        Ok(session_id)
    }

    /// Capture an image from a file or base64 source.
    pub fn capture(
        &mut self,
        source_type: &str,
        source_data: &str,
        mime: Option<&str>,
        labels: Vec<String>,
        description: Option<String>,
        _extract_ocr: bool,
    ) -> McpResult<CaptureResult> {
        let (img, source) = match source_type {
            "file" => capture_from_file(source_data)
                .map_err(|e| McpError::VisionError(format!("Failed to capture from file: {e}")))?,
            "base64" => {
                let m = mime.unwrap_or("image/png");
                capture_from_base64(source_data, m)
                    .map_err(|e| McpError::VisionError(format!("Failed to decode base64: {e}")))?
            }
            _ => {
                return Err(McpError::InvalidParams(format!(
                    "Unsupported source type: {source_type}. Use 'file' or 'base64'."
                )));
            }
        };

        self.store_capture(img, source, labels, description)
    }

    /// Capture a screenshot and store it in visual memory.
    pub fn capture_screenshot(
        &mut self,
        region: Option<Rect>,
        labels: Vec<String>,
        description: Option<String>,
        _extract_ocr: bool,
    ) -> McpResult<CaptureResult> {
        let (img, source) = agentic_vision::capture_screenshot(region)
            .map_err(|e| McpError::VisionError(format!("Screenshot capture failed: {e}")))?;

        self.store_capture(img, source, labels, description)
    }

    /// Capture an image from the clipboard and store it in visual memory.
    pub fn capture_clipboard(
        &mut self,
        labels: Vec<String>,
        description: Option<String>,
        _extract_ocr: bool,
    ) -> McpResult<CaptureResult> {
        let (img, source) = agentic_vision::capture_clipboard()
            .map_err(|e| McpError::VisionError(format!("Clipboard capture failed: {e}")))?;

        self.store_capture(img, source, labels, description)
    }

    /// Internal: process a captured image and store it as an observation.
    fn store_capture(
        &mut self,
        img: image::DynamicImage,
        source: CaptureSource,
        labels: Vec<String>,
        description: Option<String>,
    ) -> McpResult<CaptureResult> {
        let (orig_w, orig_h) = img.dimensions();
        let thumbnail = generate_thumbnail(&img);
        let thumb_img = image::load_from_memory(&thumbnail)
            .map_err(|e| McpError::VisionError(format!("Failed to load thumbnail: {e}")))?;
        let (thumb_w, thumb_h) = thumb_img.dimensions();

        let embedding = self
            .engine
            .embed(&img)
            .map_err(|e| McpError::VisionError(format!("Embedding failed: {e}")))?;

        let sanitized_labels: Vec<String> = labels
            .into_iter()
            .map(|v| sanitize_metadata_text(&v))
            .collect();
        let sanitized_description = description.map(|d| sanitize_metadata_text(&d));
        let quality_score = compute_quality_score(
            orig_w,
            orig_h,
            sanitized_labels.len(),
            sanitized_description.is_some(),
            self.engine.has_model(),
        );

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let obs = VisualObservation {
            id: 0, // assigned by store
            timestamp: now,
            session_id: self.current_session,
            source,
            embedding,
            thumbnail,
            metadata: ObservationMeta {
                width: thumb_w,
                height: thumb_h,
                original_width: orig_w,
                original_height: orig_h,
                labels: sanitized_labels,
                description: sanitized_description,
                quality_score,
            },
            memory_link: None,
        };

        let id = self.store.add(obs);
        self.dirty = true;
        self.maybe_auto_save()?;
        self.maybe_enforce_storage_budget()?;

        Ok(CaptureResult {
            capture_id: id,
            timestamp: now,
            width: orig_w,
            height: orig_h,
            embedding_dims: EMBEDDING_DIM,
            quality_score,
        })
    }

    /// Compare two captures by cosine similarity.
    pub fn compare(&self, id_a: u64, id_b: u64) -> McpResult<f32> {
        let a = self
            .store
            .get(id_a)
            .ok_or(McpError::CaptureNotFound(id_a))?;
        let b = self
            .store
            .get(id_b)
            .ok_or(McpError::CaptureNotFound(id_b))?;

        Ok(cosine_similarity(&a.embedding, &b.embedding))
    }

    /// Find similar captures.
    pub fn find_similar(
        &self,
        capture_id: u64,
        top_k: usize,
        min_similarity: f32,
    ) -> McpResult<Vec<SimilarityMatch>> {
        let obs = self
            .store
            .get(capture_id)
            .ok_or(McpError::CaptureNotFound(capture_id))?;

        let mut matches = find_similar(
            &obs.embedding,
            &self.store.observations,
            top_k + 1,
            min_similarity,
        );
        // Remove self from results
        matches.retain(|m| m.id != capture_id);
        matches.truncate(top_k);
        Ok(matches)
    }

    /// Find similar by raw embedding.
    pub fn find_similar_by_embedding(
        &self,
        embedding: &[f32],
        top_k: usize,
        min_similarity: f32,
    ) -> Vec<SimilarityMatch> {
        find_similar(embedding, &self.store.observations, top_k, min_similarity)
    }

    /// Compute visual diff between two captures.
    pub fn diff(&self, id_a: u64, id_b: u64) -> McpResult<VisualDiff> {
        let a = self
            .store
            .get(id_a)
            .ok_or(McpError::CaptureNotFound(id_a))?;
        let b = self
            .store
            .get(id_b)
            .ok_or(McpError::CaptureNotFound(id_b))?;

        let img_a = image::load_from_memory(&a.thumbnail)
            .map_err(|e| McpError::VisionError(format!("Failed to load thumbnail A: {e}")))?;
        let img_b = image::load_from_memory(&b.thumbnail)
            .map_err(|e| McpError::VisionError(format!("Failed to load thumbnail B: {e}")))?;

        compute_diff(id_a, id_b, &img_a, &img_b)
            .map_err(|e| McpError::VisionError(format!("Diff failed: {e}")))
    }

    /// Link a capture to a memory node.
    pub fn link(&mut self, capture_id: u64, memory_node_id: u64) -> McpResult<()> {
        let obs = self
            .store
            .get_mut(capture_id)
            .ok_or(McpError::CaptureNotFound(capture_id))?;
        obs.memory_link = Some(memory_node_id);
        self.dirty = true;
        Ok(())
    }

    /// Save to file.
    pub fn save(&mut self) -> McpResult<()> {
        if !self.dirty {
            return Ok(());
        }

        AvisWriter::write_to_file(&self.store, &self.file_path)
            .map_err(|e| McpError::VisionError(format!("Failed to write vision file: {e}")))?;

        self.dirty = false;
        self.last_save = Instant::now();
        tracing::debug!("Saved vision file: {}", self.file_path.display());
        Ok(())
    }

    fn maybe_auto_save(&mut self) -> McpResult<()> {
        if self.dirty && self.last_save.elapsed() >= self.auto_save_interval {
            self.save()?;
        }
        Ok(())
    }

    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn storage_budget_status(&self) -> VisionStorageBudgetStatus {
        let current_size = self.current_file_size_bytes();
        let projected = self.projected_file_size_bytes(current_size);
        let over_budget = current_size > self.storage_budget_max_bytes
            || projected
                .map(|v| v > self.storage_budget_max_bytes)
                .unwrap_or(false);

        VisionStorageBudgetStatus {
            mode: self.storage_budget_mode.as_str().to_string(),
            max_bytes: self.storage_budget_max_bytes,
            horizon_years: self.storage_budget_horizon_years,
            target_fraction: self.storage_budget_target_fraction,
            current_size_bytes: current_size,
            projected_size_bytes: projected,
            over_budget,
            rollup_count: self.storage_budget_rollup_count,
        }
    }

    fn maybe_enforce_storage_budget(&mut self) -> McpResult<()> {
        if self.storage_budget_mode == StorageBudgetMode::Off {
            return Ok(());
        }

        if self.current_file_size_bytes() == 0 && self.dirty {
            self.save()?;
        }

        let current_size = self.current_file_size_bytes();
        if current_size == 0 {
            return Ok(());
        }
        let projected = self.projected_file_size_bytes(current_size);
        let over_current = current_size > self.storage_budget_max_bytes;
        let over_projected = projected
            .map(|v| v > self.storage_budget_max_bytes)
            .unwrap_or(false);
        if !over_current && !over_projected {
            return Ok(());
        }

        if self.storage_budget_mode == StorageBudgetMode::Warn {
            tracing::warn!(
                "AVIS storage budget warning: current={} projected={:?} limit={}",
                current_size,
                projected,
                self.storage_budget_max_bytes
            );
            return Ok(());
        }

        let target_bytes = ((self.storage_budget_max_bytes as f64
            * self.storage_budget_target_fraction as f64)
            .round() as u64)
            .max(1);
        let mut pruned = 0usize;

        loop {
            let current = self.current_file_size_bytes();
            if current <= target_bytes {
                break;
            }
            let Some(idx) = self.select_prune_candidate() else {
                break;
            };
            self.store.observations.remove(idx);
            self.store.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            self.dirty = true;
            self.save()?;
            pruned = pruned.saturating_add(1);
        }

        if pruned > 0 {
            self.storage_budget_rollup_count = self
                .storage_budget_rollup_count
                .saturating_add(pruned as u64);
            tracing::info!(
                "AVIS storage budget rollup: pruned={} current_size={} limit={}",
                pruned,
                self.current_file_size_bytes(),
                self.storage_budget_max_bytes
            );
        } else {
            tracing::warn!(
                "AVIS storage budget exceeded but no prune candidate available (current={} projected={:?} limit={})",
                current_size,
                projected,
                self.storage_budget_max_bytes
            );
        }

        Ok(())
    }

    fn select_prune_candidate(&self) -> Option<usize> {
        // Prefer non-linked captures from completed sessions (oldest first).
        let mut best: Option<(usize, u64, f32)> = None;
        for (idx, obs) in self.store.observations.iter().enumerate() {
            if obs.session_id >= self.current_session || obs.memory_link.is_some() {
                continue;
            }
            let score = obs.metadata.quality_score;
            match best {
                None => best = Some((idx, obs.timestamp, score)),
                Some((_, ts, q)) => {
                    if obs.timestamp < ts || (obs.timestamp == ts && score < q) {
                        best = Some((idx, obs.timestamp, score));
                    }
                }
            }
        }
        if let Some((idx, _, _)) = best {
            return Some(idx);
        }

        // Fallback: oldest capture in completed sessions.
        self.store
            .observations
            .iter()
            .enumerate()
            .filter(|(_, obs)| obs.session_id < self.current_session)
            .min_by_key(|(_, obs)| obs.timestamp)
            .map(|(idx, _)| idx)
    }

    fn current_file_size_bytes(&self) -> u64 {
        std::fs::metadata(&self.file_path)
            .map(|m| m.len())
            .unwrap_or(0)
    }

    fn projected_file_size_bytes(&self, current_size: u64) -> Option<u64> {
        if current_size == 0 || self.store.observations.len() < 2 {
            return None;
        }
        let mut min_ts = u64::MAX;
        let mut max_ts = 0u64;
        for obs in &self.store.observations {
            min_ts = min_ts.min(obs.timestamp);
            max_ts = max_ts.max(obs.timestamp);
        }
        if min_ts == u64::MAX || max_ts <= min_ts {
            return None;
        }
        let span_secs = (max_ts - min_ts).max(7 * 24 * 3600) as f64;
        let per_sec = current_size as f64 / span_secs;
        let horizon_secs = (self.storage_budget_horizon_years as f64) * 365.25 * 24.0 * 3600.0;
        let projected = (per_sec * horizon_secs).round();
        Some(projected.max(0.0).min(u64::MAX as f64) as u64)
    }
}

impl Drop for VisionSessionManager {
    fn drop(&mut self) {
        if self.dirty {
            if let Err(e) = self.save() {
                tracing::error!("Failed to save on drop: {e}");
            }
        }
    }
}

/// Result of a capture operation.
pub struct CaptureResult {
    pub capture_id: u64,
    pub timestamp: u64,
    pub width: u32,
    pub height: u32,
    pub embedding_dims: u32,
    pub quality_score: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VisionStorageBudgetStatus {
    pub mode: String,
    pub max_bytes: u64,
    pub horizon_years: u32,
    pub target_fraction: f32,
    pub current_size_bytes: u64,
    pub projected_size_bytes: Option<u64>,
    pub over_budget: bool,
    pub rollup_count: u64,
}

fn read_env_string(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|v| v.trim().to_string())
}

fn read_env_u64(name: &str, default_value: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default_value)
}

fn read_env_u32(name: &str, default_value: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(default_value)
}

fn read_env_f32(name: &str, default_value: f32) -> f32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(default_value)
}

fn sanitize_metadata_text(raw: &str) -> String {
    raw.split_whitespace()
        .map(|token| {
            if looks_like_email(token) {
                "[redacted-email]".to_string()
            } else if looks_like_secret(token) {
                "[redacted-secret]".to_string()
            } else if looks_like_local_path(token) {
                "[redacted-path]".to_string()
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn looks_like_email(token: &str) -> bool {
    token.contains('@') && token.contains('.')
}

fn looks_like_secret(token: &str) -> bool {
    let t = token.trim_matches(|c: char| ",.;:()[]{}<>\"'".contains(c));
    if t.starts_with("sk-") && t.len() >= 12 {
        return true;
    }
    if t.len() >= 32 && t.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }
    t.to_ascii_lowercase().contains("token=") && t.len() >= 16
}

fn looks_like_local_path(token: &str) -> bool {
    token.starts_with("/Users/")
        || token.starts_with("/home/")
        || token.starts_with("C:\\")
        || token.starts_with("D:\\")
}

fn compute_quality_score(
    width: u32,
    height: u32,
    label_count: usize,
    has_description: bool,
    model_available: bool,
) -> f32 {
    let px = width as f32 * height as f32;
    let resolution_score = (px / (1280.0 * 720.0)).clamp(0.0, 1.0);
    let label_score = (label_count as f32 / 6.0).clamp(0.0, 1.0);
    let description_score = if has_description { 1.0 } else { 0.0 };
    let model_score = if model_available { 1.0 } else { 0.35 };

    // Weighted blend focused on actionable retrieval quality.
    (0.35 * resolution_score + 0.20 * label_score + 0.20 * description_score + 0.25 * model_score)
        .clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_obs(session_id: u32, timestamp: u64, linked: bool) -> VisualObservation {
        VisualObservation {
            id: 0,
            timestamp,
            session_id,
            source: CaptureSource::Clipboard,
            embedding: vec![0.0; EMBEDDING_DIM as usize],
            thumbnail: vec![1u8; 1024],
            metadata: ObservationMeta {
                width: 64,
                height: 64,
                original_width: 512,
                original_height: 512,
                labels: vec!["test".to_string()],
                description: Some("observation".to_string()),
                quality_score: 0.4,
            },
            memory_link: if linked { Some(42) } else { None },
        }
    }

    #[test]
    fn budget_projection_available_with_timeline() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("vision-projection.avis");
        let mut manager =
            VisionSessionManager::open(path.to_str().expect("path"), None).expect("open");

        manager.store.add(make_obs(1, 1_700_000_000, false));
        manager
            .store
            .add(make_obs(1, 1_700_000_000 + 15 * 24 * 3600, false));
        manager.dirty = true;
        manager.save().expect("save");

        let size = manager.current_file_size_bytes();
        let projected = manager.projected_file_size_bytes(size);
        assert!(size > 0);
        assert!(projected.is_some());
    }

    #[test]
    fn budget_auto_rollup_prunes_completed_sessions() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("vision-rollup.avis");
        let mut manager =
            VisionSessionManager::open(path.to_str().expect("path"), None).expect("open");

        manager.store.add(make_obs(1, 1_700_000_000, false));
        manager.store.add(make_obs(1, 1_700_000_001, false));
        manager.start_session(Some(2)).expect("session");
        manager.dirty = true;
        manager.save().expect("save");

        let before = manager.store.count();
        manager.storage_budget_mode = StorageBudgetMode::AutoRollup;
        manager.storage_budget_max_bytes = 1;
        manager.storage_budget_target_fraction = 0.5;

        manager
            .maybe_enforce_storage_budget()
            .expect("enforce budget");

        assert!(manager.store.count() < before);
        assert!(manager.storage_budget_rollup_count >= 1);
    }
}
