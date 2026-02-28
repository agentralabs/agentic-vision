//! Forensics Inventions (20–22): Visual Diff Engine, Visual Anomaly Detection,
//! Visual Regression Testing.
//!
//! 11 MCP tools that investigate and audit visual captures.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::session::manager::VisionSessionManager;
use crate::types::error::{McpError, McpResult};
use crate::types::response::{ToolCallResult, ToolDefinition};

// =========================================================================
// Types
// =========================================================================

/// Type of forensic change detected between captures.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ForensicChangeType {
    ContentAdded,
    ContentRemoved,
    ContentModified,
    LayoutShifted,
    ColorChanged,
    SizeChanged,
    QualityChanged,
    LabelChanged,
    NoChange,
}

impl ForensicChangeType {
    fn label(&self) -> &str {
        match self {
            Self::ContentAdded => "content_added",
            Self::ContentRemoved => "content_removed",
            Self::ContentModified => "content_modified",
            Self::LayoutShifted => "layout_shifted",
            Self::ColorChanged => "color_changed",
            Self::SizeChanged => "size_changed",
            Self::QualityChanged => "quality_changed",
            Self::LabelChanged => "label_changed",
            Self::NoChange => "no_change",
        }
    }
}

/// Severity of a forensic finding.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ForensicSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl ForensicSeverity {
    fn label(&self) -> &str {
        match self {
            Self::Info => "info",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    fn from_score(score: f64) -> Self {
        if score > 0.8 {
            Self::Critical
        } else if score > 0.6 {
            Self::High
        } else if score > 0.4 {
            Self::Medium
        } else if score > 0.2 {
            Self::Low
        } else {
            Self::Info
        }
    }
}

/// A forensic finding entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicFinding {
    pub change_type: String,
    pub severity: String,
    pub description: String,
    pub evidence_score: f64,
    pub affected_dimension: String,
}

/// Anomaly detection result for a single capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    pub capture_id: u64,
    pub is_anomaly: bool,
    pub anomaly_score: f64,
    pub z_scores: HashMap<String, f64>,
    pub contributing_factors: Vec<String>,
}

/// Statistical baseline for a capture series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalBaseline {
    pub sample_count: usize,
    pub feature_stats: HashMap<String, FeatureStats>,
    pub created_at: u64,
}

/// Statistics for a single feature dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStats {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub iqr: f64,
    pub q1: f64,
    pub q3: f64,
}

/// Regression test result.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RegressionStatus {
    Pass,
    Warning,
    Fail,
    Error,
}

impl RegressionStatus {
    fn label(&self) -> &str {
        match self {
            Self::Pass => "pass",
            Self::Warning => "warning",
            Self::Fail => "fail",
            Self::Error => "error",
        }
    }
}

/// Type of anomaly pattern detected.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnomalyPatternType {
    SingleSpike,
    GradualDrift,
    OscillatingValues,
    StepChange,
    Clustering,
    Trending,
}

impl AnomalyPatternType {
    fn label(&self) -> &str {
        match self {
            Self::SingleSpike => "single_spike",
            Self::GradualDrift => "gradual_drift",
            Self::OscillatingValues => "oscillating_values",
            Self::StepChange => "step_change",
            Self::Clustering => "clustering",
            Self::Trending => "trending",
        }
    }
}

// =========================================================================
// Helpers
// =========================================================================

fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn word_overlap(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let a_words: std::collections::HashSet<&str> = a_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
        .collect();
    let b_words: std::collections::HashSet<&str> = b_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
        .collect();
    if a_words.is_empty() || b_words.is_empty() {
        return 0.0;
    }
    let intersection = a_words.intersection(&b_words).count();
    intersection as f64 / a_words.len().max(b_words.len()) as f64
}

/// Compute cosine similarity between two embedding vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let min_len = a.len().min(b.len());
    if min_len == 0 {
        return 0.0;
    }
    let mut dot = 0.0f64;
    let mut norm_a = 0.0f64;
    let mut norm_b = 0.0f64;
    for i in 0..min_len {
        let va = a[i] as f64;
        let vb = b[i] as f64;
        dot += va * vb;
        norm_a += va * va;
        norm_b += vb * vb;
    }
    let denom = (norm_a.sqrt()) * (norm_b.sqrt());
    if denom < 1e-10 {
        return 0.0;
    }
    (dot / denom).max(-1.0).min(1.0)
}

/// Compute embedding distance (1 - cosine_similarity).
fn embedding_distance(a: &[f32], b: &[f32]) -> f64 {
    1.0 - cosine_similarity(a, b)
}

/// Extract a named feature set from an observation for anomaly detection.
fn extract_forensic_features(
    embedding: &[f32],
    width: u32,
    height: u32,
    labels: &[String],
    quality_score: f32,
    description: &Option<String>,
) -> HashMap<String, f64> {
    let mut features = HashMap::new();

    // Dimension features
    features.insert("width".to_string(), width as f64);
    features.insert("height".to_string(), height as f64);
    features.insert(
        "aspect_ratio".to_string(),
        if height > 0 {
            width as f64 / height as f64
        } else {
            1.0
        },
    );
    features.insert("pixel_count".to_string(), (width as f64) * (height as f64));

    // Quality
    features.insert("quality_score".to_string(), quality_score as f64);

    // Label count
    features.insert("label_count".to_string(), labels.len() as f64);

    // Description length
    let desc_len = description.as_ref().map(|d| d.len()).unwrap_or(0);
    features.insert("description_length".to_string(), desc_len as f64);

    // Embedding statistics
    let dim = embedding.len();
    if dim > 0 {
        let mean: f64 = embedding.iter().map(|&v| v as f64).sum::<f64>() / dim as f64;
        let variance: f64 = embedding
            .iter()
            .map(|&v| {
                let d = v as f64 - mean;
                d * d
            })
            .sum::<f64>()
            / dim as f64;
        let std_dev = variance.sqrt();
        let max_val = embedding
            .iter()
            .map(|&v| v as f64)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_val = embedding
            .iter()
            .map(|&v| v as f64)
            .fold(f64::INFINITY, f64::min);

        features.insert("embedding_mean".to_string(), mean);
        features.insert("embedding_std".to_string(), std_dev);
        features.insert("embedding_max".to_string(), max_val);
        features.insert("embedding_min".to_string(), min_val);
        features.insert("embedding_range".to_string(), max_val - min_val);
        features.insert("embedding_dim".to_string(), dim as f64);

        // Energy (sum of squares)
        let energy: f64 = embedding.iter().map(|&v| (v as f64).powi(2)).sum::<f64>() / dim as f64;
        features.insert("embedding_energy".to_string(), energy);

        // Zero-crossing rate
        let mut crossings = 0u64;
        for i in 1..dim {
            if (embedding[i] >= 0.0) != (embedding[i - 1] >= 0.0) {
                crossings += 1;
            }
        }
        features.insert(
            "zero_crossing_rate".to_string(),
            crossings as f64 / (dim - 1).max(1) as f64,
        );

        // Spectral centroid approximation
        let mut weighted_sum = 0.0f64;
        let mut total_magnitude = 0.0f64;
        for (i, &v) in embedding.iter().enumerate() {
            let mag = (v as f64).abs();
            weighted_sum += i as f64 * mag;
            total_magnitude += mag;
        }
        let spectral_centroid = if total_magnitude > 1e-10 {
            weighted_sum / total_magnitude / dim as f64
        } else {
            0.5
        };
        features.insert("spectral_centroid".to_string(), spectral_centroid);

        // Kurtosis
        if std_dev > 1e-10 {
            let kurtosis: f64 = embedding
                .iter()
                .map(|&v| ((v as f64 - mean) / std_dev).powi(4))
                .sum::<f64>()
                / dim as f64
                - 3.0;
            features.insert("embedding_kurtosis".to_string(), kurtosis);
        }

        // Skewness
        if std_dev > 1e-10 {
            let skewness: f64 = embedding
                .iter()
                .map(|&v| ((v as f64 - mean) / std_dev).powi(3))
                .sum::<f64>()
                / dim as f64;
            features.insert("embedding_skewness".to_string(), skewness);
        }
    }

    features
}

/// Compute statistical summary for a slice of f64 values.
fn compute_feature_stats(values: &[f64]) -> FeatureStats {
    let n = values.len();
    if n == 0 {
        return FeatureStats {
            mean: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            median: 0.0,
            iqr: 0.0,
            q1: 0.0,
            q3: 0.0,
        };
    }

    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let std_dev = variance.sqrt();
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let median = if n % 2 == 0 {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    };

    let q1_idx = n / 4;
    let q3_idx = (3 * n) / 4;
    let q1 = sorted[q1_idx.min(n - 1)];
    let q3 = sorted[q3_idx.min(n - 1)];
    let iqr = q3 - q1;

    FeatureStats {
        mean,
        std_dev,
        min,
        max,
        median,
        iqr,
        q1,
        q3,
    }
}

/// Compute z-score of a value given mean and std_dev.
fn z_score(value: f64, mean: f64, std_dev: f64) -> f64 {
    if std_dev < 1e-10 {
        if (value - mean).abs() < 1e-10 {
            return 0.0;
        }
        return 3.0; // Cannot compute z-score; flag as anomalous
    }
    (value - mean) / std_dev
}

/// Check if a value is an outlier using the IQR method.
fn is_iqr_outlier(value: f64, q1: f64, q3: f64, iqr: f64, factor: f64) -> bool {
    let lower = q1 - factor * iqr;
    let upper = q3 + factor * iqr;
    value < lower || value > upper
}

/// Detect change type between two observations based on metadata comparison.
fn detect_change_types(
    desc_a: &Option<String>,
    desc_b: &Option<String>,
    labels_a: &[String],
    labels_b: &[String],
    width_a: u32,
    height_a: u32,
    width_b: u32,
    height_b: u32,
    quality_a: f32,
    quality_b: f32,
    embedding_dist: f64,
) -> Vec<ForensicFinding> {
    let mut findings = Vec::new();

    // Size change
    if width_a != width_b || height_a != height_b {
        let size_change = ((width_b as f64 * height_b as f64)
            / (width_a as f64 * height_a as f64).max(1.0)
            - 1.0)
            .abs();
        findings.push(ForensicFinding {
            change_type: ForensicChangeType::SizeChanged.label().to_string(),
            severity: ForensicSeverity::from_score(size_change.min(1.0))
                .label()
                .to_string(),
            description: format!(
                "Dimensions changed from {}x{} to {}x{}",
                width_a, height_a, width_b, height_b
            ),
            evidence_score: size_change.min(1.0),
            affected_dimension: "dimensions".to_string(),
        });
    }

    // Quality change
    let quality_diff = (quality_b - quality_a).abs();
    if quality_diff > 0.1 {
        findings.push(ForensicFinding {
            change_type: ForensicChangeType::QualityChanged.label().to_string(),
            severity: ForensicSeverity::from_score(quality_diff as f64)
                .label()
                .to_string(),
            description: format!(
                "Quality score changed from {:.2} to {:.2}",
                quality_a, quality_b
            ),
            evidence_score: quality_diff as f64,
            affected_dimension: "quality".to_string(),
        });
    }

    // Label changes
    let labels_set_a: std::collections::HashSet<&String> = labels_a.iter().collect();
    let labels_set_b: std::collections::HashSet<&String> = labels_b.iter().collect();
    let added: Vec<&&String> = labels_set_b.difference(&labels_set_a).collect();
    let removed: Vec<&&String> = labels_set_a.difference(&labels_set_b).collect();

    if !added.is_empty() {
        findings.push(ForensicFinding {
            change_type: ForensicChangeType::ContentAdded.label().to_string(),
            severity: ForensicSeverity::Low.label().to_string(),
            description: format!("Labels added: {:?}", added),
            evidence_score: (added.len() as f64 * 0.2).min(1.0),
            affected_dimension: "labels".to_string(),
        });
    }
    if !removed.is_empty() {
        findings.push(ForensicFinding {
            change_type: ForensicChangeType::ContentRemoved.label().to_string(),
            severity: ForensicSeverity::Medium.label().to_string(),
            description: format!("Labels removed: {:?}", removed),
            evidence_score: (removed.len() as f64 * 0.25).min(1.0),
            affected_dimension: "labels".to_string(),
        });
    }

    // Description change
    let desc_similarity = match (desc_a, desc_b) {
        (Some(da), Some(db)) => word_overlap(da, db),
        (None, None) => 1.0,
        _ => 0.0,
    };
    if desc_similarity < 0.7 {
        let change_score = 1.0 - desc_similarity;
        findings.push(ForensicFinding {
            change_type: ForensicChangeType::ContentModified.label().to_string(),
            severity: ForensicSeverity::from_score(change_score)
                .label()
                .to_string(),
            description: format!(
                "Description changed (similarity: {:.1}%)",
                desc_similarity * 100.0
            ),
            evidence_score: change_score,
            affected_dimension: "description".to_string(),
        });
    }

    // Embedding-based visual change (color/layout proxy)
    if embedding_dist > 0.1 {
        let change_type = if embedding_dist > 0.5 {
            ForensicChangeType::LayoutShifted
        } else {
            ForensicChangeType::ColorChanged
        };
        findings.push(ForensicFinding {
            change_type: change_type.label().to_string(),
            severity: ForensicSeverity::from_score(embedding_dist)
                .label()
                .to_string(),
            description: format!(
                "Visual content changed (embedding distance: {:.3})",
                embedding_dist
            ),
            evidence_score: embedding_dist.min(1.0),
            affected_dimension: "visual_content".to_string(),
        });
    }

    if findings.is_empty() {
        findings.push(ForensicFinding {
            change_type: ForensicChangeType::NoChange.label().to_string(),
            severity: ForensicSeverity::Info.label().to_string(),
            description: "No significant changes detected".to_string(),
            evidence_score: 0.0,
            affected_dimension: "none".to_string(),
        });
    }

    // Sort by evidence_score descending
    findings.sort_by(|a, b| {
        b.evidence_score
            .partial_cmp(&a.evidence_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    findings
}

/// Detect anomaly pattern type from a series of anomaly scores.
fn detect_anomaly_pattern(scores: &[f64]) -> AnomalyPatternType {
    let n = scores.len();
    if n < 3 {
        return AnomalyPatternType::SingleSpike;
    }

    // Check for step change: large jump between consecutive values
    let mut max_jump = 0.0f64;
    let mut max_jump_idx = 0;
    for i in 1..n {
        let jump = (scores[i] - scores[i - 1]).abs();
        if jump > max_jump {
            max_jump = jump;
            max_jump_idx = i;
        }
    }

    // Check for trend: monotonic increase/decrease
    let mut increasing = 0;
    let mut decreasing = 0;
    for i in 1..n {
        if scores[i] > scores[i - 1] + 0.01 {
            increasing += 1;
        } else if scores[i] < scores[i - 1] - 0.01 {
            decreasing += 1;
        }
    }
    let trend_ratio = increasing.max(decreasing) as f64 / (n - 1) as f64;

    // Check for oscillation: alternating increases and decreases
    let mut sign_changes = 0;
    for i in 2..n {
        let d1 = scores[i - 1] - scores[i - 2];
        let d2 = scores[i] - scores[i - 1];
        if (d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0) {
            sign_changes += 1;
        }
    }
    let oscillation_ratio = sign_changes as f64 / (n - 2).max(1) as f64;

    // Count spikes (values > 2 std devs from mean)
    let mean = scores.iter().sum::<f64>() / n as f64;
    let std_dev = (scores.iter().map(|&s| (s - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
    let spike_count = scores
        .iter()
        .filter(|&&s| (s - mean).abs() > 2.0 * std_dev)
        .count();

    // Check for clustering: groups of anomalies
    let mut cluster_runs = 0;
    let mut in_cluster = false;
    let threshold = mean + std_dev;
    for &s in scores {
        if s > threshold {
            if !in_cluster {
                cluster_runs += 1;
                in_cluster = true;
            }
        } else {
            in_cluster = false;
        }
    }

    // Classify pattern — check more specific patterns first.
    // A single spike is when exactly one value jumps and immediately returns.
    // A step change is when values jump and STAY at the new level.
    if oscillation_ratio > 0.6 {
        AnomalyPatternType::OscillatingValues
    } else if trend_ratio > 0.7 {
        AnomalyPatternType::Trending
    } else if spike_count == 1 {
        // Exactly one outlier value — spike wins over step change
        AnomalyPatternType::SingleSpike
    } else if max_jump > std_dev * 3.0 && max_jump_idx > n / 4 && max_jump_idx < 3 * n / 4 {
        // Large jump in the middle that persists (not a single spike)
        AnomalyPatternType::StepChange
    } else if cluster_runs > 2 {
        AnomalyPatternType::Clustering
    } else {
        let first_half_mean = scores[..n / 2].iter().sum::<f64>() / (n / 2) as f64;
        let second_half_mean = scores[n / 2..].iter().sum::<f64>() / (n - n / 2) as f64;
        if (second_half_mean - first_half_mean).abs() > std_dev.max(1e-10) {
            AnomalyPatternType::GradualDrift
        } else {
            AnomalyPatternType::SingleSpike
        }
    }
}

// =========================================================================
// INVENTION 20: Visual Diff Engine — 4 tools
// =========================================================================

// -- vision_forensic_diff -------------------------------------------------

pub fn definition_vision_forensic_diff() -> ToolDefinition {
    ToolDefinition {
        name: "vision_forensic_diff".to_string(),
        description: Some(
            "Perform deep forensic diff between two captures with pixel-level analysis and noise tolerance"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["capture_a", "capture_b"],
            "properties": {
                "capture_a": { "type": "number", "description": "First capture ID (before)" },
                "capture_b": { "type": "number", "description": "Second capture ID (after)" },
                "noise_threshold": { "type": "number", "description": "Noise tolerance (0.0-1.0)", "default": 0.05 }
            }
        }),
    }
}

pub async fn execute_vision_forensic_diff(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_a: u64,
        capture_b: u64,
        #[serde(default = "default_noise_threshold")]
        noise_threshold: f64,
    }
    fn default_noise_threshold() -> f64 {
        0.05
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs_a = store
        .observations
        .iter()
        .find(|o| o.id == p.capture_a)
        .ok_or(McpError::CaptureNotFound(p.capture_a))?;
    let obs_b = store
        .observations
        .iter()
        .find(|o| o.id == p.capture_b)
        .ok_or(McpError::CaptureNotFound(p.capture_b))?;

    // Embedding-level diff
    let emb_dist = embedding_distance(&obs_a.embedding, &obs_b.embedding);
    let cos_sim = cosine_similarity(&obs_a.embedding, &obs_b.embedding);

    // Per-dimension diff analysis
    let min_dim = obs_a.embedding.len().min(obs_b.embedding.len());
    let noise_thresh = p.noise_threshold.max(0.0).min(1.0);
    let mut changed_dimensions = 0u32;
    let mut total_abs_diff = 0.0f64;
    let mut max_diff = 0.0f64;
    let mut max_diff_dim = 0usize;
    let mut region_diffs = [0.0f64; 4]; // 4 quadrants

    for i in 0..min_dim {
        let diff = (obs_a.embedding[i] as f64 - obs_b.embedding[i] as f64).abs();
        if diff > noise_thresh {
            changed_dimensions += 1;
        }
        total_abs_diff += diff;
        if diff > max_diff {
            max_diff = diff;
            max_diff_dim = i;
        }
        let quadrant = (i * 4) / min_dim.max(1);
        let quadrant = quadrant.min(3);
        region_diffs[quadrant] += diff;
    }

    // Normalize region diffs
    let quarter_len = (min_dim / 4).max(1) as f64;
    for rd in &mut region_diffs {
        *rd /= quarter_len;
    }

    let avg_diff = if min_dim > 0 {
        total_abs_diff / min_dim as f64
    } else {
        0.0
    };

    // Detect forensic change types
    let findings = detect_change_types(
        &obs_a.metadata.description,
        &obs_b.metadata.description,
        &obs_a.metadata.labels,
        &obs_b.metadata.labels,
        obs_a.metadata.width,
        obs_a.metadata.height,
        obs_b.metadata.width,
        obs_b.metadata.height,
        obs_a.metadata.quality_score,
        obs_b.metadata.quality_score,
        emb_dist,
    );

    let overall_change_score = findings
        .iter()
        .map(|f| f.evidence_score)
        .fold(0.0f64, f64::max);

    let verdict = if overall_change_score < 0.05 {
        "identical"
    } else if overall_change_score < 0.15 {
        "minor_differences"
    } else if overall_change_score < 0.4 {
        "moderate_differences"
    } else if overall_change_score < 0.7 {
        "significant_differences"
    } else {
        "major_differences"
    };

    Ok(ToolCallResult::json(&json!({
        "capture_a": p.capture_a,
        "capture_b": p.capture_b,
        "verdict": verdict,
        "overall_change_score": (overall_change_score * 100.0).round() / 100.0,
        "embedding_analysis": {
            "cosine_similarity": (cos_sim * 10000.0).round() / 10000.0,
            "embedding_distance": (emb_dist * 10000.0).round() / 10000.0,
            "changed_dimensions": changed_dimensions,
            "total_dimensions": min_dim,
            "change_ratio": if min_dim > 0 { (changed_dimensions as f64 / min_dim as f64 * 100.0).round() / 100.0 } else { 0.0 },
            "avg_absolute_diff": (avg_diff * 10000.0).round() / 10000.0,
            "max_diff": (max_diff * 10000.0).round() / 10000.0,
            "max_diff_dimension": max_diff_dim,
        },
        "region_analysis": {
            "top_left_diff": (region_diffs[0] * 1000.0).round() / 1000.0,
            "top_right_diff": (region_diffs[1] * 1000.0).round() / 1000.0,
            "bottom_left_diff": (region_diffs[2] * 1000.0).round() / 1000.0,
            "bottom_right_diff": (region_diffs[3] * 1000.0).round() / 1000.0,
        },
        "forensic_findings": findings.iter().map(|f| json!({
            "change_type": f.change_type,
            "severity": f.severity,
            "description": f.description,
            "evidence_score": (f.evidence_score * 100.0).round() / 100.0,
            "affected_dimension": f.affected_dimension,
        })).collect::<Vec<_>>(),
        "noise_threshold": noise_thresh,
        "time_delta_seconds": obs_b.timestamp.saturating_sub(obs_a.timestamp),
    })))
}

// -- vision_forensic_timeline ---------------------------------------------

pub fn definition_vision_forensic_timeline() -> ToolDefinition {
    ToolDefinition {
        name: "vision_forensic_timeline".to_string(),
        description: Some("Build forensic timeline of all visual changes in a session".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "number", "description": "Session to analyze (default: all)" },
                "min_change_score": { "type": "number", "description": "Min change score to include (0.0-1.0)", "default": 0.05 },
                "max_entries": { "type": "number", "description": "Max timeline entries", "default": 50 }
            }
        }),
    }
}

pub async fn execute_vision_forensic_timeline(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        session_id: Option<u32>,
        #[serde(default = "default_min_change")]
        min_change_score: f64,
        #[serde(default = "default_max_entries")]
        max_entries: usize,
    }
    fn default_min_change() -> f64 {
        0.05
    }
    fn default_max_entries() -> usize {
        50
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let mut obs_list: Vec<_> = store
        .observations
        .iter()
        .filter(|o| match p.session_id {
            Some(sid) => o.session_id == sid,
            None => true,
        })
        .collect();
    obs_list.sort_by_key(|o| o.timestamp);

    if obs_list.len() < 2 {
        return Ok(ToolCallResult::json(&json!({
            "timeline": [],
            "total_events": 0,
            "summary": "Insufficient captures for timeline analysis",
        })));
    }

    let mut timeline: Vec<Value> = Vec::new();
    let mut total_change = 0.0f64;
    let mut max_change = 0.0f64;
    let mut change_types_count: HashMap<String, usize> = HashMap::new();

    for i in 1..obs_list.len() {
        let a = obs_list[i - 1];
        let b = obs_list[i];
        let emb_dist = embedding_distance(&a.embedding, &b.embedding);

        let findings = detect_change_types(
            &a.metadata.description,
            &b.metadata.description,
            &a.metadata.labels,
            &b.metadata.labels,
            a.metadata.width,
            a.metadata.height,
            b.metadata.width,
            b.metadata.height,
            a.metadata.quality_score,
            b.metadata.quality_score,
            emb_dist,
        );

        let change_score = findings
            .iter()
            .map(|f| f.evidence_score)
            .fold(0.0f64, f64::max);

        if change_score >= p.min_change_score {
            total_change += change_score;
            max_change = max_change.max(change_score);

            for f in &findings {
                *change_types_count.entry(f.change_type.clone()).or_insert(0) += 1;
            }

            let primary_change = findings
                .first()
                .map(|f| f.change_type.clone())
                .unwrap_or_default();

            timeline.push(json!({
                "index": timeline.len(),
                "from_capture": a.id,
                "to_capture": b.id,
                "timestamp": b.timestamp,
                "time_delta_seconds": b.timestamp.saturating_sub(a.timestamp),
                "change_score": (change_score * 100.0).round() / 100.0,
                "primary_change_type": primary_change,
                "finding_count": findings.len(),
                "findings": findings.iter().take(3).map(|f| json!({
                    "type": f.change_type,
                    "severity": f.severity,
                    "description": f.description,
                })).collect::<Vec<_>>(),
            }));
        }

        if timeline.len() >= p.max_entries {
            break;
        }
    }

    let avg_change = if !timeline.is_empty() {
        total_change / timeline.len() as f64
    } else {
        0.0
    };

    // Sort change types by count
    let mut type_summary: Vec<Value> = change_types_count
        .iter()
        .map(|(t, &c)| json!({"type": t, "count": c}))
        .collect();
    type_summary.sort_by(|a, b| {
        b["count"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["count"].as_u64().unwrap_or(0))
    });

    Ok(ToolCallResult::json(&json!({
        "total_events": timeline.len(),
        "total_captures_analyzed": obs_list.len(),
        "summary": {
            "average_change_score": (avg_change * 100.0).round() / 100.0,
            "max_change_score": (max_change * 100.0).round() / 100.0,
            "total_accumulated_change": (total_change * 100.0).round() / 100.0,
            "change_type_distribution": type_summary,
        },
        "timeline": timeline,
    })))
}

// -- vision_forensic_blame ------------------------------------------------

pub fn definition_vision_forensic_blame() -> ToolDefinition {
    ToolDefinition {
        name: "vision_forensic_blame".to_string(),
        description: Some(
            "Attribute which capture transition likely caused a specific visual change using correlation analysis"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["description"],
            "properties": {
                "description": { "type": "string", "description": "Description of the visual change to investigate" },
                "session_id": { "type": "number", "description": "Session to search (default: all)" },
                "max_results": { "type": "number", "description": "Max blame results", "default": 5 }
            }
        }),
    }
}

pub async fn execute_vision_forensic_blame(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        description: String,
        session_id: Option<u32>,
        #[serde(default = "default_max_blame")]
        max_results: usize,
    }
    fn default_max_blame() -> usize {
        5
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let mut obs_list: Vec<_> = store
        .observations
        .iter()
        .filter(|o| match p.session_id {
            Some(sid) => o.session_id == sid,
            None => true,
        })
        .collect();
    obs_list.sort_by_key(|o| o.timestamp);

    if obs_list.len() < 2 {
        return Ok(ToolCallResult::json(&json!({
            "blame_results": [],
            "investigation": "Insufficient captures for blame analysis",
        })));
    }

    let query_lower = p.description.to_lowercase();
    let mut blame_candidates: Vec<Value> = Vec::new();

    for i in 1..obs_list.len() {
        let a = obs_list[i - 1];
        let b = obs_list[i];

        // Compute relevance to the queried change
        let mut relevance = 0.0f64;

        // Check if the description of either capture relates to the query
        if let Some(desc_b) = &b.metadata.description {
            relevance += word_overlap(&query_lower, &desc_b.to_lowercase()) * 0.4;
        }
        if let Some(desc_a) = &a.metadata.description {
            relevance += word_overlap(&query_lower, &desc_a.to_lowercase()) * 0.2;
        }

        // Check label relevance
        for label in &b.metadata.labels {
            if query_lower.contains(&label.to_lowercase()) {
                relevance += 0.2;
                break;
            }
        }

        // Check for actual change magnitude
        let emb_dist = embedding_distance(&a.embedding, &b.embedding);
        let change_factor = emb_dist.min(1.0);
        relevance += change_factor * 0.2;

        // Label changes that match query
        let labels_a: std::collections::HashSet<&String> = a.metadata.labels.iter().collect();
        let labels_b: std::collections::HashSet<&String> = b.metadata.labels.iter().collect();
        let added: Vec<&&String> = labels_b.difference(&labels_a).collect();
        let removed: Vec<&&String> = labels_a.difference(&labels_b).collect();
        for label in added.iter().chain(removed.iter()) {
            if query_lower.contains(&label.to_lowercase()) {
                relevance += 0.3;
                break;
            }
        }

        if relevance > 0.1 {
            blame_candidates.push(json!({
                "from_capture": a.id,
                "to_capture": b.id,
                "timestamp": b.timestamp,
                "relevance_score": (relevance.min(1.0) * 100.0).round() / 100.0,
                "change_magnitude": (emb_dist * 100.0).round() / 100.0,
                "description_before": a.metadata.description,
                "description_after": b.metadata.description,
                "labels_added": added.iter().map(|l| l.to_string()).collect::<Vec<_>>(),
                "labels_removed": removed.iter().map(|l| l.to_string()).collect::<Vec<_>>(),
                "time_delta_seconds": b.timestamp.saturating_sub(a.timestamp),
            }));
        }
    }

    // Sort by relevance descending
    blame_candidates.sort_by(|a, b| {
        b["relevance_score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["relevance_score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    blame_candidates.truncate(p.max_results);

    let confidence = blame_candidates
        .first()
        .and_then(|c| c["relevance_score"].as_f64())
        .unwrap_or(0.0);

    Ok(ToolCallResult::json(&json!({
        "query": p.description,
        "blame_count": blame_candidates.len(),
        "confidence": confidence,
        "blame_results": blame_candidates,
    })))
}

// -- vision_forensic_reconstruct ------------------------------------------

pub fn definition_vision_forensic_reconstruct() -> ToolDefinition {
    ToolDefinition {
        name: "vision_forensic_reconstruct".to_string(),
        description: Some(
            "Reconstruct what happened between two captures by analyzing intermediate evidence"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["capture_start", "capture_end"],
            "properties": {
                "capture_start": { "type": "number", "description": "Starting capture ID" },
                "capture_end": { "type": "number", "description": "Ending capture ID" }
            }
        }),
    }
}

pub async fn execute_vision_forensic_reconstruct(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_start: u64,
        capture_end: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs_start = store
        .observations
        .iter()
        .find(|o| o.id == p.capture_start)
        .ok_or(McpError::CaptureNotFound(p.capture_start))?;
    let obs_end = store
        .observations
        .iter()
        .find(|o| o.id == p.capture_end)
        .ok_or(McpError::CaptureNotFound(p.capture_end))?;

    let (time_start, time_end) = if obs_start.timestamp <= obs_end.timestamp {
        (obs_start.timestamp, obs_end.timestamp)
    } else {
        (obs_end.timestamp, obs_start.timestamp)
    };

    // Find all intermediate captures
    let mut intermediates: Vec<_> = store
        .observations
        .iter()
        .filter(|o| o.timestamp > time_start && o.timestamp < time_end)
        .collect();
    intermediates.sort_by_key(|o| o.timestamp);

    // Build reconstruction chain
    let mut chain: Vec<Value> = Vec::new();
    let mut prev = obs_start;
    let full_chain: Vec<_> = std::iter::once(obs_start)
        .chain(intermediates.iter().copied())
        .chain(std::iter::once(obs_end))
        .collect();

    let mut cumulative_change = 0.0f64;

    for i in 1..full_chain.len() {
        let curr = full_chain[i];
        let emb_dist = embedding_distance(&prev.embedding, &curr.embedding);
        cumulative_change += emb_dist;

        let findings = detect_change_types(
            &prev.metadata.description,
            &curr.metadata.description,
            &prev.metadata.labels,
            &curr.metadata.labels,
            prev.metadata.width,
            prev.metadata.height,
            curr.metadata.width,
            curr.metadata.height,
            prev.metadata.quality_score,
            curr.metadata.quality_score,
            emb_dist,
        );

        let step_type = if curr.id == obs_end.id {
            "final"
        } else if curr.id == obs_start.id {
            "initial"
        } else {
            "intermediate"
        };

        chain.push(json!({
            "step": i,
            "step_type": step_type,
            "from_capture": prev.id,
            "to_capture": curr.id,
            "timestamp": curr.timestamp,
            "time_delta_seconds": curr.timestamp.saturating_sub(prev.timestamp),
            "change_magnitude": (emb_dist * 100.0).round() / 100.0,
            "cumulative_change": (cumulative_change * 100.0).round() / 100.0,
            "primary_finding": findings.first().map(|f| json!({
                "type": f.change_type,
                "severity": f.severity,
                "description": f.description,
            })),
            "description": curr.metadata.description,
        }));

        prev = curr;
    }

    // Overall diff between start and end
    let total_emb_dist = embedding_distance(&obs_start.embedding, &obs_end.embedding);

    let reconstruction_quality = if intermediates.is_empty() {
        "direct_comparison"
    } else if intermediates.len() > 10 {
        "high_fidelity"
    } else if intermediates.len() > 3 {
        "moderate_fidelity"
    } else {
        "low_fidelity"
    };

    Ok(ToolCallResult::json(&json!({
        "capture_start": p.capture_start,
        "capture_end": p.capture_end,
        "time_span_seconds": time_end.saturating_sub(time_start),
        "intermediate_captures": intermediates.len(),
        "total_steps": chain.len(),
        "reconstruction_quality": reconstruction_quality,
        "overall_change": (total_emb_dist * 100.0).round() / 100.0,
        "cumulative_path_change": (cumulative_change * 100.0).round() / 100.0,
        "path_efficiency": if cumulative_change > 1e-10 {
            (total_emb_dist / cumulative_change * 100.0).round() / 100.0
        } else { 1.0 },
        "reconstruction_chain": chain,
    })))
}

// =========================================================================
// INVENTION 21: Visual Anomaly Detection — 4 tools
// =========================================================================

// -- vision_anomaly_detect ------------------------------------------------

pub fn definition_vision_anomaly_detect() -> ToolDefinition {
    ToolDefinition {
        name: "vision_anomaly_detect".to_string(),
        description: Some(
            "Detect statistical anomalies across a capture series using z-score and IQR methods"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "number", "description": "Session to analyze (default: all)" },
                "z_threshold": { "type": "number", "description": "Z-score threshold for anomaly", "default": 2.0 },
                "max_captures": { "type": "number", "description": "Max captures to analyze", "default": 100 }
            }
        }),
    }
}

pub async fn execute_vision_anomaly_detect(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        session_id: Option<u32>,
        #[serde(default = "default_z_threshold")]
        z_threshold: f64,
        #[serde(default = "default_max_anomaly")]
        max_captures: usize,
    }
    fn default_z_threshold() -> f64 {
        2.0
    }
    fn default_max_anomaly() -> usize {
        100
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let mut obs_list: Vec<_> = store
        .observations
        .iter()
        .filter(|o| match p.session_id {
            Some(sid) => o.session_id == sid,
            None => true,
        })
        .collect();
    obs_list.sort_by_key(|o| o.timestamp);
    obs_list.truncate(p.max_captures);

    let n = obs_list.len();
    if n < 3 {
        return Ok(ToolCallResult::json(&json!({
            "error": "Need at least 3 captures for anomaly detection",
            "capture_count": n,
        })));
    }

    // Extract features for all observations
    let all_features: Vec<HashMap<String, f64>> = obs_list
        .iter()
        .map(|obs| {
            extract_forensic_features(
                &obs.embedding,
                obs.metadata.width,
                obs.metadata.height,
                &obs.metadata.labels,
                obs.metadata.quality_score,
                &obs.metadata.description,
            )
        })
        .collect();

    // Compute baseline statistics for each feature
    let feature_names: Vec<String> = all_features[0].keys().cloned().collect();
    let mut baseline: HashMap<String, FeatureStats> = HashMap::new();

    for name in &feature_names {
        let values: Vec<f64> = all_features
            .iter()
            .filter_map(|f| f.get(name).copied())
            .collect();
        baseline.insert(name.clone(), compute_feature_stats(&values));
    }

    // Score each observation
    let threshold = p.z_threshold.max(0.5);
    let mut anomalies: Vec<Value> = Vec::new();
    let mut normal: Vec<Value> = Vec::new();

    for (idx, obs) in obs_list.iter().enumerate() {
        let features = &all_features[idx];
        let mut z_scores_map: HashMap<String, f64> = HashMap::new();
        let mut max_z = 0.0f64;
        let mut contributing: Vec<String> = Vec::new();

        for (name, &value) in features {
            if let Some(stats) = baseline.get(name) {
                let z = z_score(value, stats.mean, stats.std_dev).abs();
                z_scores_map.insert(name.clone(), (z * 100.0).round() / 100.0);
                if z > threshold {
                    contributing.push(format!("{} (z={:.2})", name, z));
                }
                max_z = max_z.max(z);
            }
        }

        let anomaly_score = (max_z / (threshold * 2.0)).min(1.0);
        let is_anomaly = max_z > threshold;

        let entry = json!({
            "capture_id": obs.id,
            "timestamp": obs.timestamp,
            "is_anomaly": is_anomaly,
            "anomaly_score": (anomaly_score * 100.0).round() / 100.0,
            "max_z_score": (max_z * 100.0).round() / 100.0,
            "contributing_factors": contributing,
            "description": obs.metadata.description,
        });

        if is_anomaly {
            anomalies.push(entry);
        } else {
            normal.push(entry);
        }
    }

    // Sort anomalies by score
    anomalies.sort_by(|a, b| {
        b["anomaly_score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["anomaly_score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(ToolCallResult::json(&json!({
        "total_captures": n,
        "anomaly_count": anomalies.len(),
        "normal_count": normal.len(),
        "anomaly_rate": if n > 0 { (anomalies.len() as f64 / n as f64 * 100.0).round() / 100.0 } else { 0.0 },
        "z_threshold": threshold,
        "anomalies": anomalies,
        "baseline_features_used": feature_names.len(),
    })))
}

// -- vision_anomaly_pattern -----------------------------------------------

pub fn definition_vision_anomaly_pattern() -> ToolDefinition {
    ToolDefinition {
        name: "vision_anomaly_pattern".to_string(),
        description: Some("Detect recurring anomaly patterns across a capture series".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "number", "description": "Session to analyze (default: all)" },
                "max_captures": { "type": "number", "description": "Max captures", "default": 100 }
            }
        }),
    }
}

pub async fn execute_vision_anomaly_pattern(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        session_id: Option<u32>,
        #[serde(default = "default_max_pattern")]
        max_captures: usize,
    }
    fn default_max_pattern() -> usize {
        100
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let mut obs_list: Vec<_> = store
        .observations
        .iter()
        .filter(|o| match p.session_id {
            Some(sid) => o.session_id == sid,
            None => true,
        })
        .collect();
    obs_list.sort_by_key(|o| o.timestamp);
    obs_list.truncate(p.max_captures);

    let n = obs_list.len();
    if n < 4 {
        return Ok(ToolCallResult::json(&json!({
            "error": "Need at least 4 captures for pattern detection",
            "capture_count": n,
        })));
    }

    // Compute per-pair change scores as the anomaly series
    let mut change_scores: Vec<f64> = Vec::new();
    for i in 1..n {
        let dist = embedding_distance(&obs_list[i - 1].embedding, &obs_list[i].embedding);
        change_scores.push(dist);
    }

    // Detect overall pattern
    let pattern = detect_anomaly_pattern(&change_scores);

    // Analyze feature-level patterns
    let all_features: Vec<HashMap<String, f64>> = obs_list
        .iter()
        .map(|obs| {
            extract_forensic_features(
                &obs.embedding,
                obs.metadata.width,
                obs.metadata.height,
                &obs.metadata.labels,
                obs.metadata.quality_score,
                &obs.metadata.description,
            )
        })
        .collect();

    // Track feature drift over time
    let key_features = [
        "quality_score",
        "embedding_mean",
        "embedding_std",
        "embedding_energy",
        "zero_crossing_rate",
        "spectral_centroid",
    ];

    let mut feature_patterns: Vec<Value> = Vec::new();
    for &feature_name in &key_features {
        let series: Vec<f64> = all_features
            .iter()
            .filter_map(|f| f.get(feature_name).copied())
            .collect();
        if series.len() >= 4 {
            let feat_pattern = detect_anomaly_pattern(&series);
            let stats = compute_feature_stats(&series);

            // Check if this feature is interesting (has significant variation)
            let cv = if stats.mean.abs() > 1e-10 {
                stats.std_dev / stats.mean.abs()
            } else {
                0.0
            };

            if cv > 0.05 {
                feature_patterns.push(json!({
                    "feature": feature_name,
                    "pattern_type": feat_pattern.label(),
                    "mean": (stats.mean * 1000.0).round() / 1000.0,
                    "std_dev": (stats.std_dev * 1000.0).round() / 1000.0,
                    "coefficient_of_variation": (cv * 100.0).round() / 100.0,
                    "range": (stats.max - stats.min),
                }));
            }
        }
    }

    // Compute overall change statistics
    let change_stats = compute_feature_stats(&change_scores);

    Ok(ToolCallResult::json(&json!({
        "total_captures": n,
        "overall_pattern": pattern.label(),
        "change_series_stats": {
            "mean": (change_stats.mean * 1000.0).round() / 1000.0,
            "std_dev": (change_stats.std_dev * 1000.0).round() / 1000.0,
            "median": (change_stats.median * 1000.0).round() / 1000.0,
            "min": (change_stats.min * 1000.0).round() / 1000.0,
            "max": (change_stats.max * 1000.0).round() / 1000.0,
            "iqr": (change_stats.iqr * 1000.0).round() / 1000.0,
        },
        "feature_patterns": feature_patterns,
        "pattern_assessment": match pattern {
            AnomalyPatternType::SingleSpike => "Isolated anomaly event detected",
            AnomalyPatternType::GradualDrift => "Gradual drift in visual properties over time",
            AnomalyPatternType::OscillatingValues => "Oscillating pattern suggests unstable visual state",
            AnomalyPatternType::StepChange => "Step change detected, indicating a discrete visual transition",
            AnomalyPatternType::Clustering => "Multiple clustered anomaly events detected",
            AnomalyPatternType::Trending => "Monotonic trend in visual properties",
        },
    })))
}

// -- vision_anomaly_baseline ----------------------------------------------

pub fn definition_vision_anomaly_baseline() -> ToolDefinition {
    ToolDefinition {
        name: "vision_anomaly_baseline".to_string(),
        description: Some(
            "Establish baseline statistics for a session to use in anomaly detection".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "number", "description": "Session to baseline (default: all)" },
                "max_captures": { "type": "number", "description": "Max captures for baseline", "default": 100 }
            }
        }),
    }
}

pub async fn execute_vision_anomaly_baseline(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        session_id: Option<u32>,
        #[serde(default = "default_max_baseline")]
        max_captures: usize,
    }
    fn default_max_baseline() -> usize {
        100
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let mut obs_list: Vec<_> = store
        .observations
        .iter()
        .filter(|o| match p.session_id {
            Some(sid) => o.session_id == sid,
            None => true,
        })
        .collect();
    obs_list.sort_by_key(|o| o.timestamp);
    obs_list.truncate(p.max_captures);

    let n = obs_list.len();
    if n < 2 {
        return Ok(ToolCallResult::json(&json!({
            "error": "Need at least 2 captures to establish baseline",
            "capture_count": n,
        })));
    }

    let all_features: Vec<HashMap<String, f64>> = obs_list
        .iter()
        .map(|obs| {
            extract_forensic_features(
                &obs.embedding,
                obs.metadata.width,
                obs.metadata.height,
                &obs.metadata.labels,
                obs.metadata.quality_score,
                &obs.metadata.description,
            )
        })
        .collect();

    let feature_names: Vec<String> = {
        let mut names: Vec<String> = all_features[0].keys().cloned().collect();
        names.sort();
        names
    };

    let mut baseline_stats: Vec<Value> = Vec::new();

    for name in &feature_names {
        let values: Vec<f64> = all_features
            .iter()
            .filter_map(|f| f.get(name).copied())
            .collect();
        let stats = compute_feature_stats(&values);

        baseline_stats.push(json!({
            "feature": name,
            "mean": (stats.mean * 10000.0).round() / 10000.0,
            "std_dev": (stats.std_dev * 10000.0).round() / 10000.0,
            "min": (stats.min * 10000.0).round() / 10000.0,
            "max": (stats.max * 10000.0).round() / 10000.0,
            "median": (stats.median * 10000.0).round() / 10000.0,
            "q1": (stats.q1 * 10000.0).round() / 10000.0,
            "q3": (stats.q3 * 10000.0).round() / 10000.0,
            "iqr": (stats.iqr * 10000.0).round() / 10000.0,
        }));
    }

    Ok(ToolCallResult::json(&json!({
        "sample_count": n,
        "feature_count": feature_names.len(),
        "created_at": now_epoch(),
        "session_id": p.session_id,
        "baseline": baseline_stats,
    })))
}

// -- vision_anomaly_alert -------------------------------------------------

pub fn definition_vision_anomaly_alert() -> ToolDefinition {
    ToolDefinition {
        name: "vision_anomaly_alert".to_string(),
        description: Some(
            "Check a specific capture against baseline thresholds and alert on anomalies"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture to check" },
                "z_threshold": { "type": "number", "description": "Z-score threshold", "default": 2.0 },
                "iqr_factor": { "type": "number", "description": "IQR factor for outlier detection", "default": 1.5 }
            }
        }),
    }
}

pub async fn execute_vision_anomaly_alert(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
        #[serde(default = "default_z_alert")]
        z_threshold: f64,
        #[serde(default = "default_iqr_factor")]
        iqr_factor: f64,
    }
    fn default_z_alert() -> f64 {
        2.0
    }
    fn default_iqr_factor() -> f64 {
        1.5
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let target_obs = store
        .observations
        .iter()
        .find(|o| o.id == p.capture_id)
        .ok_or(McpError::CaptureNotFound(p.capture_id))?;

    // Build baseline from all OTHER captures in the same session
    let baseline_obs: Vec<_> = store
        .observations
        .iter()
        .filter(|o| o.session_id == target_obs.session_id && o.id != p.capture_id)
        .collect();

    if baseline_obs.len() < 2 {
        return Ok(ToolCallResult::json(&json!({
            "capture_id": p.capture_id,
            "alert": false,
            "reason": "Insufficient baseline captures in this session",
            "baseline_count": baseline_obs.len(),
        })));
    }

    // Extract features
    let target_features = extract_forensic_features(
        &target_obs.embedding,
        target_obs.metadata.width,
        target_obs.metadata.height,
        &target_obs.metadata.labels,
        target_obs.metadata.quality_score,
        &target_obs.metadata.description,
    );

    let baseline_features: Vec<HashMap<String, f64>> = baseline_obs
        .iter()
        .map(|obs| {
            extract_forensic_features(
                &obs.embedding,
                obs.metadata.width,
                obs.metadata.height,
                &obs.metadata.labels,
                obs.metadata.quality_score,
                &obs.metadata.description,
            )
        })
        .collect();

    let mut alerts: Vec<Value> = Vec::new();
    let mut max_severity = ForensicSeverity::Info;
    let threshold = p.z_threshold.max(0.5);
    let iqr_factor = p.iqr_factor.max(0.5);

    for (name, &value) in &target_features {
        let baseline_values: Vec<f64> = baseline_features
            .iter()
            .filter_map(|f| f.get(name).copied())
            .collect();

        if baseline_values.len() < 2 {
            continue;
        }

        let stats = compute_feature_stats(&baseline_values);
        let z = z_score(value, stats.mean, stats.std_dev).abs();
        let is_z_anomaly = z > threshold;
        let is_iqr_anomaly = is_iqr_outlier(value, stats.q1, stats.q3, stats.iqr, iqr_factor);

        if is_z_anomaly || is_iqr_anomaly {
            let severity = ForensicSeverity::from_score((z / (threshold * 2.0)).min(1.0));
            match (&severity, &max_severity) {
                (ForensicSeverity::Critical, _) => max_severity = ForensicSeverity::Critical,
                (
                    ForensicSeverity::High,
                    ForensicSeverity::Info | ForensicSeverity::Low | ForensicSeverity::Medium,
                ) => max_severity = ForensicSeverity::High,
                (ForensicSeverity::Medium, ForensicSeverity::Info | ForensicSeverity::Low) => {
                    max_severity = ForensicSeverity::Medium
                }
                (ForensicSeverity::Low, ForensicSeverity::Info) => {
                    max_severity = ForensicSeverity::Low
                }
                _ => {}
            }

            alerts.push(json!({
                "feature": name,
                "value": (value * 10000.0).round() / 10000.0,
                "baseline_mean": (stats.mean * 10000.0).round() / 10000.0,
                "z_score": (z * 100.0).round() / 100.0,
                "is_z_anomaly": is_z_anomaly,
                "is_iqr_anomaly": is_iqr_anomaly,
                "severity": severity.label(),
                "direction": if value > stats.mean { "above" } else { "below" },
            }));
        }
    }

    // Sort alerts by z_score descending
    alerts.sort_by(|a, b| {
        b["z_score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["z_score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let has_alert = !alerts.is_empty();

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "alert": has_alert,
        "severity": max_severity.label(),
        "alert_count": alerts.len(),
        "baseline_count": baseline_obs.len(),
        "alerts": alerts,
        "thresholds": {
            "z_score": threshold,
            "iqr_factor": iqr_factor,
        },
    })))
}

// =========================================================================
// INVENTION 22: Visual Regression Testing — 3 tools
// =========================================================================

// -- vision_regression_snapshot -------------------------------------------

pub fn definition_vision_regression_snapshot() -> ToolDefinition {
    ToolDefinition {
        name: "vision_regression_snapshot".to_string(),
        description: Some(
            "Take a regression snapshot of a capture for future comparison".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture to snapshot" },
                "name": { "type": "string", "description": "Name for this snapshot" }
            }
        }),
    }
}

pub async fn execute_vision_regression_snapshot(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
        name: Option<String>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs = store
        .observations
        .iter()
        .find(|o| o.id == p.capture_id)
        .ok_or(McpError::CaptureNotFound(p.capture_id))?;

    // Extract comprehensive snapshot data
    let features = extract_forensic_features(
        &obs.embedding,
        obs.metadata.width,
        obs.metadata.height,
        &obs.metadata.labels,
        obs.metadata.quality_score,
        &obs.metadata.description,
    );

    let snapshot_name = p.name.unwrap_or_else(|| format!("snapshot_{}", obs.id));

    // Compute embedding hash for quick comparison
    let emb_hash = {
        let mut h: u64 = 0xcbf29ce484222325;
        for &v in &obs.embedding {
            h ^= v.to_bits() as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    };

    Ok(ToolCallResult::json(&json!({
        "snapshot_name": snapshot_name,
        "capture_id": p.capture_id,
        "created_at": now_epoch(),
        "dimensions": {
            "width": obs.metadata.width,
            "height": obs.metadata.height,
        },
        "quality_score": obs.metadata.quality_score,
        "labels": obs.metadata.labels,
        "description": obs.metadata.description,
        "embedding_hash": format!("{:016x}", emb_hash),
        "embedding_dim": obs.embedding.len(),
        "feature_snapshot": features.iter().map(|(k, v)| {
            json!({ "feature": k, "value": (*v * 10000.0).round() / 10000.0 })
        }).collect::<Vec<_>>(),
        "status": "snapshot_created",
    })))
}

// -- vision_regression_check ----------------------------------------------

pub fn definition_vision_regression_check() -> ToolDefinition {
    ToolDefinition {
        name: "vision_regression_check".to_string(),
        description: Some(
            "Check current capture against a reference capture for visual regression".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["current_capture_id", "reference_capture_id"],
            "properties": {
                "current_capture_id": { "type": "number", "description": "Current capture to check" },
                "reference_capture_id": { "type": "number", "description": "Reference/baseline capture" },
                "tolerance": { "type": "number", "description": "Tolerance for pass/fail (0.0-1.0)", "default": 0.1 }
            }
        }),
    }
}

pub async fn execute_vision_regression_check(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        current_capture_id: u64,
        reference_capture_id: u64,
        #[serde(default = "default_tolerance")]
        tolerance: f64,
    }
    fn default_tolerance() -> f64 {
        0.1
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs_current = store
        .observations
        .iter()
        .find(|o| o.id == p.current_capture_id)
        .ok_or(McpError::CaptureNotFound(p.current_capture_id))?;
    let obs_reference = store
        .observations
        .iter()
        .find(|o| o.id == p.reference_capture_id)
        .ok_or(McpError::CaptureNotFound(p.reference_capture_id))?;

    let tolerance = p.tolerance.max(0.01).min(1.0);

    // Compute various diff metrics
    let emb_dist = embedding_distance(&obs_current.embedding, &obs_reference.embedding);
    let cos_sim = cosine_similarity(&obs_current.embedding, &obs_reference.embedding);

    // Size regression check
    let size_match = obs_current.metadata.width == obs_reference.metadata.width
        && obs_current.metadata.height == obs_reference.metadata.height;

    // Quality regression check
    let quality_diff =
        (obs_current.metadata.quality_score - obs_reference.metadata.quality_score).abs();
    let quality_regression =
        obs_current.metadata.quality_score < obs_reference.metadata.quality_score - 0.1;

    // Label regression check
    let ref_labels: std::collections::HashSet<&String> =
        obs_reference.metadata.labels.iter().collect();
    let cur_labels: std::collections::HashSet<&String> =
        obs_current.metadata.labels.iter().collect();
    let missing_labels: Vec<&&String> = ref_labels.difference(&cur_labels).collect();
    let new_labels: Vec<&&String> = cur_labels.difference(&ref_labels).collect();

    // Description change
    let desc_similarity = match (
        &obs_current.metadata.description,
        &obs_reference.metadata.description,
    ) {
        (Some(dc), Some(dr)) => word_overlap(dc, dr),
        (None, None) => 1.0,
        _ => 0.0,
    };

    // Determine forensic findings
    let findings = detect_change_types(
        &obs_reference.metadata.description,
        &obs_current.metadata.description,
        &obs_reference.metadata.labels,
        &obs_current.metadata.labels,
        obs_reference.metadata.width,
        obs_reference.metadata.height,
        obs_current.metadata.width,
        obs_current.metadata.height,
        obs_reference.metadata.quality_score,
        obs_current.metadata.quality_score,
        emb_dist,
    );

    // Overall regression score (higher = more regression)
    let mut regression_score = 0.0f64;
    regression_score += emb_dist * 0.4;
    if !size_match {
        regression_score += 0.15;
    }
    if quality_regression {
        regression_score += quality_diff as f64 * 0.2;
    }
    regression_score += (1.0 - desc_similarity) * 0.15;
    regression_score += (missing_labels.len() as f64 * 0.05).min(0.1);
    regression_score = regression_score.min(1.0);

    let status = if regression_score <= tolerance * 0.5 {
        RegressionStatus::Pass
    } else if regression_score <= tolerance {
        RegressionStatus::Warning
    } else {
        RegressionStatus::Fail
    };

    Ok(ToolCallResult::json(&json!({
        "current_capture_id": p.current_capture_id,
        "reference_capture_id": p.reference_capture_id,
        "status": status.label(),
        "regression_score": (regression_score * 100.0).round() / 100.0,
        "tolerance": tolerance,
        "checks": {
            "embedding_distance": (emb_dist * 10000.0).round() / 10000.0,
            "cosine_similarity": (cos_sim * 10000.0).round() / 10000.0,
            "size_match": size_match,
            "quality_diff": (quality_diff * 100.0).round() / 100.0,
            "quality_regression": quality_regression,
            "description_similarity": (desc_similarity * 100.0).round() / 100.0,
            "missing_labels": missing_labels.iter().map(|l| l.to_string()).collect::<Vec<_>>(),
            "new_labels": new_labels.iter().map(|l| l.to_string()).collect::<Vec<_>>(),
        },
        "findings": findings.iter().map(|f| json!({
            "type": f.change_type,
            "severity": f.severity,
            "description": f.description,
        })).collect::<Vec<_>>(),
    })))
}

// -- vision_regression_report ---------------------------------------------

pub fn definition_vision_regression_report() -> ToolDefinition {
    ToolDefinition {
        name: "vision_regression_report".to_string(),
        description: Some(
            "Generate a full regression report comparing captures across a session".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "number", "description": "Session to report on (default: all)" },
                "reference_capture_id": { "type": "number", "description": "Reference capture for comparison (default: first)" },
                "tolerance": { "type": "number", "description": "Tolerance threshold", "default": 0.1 },
                "max_captures": { "type": "number", "description": "Max captures to include", "default": 50 }
            }
        }),
    }
}

pub async fn execute_vision_regression_report(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        session_id: Option<u32>,
        reference_capture_id: Option<u64>,
        #[serde(default = "default_report_tolerance")]
        tolerance: f64,
        #[serde(default = "default_max_report")]
        max_captures: usize,
    }
    fn default_report_tolerance() -> f64 {
        0.1
    }
    fn default_max_report() -> usize {
        50
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let mut obs_list: Vec<_> = store
        .observations
        .iter()
        .filter(|o| match p.session_id {
            Some(sid) => o.session_id == sid,
            None => true,
        })
        .collect();
    obs_list.sort_by_key(|o| o.timestamp);
    obs_list.truncate(p.max_captures);

    if obs_list.is_empty() {
        return Ok(ToolCallResult::json(&json!({
            "error": "No captures found for report",
        })));
    }

    // Determine reference
    let ref_id = p.reference_capture_id.unwrap_or(obs_list[0].id);
    let ref_obs = store
        .observations
        .iter()
        .find(|o| o.id == ref_id)
        .ok_or(McpError::CaptureNotFound(ref_id))?;

    let tolerance = p.tolerance.max(0.01).min(1.0);

    let mut pass_count = 0u32;
    let mut warning_count = 0u32;
    let mut fail_count = 0u32;
    let mut results: Vec<Value> = Vec::new();

    for obs in &obs_list {
        if obs.id == ref_id {
            continue;
        }

        let emb_dist = embedding_distance(&obs.embedding, &ref_obs.embedding);
        let desc_sim = match (&obs.metadata.description, &ref_obs.metadata.description) {
            (Some(a), Some(b)) => word_overlap(a, b),
            (None, None) => 1.0,
            _ => 0.0,
        };

        let size_match = obs.metadata.width == ref_obs.metadata.width
            && obs.metadata.height == ref_obs.metadata.height;
        let quality_diff = (obs.metadata.quality_score - ref_obs.metadata.quality_score).abs();

        let mut score = emb_dist * 0.4 + (1.0 - desc_sim) * 0.2;
        if !size_match {
            score += 0.15;
        }
        score += quality_diff as f64 * 0.15;
        score = score.min(1.0);

        let status = if score <= tolerance * 0.5 {
            pass_count += 1;
            RegressionStatus::Pass
        } else if score <= tolerance {
            warning_count += 1;
            RegressionStatus::Warning
        } else {
            fail_count += 1;
            RegressionStatus::Fail
        };

        results.push(json!({
            "capture_id": obs.id,
            "timestamp": obs.timestamp,
            "status": status.label(),
            "regression_score": (score * 100.0).round() / 100.0,
            "embedding_distance": (emb_dist * 1000.0).round() / 1000.0,
            "description_similarity": (desc_sim * 100.0).round() / 100.0,
            "size_match": size_match,
        }));
    }

    let total = pass_count + warning_count + fail_count;
    let pass_rate = if total > 0 {
        pass_count as f64 / total as f64
    } else {
        1.0
    };

    let overall_status = if fail_count > 0 {
        "fail"
    } else if warning_count > 0 {
        "warning"
    } else {
        "pass"
    };

    Ok(ToolCallResult::json(&json!({
        "reference_capture_id": ref_id,
        "tolerance": tolerance,
        "overall_status": overall_status,
        "summary": {
            "total_checked": total,
            "passed": pass_count,
            "warnings": warning_count,
            "failed": fail_count,
            "pass_rate": (pass_rate * 100.0).round() / 100.0,
        },
        "results": results,
    })))
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0f32, 2.0, 3.0];
        let sim = cosine_similarity(&a, &a);
        assert!(
            (sim - 1.0).abs() < 0.001,
            "Same vector should have sim ~1.0: {}",
            sim
        );
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0f32, 0.0, 0.0];
        let b = vec![0.0f32, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(
            sim.abs() < 0.001,
            "Orthogonal vectors should have sim ~0: {}",
            sim
        );
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0f32, 2.0, 3.0];
        let b = vec![-1.0f32, -2.0, -3.0];
        let sim = cosine_similarity(&a, &b);
        assert!(
            (sim + 1.0).abs() < 0.001,
            "Opposite vectors should have sim ~-1.0: {}",
            sim
        );
    }

    #[test]
    fn test_embedding_distance_range() {
        let a = vec![1.0f32, 0.5, 0.2];
        let b = vec![0.3f32, 0.8, 0.1];
        let d = embedding_distance(&a, &b);
        assert!(d >= 0.0 && d <= 2.0);
    }

    #[test]
    fn test_compute_feature_stats_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = compute_feature_stats(&values);
        assert!((stats.mean - 3.0).abs() < 0.001);
        assert!((stats.min - 1.0).abs() < 0.001);
        assert!((stats.max - 5.0).abs() < 0.001);
        assert!((stats.median - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_feature_stats_empty() {
        let stats = compute_feature_stats(&[]);
        assert_eq!(stats.mean, 0.0);
        assert_eq!(stats.std_dev, 0.0);
    }

    #[test]
    fn test_compute_feature_stats_single() {
        let stats = compute_feature_stats(&[42.0]);
        assert!((stats.mean - 42.0).abs() < 0.001);
        assert!((stats.median - 42.0).abs() < 0.001);
    }

    #[test]
    fn test_z_score_at_mean() {
        let z = z_score(5.0, 5.0, 2.0);
        assert!((z).abs() < 0.001);
    }

    #[test]
    fn test_z_score_one_std() {
        let z = z_score(7.0, 5.0, 2.0);
        assert!((z - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_z_score_zero_std() {
        let z = z_score(5.0, 5.0, 0.0);
        assert_eq!(z, 0.0); // value == mean, std == 0 => 0
    }

    #[test]
    fn test_z_score_zero_std_different() {
        let z = z_score(6.0, 5.0, 0.0);
        assert_eq!(z, 3.0); // fallback for zero std
    }

    #[test]
    fn test_is_iqr_outlier() {
        assert!(is_iqr_outlier(100.0, 10.0, 30.0, 20.0, 1.5));
        assert!(!is_iqr_outlier(20.0, 10.0, 30.0, 20.0, 1.5));
    }

    #[test]
    fn test_detect_change_types_no_change() {
        let desc = Some("test page".to_string());
        let labels = vec!["ui".to_string()];
        let findings = detect_change_types(
            &desc, &desc, &labels, &labels, 800, 600, 800, 600, 0.8, 0.8, 0.01,
        );
        assert_eq!(findings[0].change_type, "no_change");
    }

    #[test]
    fn test_detect_change_types_size_change() {
        let desc = Some("test".to_string());
        let labels = vec![];
        let findings = detect_change_types(
            &desc, &desc, &labels, &labels, 800, 600, 1920, 1080, 0.8, 0.8, 0.01,
        );
        assert!(findings.iter().any(|f| f.change_type == "size_changed"));
    }

    #[test]
    fn test_detect_change_types_label_change() {
        let desc = Some("test".to_string());
        let labels_a = vec!["button".to_string()];
        let labels_b = vec!["button".to_string(), "modal".to_string()];
        let findings = detect_change_types(
            &desc, &desc, &labels_a, &labels_b, 800, 600, 800, 600, 0.8, 0.8, 0.01,
        );
        assert!(findings.iter().any(|f| f.change_type == "content_added"));
    }

    #[test]
    fn test_detect_anomaly_pattern_spike() {
        let scores = vec![0.1, 0.1, 0.1, 0.1, 0.9, 0.1, 0.1, 0.1, 0.1, 0.1];
        let pattern = detect_anomaly_pattern(&scores);
        assert!(matches!(pattern, AnomalyPatternType::SingleSpike));
    }

    #[test]
    fn test_detect_anomaly_pattern_oscillating() {
        let scores = vec![0.1, 0.9, 0.1, 0.9, 0.1, 0.9, 0.1, 0.9];
        let pattern = detect_anomaly_pattern(&scores);
        assert!(matches!(pattern, AnomalyPatternType::OscillatingValues));
    }

    #[test]
    fn test_detect_anomaly_pattern_trending() {
        let scores = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let pattern = detect_anomaly_pattern(&scores);
        assert!(matches!(pattern, AnomalyPatternType::Trending));
    }

    #[test]
    fn test_extract_forensic_features_basic() {
        let embedding: Vec<f32> = (0..64).map(|i| i as f32 * 0.1).collect();
        let labels = vec!["ui".to_string()];
        let desc = Some("A test page".to_string());
        let features = extract_forensic_features(&embedding, 800, 600, &labels, 0.8, &desc);

        assert!(features.contains_key("width"));
        assert!(features.contains_key("height"));
        assert!(features.contains_key("quality_score"));
        assert!(features.contains_key("embedding_mean"));
        assert!(features.contains_key("embedding_std"));
        assert!(features.contains_key("embedding_energy"));
        assert!(features.contains_key("zero_crossing_rate"));
        assert!(features.contains_key("spectral_centroid"));
        assert!(features.contains_key("embedding_kurtosis"));
        assert!(features.contains_key("embedding_skewness"));
    }

    #[test]
    fn test_extract_forensic_features_empty_embedding() {
        let features = extract_forensic_features(&[], 800, 600, &[], 0.5, &None);
        assert_eq!(*features.get("width").unwrap(), 800.0);
        assert_eq!(*features.get("height").unwrap(), 600.0);
        assert!(!features.contains_key("embedding_mean")); // No embedding stats
    }

    #[test]
    fn test_forensic_severity_from_score() {
        assert!(matches!(
            ForensicSeverity::from_score(0.1),
            ForensicSeverity::Info
        ));
        assert!(matches!(
            ForensicSeverity::from_score(0.3),
            ForensicSeverity::Low
        ));
        assert!(matches!(
            ForensicSeverity::from_score(0.5),
            ForensicSeverity::Medium
        ));
        assert!(matches!(
            ForensicSeverity::from_score(0.7),
            ForensicSeverity::High
        ));
        assert!(matches!(
            ForensicSeverity::from_score(0.9),
            ForensicSeverity::Critical
        ));
    }

    #[test]
    fn test_forensic_change_type_labels() {
        assert_eq!(ForensicChangeType::ContentAdded.label(), "content_added");
        assert_eq!(ForensicChangeType::LayoutShifted.label(), "layout_shifted");
        assert_eq!(ForensicChangeType::NoChange.label(), "no_change");
    }

    #[test]
    fn test_regression_status_labels() {
        assert_eq!(RegressionStatus::Pass.label(), "pass");
        assert_eq!(RegressionStatus::Warning.label(), "warning");
        assert_eq!(RegressionStatus::Fail.label(), "fail");
        assert_eq!(RegressionStatus::Error.label(), "error");
    }

    #[test]
    fn test_anomaly_pattern_labels() {
        assert_eq!(AnomalyPatternType::SingleSpike.label(), "single_spike");
        assert_eq!(AnomalyPatternType::GradualDrift.label(), "gradual_drift");
        assert_eq!(AnomalyPatternType::StepChange.label(), "step_change");
    }

    #[test]
    fn test_word_overlap_partial() {
        let score = word_overlap("hello world test", "hello test");
        assert!(
            score > 0.5,
            "Partial overlap should give moderate score: {}",
            score
        );
    }
}
