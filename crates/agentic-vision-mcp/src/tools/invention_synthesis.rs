//! Synthesis Inventions (17–19): Visual DNA, Visual Composition Analysis,
//! Visual Clustering.
//!
//! 11 MCP tools that synthesize and generate visual understanding.

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

/// Color histogram bucket for a visual capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorHistogram {
    /// Distribution across 8 hue buckets (0-360 mapped to 8 bins).
    pub hue_bins: [f64; 8],
    /// Average saturation in [0.0, 1.0].
    pub avg_saturation: f64,
    /// Average brightness in [0.0, 1.0].
    pub avg_brightness: f64,
    /// Dominant hue bucket index (0-7).
    pub dominant_hue: usize,
    /// Color entropy (higher = more varied colors).
    pub entropy: f64,
}

/// Edge density metrics for a visual capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDensityProfile {
    /// Overall edge density ratio [0.0, 1.0].
    pub overall_density: f64,
    /// Edge density per quadrant [top-left, top-right, bottom-left, bottom-right].
    pub quadrant_density: [f64; 4],
    /// Horizontal edge dominance vs vertical (>1.0 = more horizontal).
    pub h_v_ratio: f64,
    /// Estimated number of distinct visual blocks.
    pub block_count: u32,
}

/// Layout pattern detected in a capture.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LayoutPattern {
    SingleColumn,
    TwoColumn,
    ThreeColumn,
    Grid,
    FreeForm,
    FullWidth,
    Centered,
    Sidebar,
    Dashboard,
    Card,
}

impl LayoutPattern {
    fn label(&self) -> &str {
        match self {
            Self::SingleColumn => "single_column",
            Self::TwoColumn => "two_column",
            Self::ThreeColumn => "three_column",
            Self::Grid => "grid",
            Self::FreeForm => "free_form",
            Self::FullWidth => "full_width",
            Self::Centered => "centered",
            Self::Sidebar => "sidebar",
            Self::Dashboard => "dashboard",
            Self::Card => "card",
        }
    }
}

/// Visual DNA fingerprint of a capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualDNA {
    pub capture_id: u64,
    pub color_profile: ColorHistogram,
    pub edge_profile: EdgeDensityProfile,
    pub layout_pattern: String,
    pub text_density: f64,
    pub aspect_ratio: f64,
    pub complexity_score: f64,
    pub label_signature: Vec<String>,
    pub fingerprint_hash: u64,
}

/// Composition analysis result.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompositionQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Unbalanced,
}

impl CompositionQuality {
    fn label(&self) -> &str {
        match self {
            Self::Excellent => "excellent",
            Self::Good => "good",
            Self::Fair => "fair",
            Self::Poor => "poor",
            Self::Unbalanced => "unbalanced",
        }
    }
}

/// Grid alignment detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridAlignment {
    pub columns: u32,
    pub rows: u32,
    pub alignment_score: f64,
    pub gutter_consistency: f64,
}

/// Visual weight distribution across quadrants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualWeightMap {
    pub top_left: f64,
    pub top_right: f64,
    pub bottom_left: f64,
    pub bottom_right: f64,
    pub center_of_mass: (f64, f64),
    pub balance_score: f64,
}

/// Cluster assignment for a capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterAssignment {
    pub capture_id: u64,
    pub cluster_id: usize,
    pub distance_to_centroid: f64,
    pub is_outlier: bool,
}

/// A cluster centroid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterCentroid {
    pub cluster_id: usize,
    pub member_count: usize,
    pub centroid_features: Vec<f64>,
    pub avg_intra_distance: f64,
    pub representative_capture_id: Option<u64>,
    pub common_labels: Vec<String>,
}

// =========================================================================
// Helpers
// =========================================================================

#[allow(dead_code)]
fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[allow(dead_code)]
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

/// Compute a deterministic hash from a slice of f64 features.
fn feature_hash(features: &[f64]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325; // FNV-1a offset basis
    for &f in features {
        let bits = f.to_bits();
        h ^= bits;
        h = h.wrapping_mul(0x100000001b3); // FNV prime
    }
    h
}

/// Compute color histogram from embedding + metadata heuristics.
/// Since we don't have raw pixels, we derive a pseudo-histogram from
/// the embedding vector, treating groups of dimensions as color channels.
fn compute_color_histogram(embedding: &[f32], width: u32, height: u32) -> ColorHistogram {
    let mut hue_bins = [0.0f64; 8];
    let dim = embedding.len();

    if dim == 0 {
        return ColorHistogram {
            hue_bins,
            avg_saturation: 0.0,
            avg_brightness: 0.0,
            dominant_hue: 0,
            entropy: 0.0,
        };
    }

    // Map embedding dimensions to 8 hue bins using modular assignment
    let mut total_magnitude = 0.0f64;
    for (i, &val) in embedding.iter().enumerate() {
        let bin = i % 8;
        let contribution = (val.abs() as f64).powi(2);
        hue_bins[bin] += contribution;
        total_magnitude += contribution;
    }

    // Normalize to probability distribution
    if total_magnitude > 0.0 {
        for bin in &mut hue_bins {
            *bin /= total_magnitude;
        }
    }

    // Dominant hue
    let dominant_hue = hue_bins
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);

    // Shannon entropy of the distribution
    let entropy = -hue_bins
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| p * p.ln())
        .sum::<f64>();

    // Saturation from spread of embedding values
    let mean = embedding.iter().map(|&v| v as f64).sum::<f64>() / dim as f64;
    let variance = embedding
        .iter()
        .map(|&v| {
            let d = v as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / dim as f64;
    let avg_saturation = (variance.sqrt() / 2.0).min(1.0);

    // Brightness from aspect ratio and mean embedding value
    let aspect = if height > 0 {
        width as f64 / height as f64
    } else {
        1.0
    };
    let avg_brightness = ((mean.abs() + aspect * 0.1) / 2.0).min(1.0);

    ColorHistogram {
        hue_bins,
        avg_saturation,
        avg_brightness,
        dominant_hue,
        entropy,
    }
}

/// Compute edge density profile from embedding features.
fn compute_edge_density(embedding: &[f32], width: u32, height: u32) -> EdgeDensityProfile {
    let dim = embedding.len();
    if dim < 4 {
        return EdgeDensityProfile {
            overall_density: 0.0,
            quadrant_density: [0.0; 4],
            h_v_ratio: 1.0,
            block_count: 1,
        };
    }

    // Split embedding into 4 quadrants
    let quarter = dim / 4;
    let mut quadrant_energy = [0.0f64; 4];

    for q in 0..4 {
        let start = q * quarter;
        let end = if q == 3 { dim } else { (q + 1) * quarter };
        let mut prev = embedding[start] as f64;
        let mut diff_sum = 0.0;
        for &val in &embedding[start + 1..end] {
            let v = val as f64;
            diff_sum += (v - prev).abs();
            prev = v;
        }
        quadrant_energy[q] = diff_sum / (end - start) as f64;
    }

    // Normalize quadrant density to [0, 1]
    let max_energy = quadrant_energy
        .iter()
        .cloned()
        .fold(0.0f64, f64::max)
        .max(1e-10);
    let mut quadrant_density = [0.0f64; 4];
    for (i, &e) in quadrant_energy.iter().enumerate() {
        quadrant_density[i] = (e / max_energy).min(1.0);
    }

    let overall_density = quadrant_density.iter().sum::<f64>() / 4.0;

    // H/V ratio: compare even vs odd indexed differences
    let mut h_energy = 0.0f64;
    let mut v_energy = 0.0f64;
    for i in 1..dim {
        let diff = (embedding[i] as f64 - embedding[i - 1] as f64).abs();
        if i % 2 == 0 {
            h_energy += diff;
        } else {
            v_energy += diff;
        }
    }
    let h_v_ratio = if v_energy > 1e-10 {
        h_energy / v_energy
    } else {
        1.0
    };

    // Block count estimation: count zero-crossings in first-derivative
    let mut crossings = 0u32;
    let mut prev_sign = true;
    for i in 2..dim {
        let d1 = embedding[i] as f64 - embedding[i - 1] as f64;
        let current_sign = d1 >= 0.0;
        if current_sign != prev_sign {
            crossings += 1;
        }
        prev_sign = current_sign;
    }
    let block_count = (crossings / 4).max(1).min(50);

    // Factor in dimensions
    let area_factor = if width > 0 && height > 0 {
        ((width * height) as f64 / 1_000_000.0).min(2.0)
    } else {
        1.0
    };

    EdgeDensityProfile {
        overall_density: (overall_density * area_factor).min(1.0),
        quadrant_density,
        h_v_ratio,
        block_count,
    }
}

/// Detect layout pattern from aspect ratio, dimensions, and labels.
fn detect_layout_pattern(
    width: u32,
    height: u32,
    labels: &[String],
    edge_density: f64,
) -> LayoutPattern {
    let aspect = if height > 0 {
        width as f64 / height as f64
    } else {
        1.0
    };
    let label_set: std::collections::HashSet<String> =
        labels.iter().map(|l| l.to_lowercase()).collect();

    // Check label-based hints first
    if label_set.contains("dashboard") || label_set.contains("admin") {
        return LayoutPattern::Dashboard;
    }
    if label_set.contains("sidebar") {
        return LayoutPattern::Sidebar;
    }
    if label_set.contains("card") || label_set.contains("cards") {
        return LayoutPattern::Card;
    }
    if label_set.contains("grid") {
        return LayoutPattern::Grid;
    }

    // Heuristic from aspect ratio and edge density
    if aspect > 2.0 {
        LayoutPattern::FullWidth
    } else if aspect > 1.4 && edge_density > 0.5 {
        LayoutPattern::ThreeColumn
    } else if aspect > 1.2 && edge_density > 0.3 {
        LayoutPattern::TwoColumn
    } else if aspect < 0.7 {
        LayoutPattern::SingleColumn
    } else if edge_density < 0.2 {
        LayoutPattern::Centered
    } else {
        LayoutPattern::FreeForm
    }
}

/// Compute text density heuristic from labels and description.
fn compute_text_density(labels: &[String], description: &Option<String>) -> f64 {
    let mut text_indicators = 0.0;
    let text_keywords = [
        "text",
        "paragraph",
        "content",
        "article",
        "heading",
        "title",
        "body",
        "code",
        "document",
        "blog",
        "post",
        "comment",
        "message",
    ];
    for label in labels {
        let lower = label.to_lowercase();
        for kw in &text_keywords {
            if lower.contains(kw) {
                text_indicators += 0.15;
            }
        }
    }
    if let Some(desc) = description {
        let word_count = desc
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 2)
            .count();
        text_indicators += (word_count as f64 * 0.02).min(0.4);
    }
    text_indicators.min(1.0)
}

/// Compute complexity score from edge density, color entropy, and label count.
fn compute_complexity(
    edge_density: f64,
    color_entropy: f64,
    label_count: usize,
    block_count: u32,
) -> f64 {
    let edge_component = edge_density * 0.3;
    let color_component = (color_entropy / 2.08).min(1.0) * 0.25; // max entropy for 8 bins = ln(8) ~ 2.08
    let label_component = (label_count as f64 * 0.05).min(0.25);
    let block_component = (block_count as f64 / 20.0).min(1.0) * 0.2;
    (edge_component + color_component + label_component + block_component).min(1.0)
}

/// Build a VisualDNA from observation data.
fn build_visual_dna(
    capture_id: u64,
    embedding: &[f32],
    width: u32,
    height: u32,
    labels: &[String],
    description: &Option<String>,
) -> VisualDNA {
    let color_profile = compute_color_histogram(embedding, width, height);
    let edge_profile = compute_edge_density(embedding, width, height);
    let layout_pattern = detect_layout_pattern(width, height, labels, edge_profile.overall_density);
    let text_density = compute_text_density(labels, description);
    let aspect_ratio = if height > 0 {
        (width as f64 / height as f64 * 100.0).round() / 100.0
    } else {
        1.0
    };
    let complexity_score = compute_complexity(
        edge_profile.overall_density,
        color_profile.entropy,
        labels.len(),
        edge_profile.block_count,
    );

    let mut sorted_labels = labels.to_vec();
    sorted_labels.sort();

    // Build feature vector for fingerprint hash
    let features: Vec<f64> = vec![
        color_profile.avg_saturation,
        color_profile.avg_brightness,
        color_profile.entropy,
        edge_profile.overall_density,
        edge_profile.h_v_ratio,
        text_density,
        aspect_ratio,
        complexity_score,
    ];
    let fingerprint_hash = feature_hash(&features);

    VisualDNA {
        capture_id,
        color_profile,
        edge_profile,
        layout_pattern: layout_pattern.label().to_string(),
        text_density: (text_density * 100.0).round() / 100.0,
        aspect_ratio,
        complexity_score: (complexity_score * 100.0).round() / 100.0,
        label_signature: sorted_labels,
        fingerprint_hash,
    }
}

/// Compute distance between two color histograms (chi-squared distance).
fn color_histogram_distance(a: &ColorHistogram, b: &ColorHistogram) -> f64 {
    let mut chi_sq = 0.0;
    for i in 0..8 {
        let sum = a.hue_bins[i] + b.hue_bins[i];
        if sum > 1e-10 {
            let diff = a.hue_bins[i] - b.hue_bins[i];
            chi_sq += (diff * diff) / sum;
        }
    }
    // Also factor in saturation and brightness differences
    let sat_diff = (a.avg_saturation - b.avg_saturation).abs();
    let bright_diff = (a.avg_brightness - b.avg_brightness).abs();
    let entropy_diff = (a.entropy - b.entropy).abs() / 2.08; // normalize

    (chi_sq * 0.5 + sat_diff * 0.2 + bright_diff * 0.15 + entropy_diff * 0.15).min(1.0)
}

/// Compute structural distance between two edge density profiles.
fn edge_profile_distance(a: &EdgeDensityProfile, b: &EdgeDensityProfile) -> f64 {
    let overall_diff = (a.overall_density - b.overall_density).abs();
    let mut quad_diff = 0.0;
    for i in 0..4 {
        quad_diff += (a.quadrant_density[i] - b.quadrant_density[i]).powi(2);
    }
    quad_diff = (quad_diff / 4.0).sqrt();
    let hv_diff = if a.h_v_ratio > 0.0 && b.h_v_ratio > 0.0 {
        ((a.h_v_ratio.ln() - b.h_v_ratio.ln()).abs() / 2.0).min(1.0)
    } else {
        0.5
    };
    let block_diff = (a.block_count as f64 - b.block_count as f64).abs()
        / (a.block_count.max(b.block_count).max(1) as f64);

    overall_diff * 0.3 + quad_diff * 0.3 + hv_diff * 0.2 + block_diff * 0.2
}

/// Compute overall DNA distance between two VisualDNA profiles.
fn dna_distance(a: &VisualDNA, b: &VisualDNA) -> f64 {
    let color_dist = color_histogram_distance(&a.color_profile, &b.color_profile);
    let edge_dist = edge_profile_distance(&a.edge_profile, &b.edge_profile);
    let layout_dist = if a.layout_pattern == b.layout_pattern {
        0.0
    } else {
        0.6
    };
    let text_dist = (a.text_density - b.text_density).abs();
    let aspect_dist = ((a.aspect_ratio - b.aspect_ratio).abs()
        / a.aspect_ratio.max(b.aspect_ratio).max(0.01))
    .min(1.0);
    let complexity_dist = (a.complexity_score - b.complexity_score).abs();

    // Label overlap (Jaccard distance)
    let a_set: std::collections::HashSet<&String> = a.label_signature.iter().collect();
    let b_set: std::collections::HashSet<&String> = b.label_signature.iter().collect();
    let union_size = a_set.union(&b_set).count();
    let intersect_size = a_set.intersection(&b_set).count();
    let label_dist = if union_size > 0 {
        1.0 - (intersect_size as f64 / union_size as f64)
    } else {
        0.5
    };

    color_dist * 0.20
        + edge_dist * 0.20
        + layout_dist * 0.15
        + text_dist * 0.10
        + aspect_dist * 0.10
        + complexity_dist * 0.10
        + label_dist * 0.15
}

/// Extract feature vector from a VisualDNA for clustering.
fn dna_to_features(dna: &VisualDNA) -> Vec<f64> {
    let mut features = Vec::with_capacity(20);
    // Color histogram (8 bins)
    features.extend_from_slice(&dna.color_profile.hue_bins);
    // Color summary
    features.push(dna.color_profile.avg_saturation);
    features.push(dna.color_profile.avg_brightness);
    features.push(dna.color_profile.entropy);
    // Edge features
    features.push(dna.edge_profile.overall_density);
    features.extend_from_slice(&dna.edge_profile.quadrant_density);
    features.push(dna.edge_profile.h_v_ratio.min(5.0) / 5.0); // normalize
    features.push(dna.edge_profile.block_count as f64 / 50.0); // normalize
                                                               // Other features
    features.push(dna.text_density);
    features.push(dna.aspect_ratio.min(3.0) / 3.0); // normalize
    features.push(dna.complexity_score);
    features
}

/// Euclidean distance between two feature vectors.
fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    let min_len = a.len().min(b.len());
    let sum: f64 = (0..min_len).map(|i| (a[i] - b[i]).powi(2)).sum();
    sum.sqrt()
}

/// K-means clustering algorithm.
fn kmeans_cluster(
    features: &[Vec<f64>],
    k: usize,
    max_iterations: usize,
) -> (Vec<usize>, Vec<Vec<f64>>) {
    let n = features.len();
    if n == 0 || k == 0 {
        return (Vec::new(), Vec::new());
    }
    let k = k.min(n);
    let dim = features[0].len();

    // Initialize centroids using k-means++ initialization
    let mut centroids: Vec<Vec<f64>> = Vec::with_capacity(k);
    // First centroid: first data point
    centroids.push(features[0].clone());

    for _ in 1..k {
        // Compute distances to nearest centroid for each point
        let mut distances: Vec<f64> = features
            .iter()
            .map(|point| {
                centroids
                    .iter()
                    .map(|c| euclidean_distance(point, c))
                    .fold(f64::MAX, f64::min)
            })
            .collect();

        // Choose next centroid: point with maximum min-distance (deterministic)
        let total: f64 = distances.iter().sum();
        if total < 1e-10 {
            // All points are identical; duplicate centroid
            centroids.push(features[0].clone());
            continue;
        }

        // Normalize to probabilities
        for d in &mut distances {
            *d /= total;
        }

        // Pick the point with maximum distance (deterministic k-means++ variant)
        let max_idx = distances
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        centroids.push(features[max_idx].clone());
    }

    let mut assignments = vec![0usize; n];

    for _iter in 0..max_iterations {
        // Assignment step
        let mut changed = false;
        for (i, point) in features.iter().enumerate() {
            let nearest = centroids
                .iter()
                .enumerate()
                .map(|(ci, c)| (ci, euclidean_distance(point, c)))
                .min_by(|(_, da), (_, db)| da.partial_cmp(db).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(ci, _)| ci)
                .unwrap_or(0);
            if assignments[i] != nearest {
                assignments[i] = nearest;
                changed = true;
            }
        }

        if !changed {
            break;
        }

        // Update step
        let mut new_centroids = vec![vec![0.0f64; dim]; k];
        let mut counts = vec![0usize; k];
        for (i, point) in features.iter().enumerate() {
            let c = assignments[i];
            counts[c] += 1;
            for (j, &val) in point.iter().enumerate() {
                new_centroids[c][j] += val;
            }
        }
        for c in 0..k {
            if counts[c] > 0 {
                for j in 0..dim {
                    new_centroids[c][j] /= counts[c] as f64;
                }
            } else {
                // Empty cluster: keep old centroid
                new_centroids[c] = centroids[c].clone();
            }
        }
        centroids = new_centroids;
    }

    (assignments, centroids)
}

/// Compute silhouette score for a clustering result.
fn silhouette_score(features: &[Vec<f64>], assignments: &[usize], k: usize) -> f64 {
    let n = features.len();
    if n < 2 || k < 2 {
        return 0.0;
    }

    let mut total_silhouette = 0.0;
    let mut valid_count = 0;

    for i in 0..n {
        let cluster_i = assignments[i];

        // a(i): average distance to points in same cluster
        let mut same_cluster_dist_sum = 0.0;
        let mut same_cluster_count = 0;
        for j in 0..n {
            if i != j && assignments[j] == cluster_i {
                same_cluster_dist_sum += euclidean_distance(&features[i], &features[j]);
                same_cluster_count += 1;
            }
        }
        if same_cluster_count == 0 {
            continue; // singleton cluster
        }
        let a_i = same_cluster_dist_sum / same_cluster_count as f64;

        // b(i): minimum average distance to any other cluster
        let mut b_i = f64::MAX;
        for c in 0..k {
            if c == cluster_i {
                continue;
            }
            let mut other_dist_sum = 0.0;
            let mut other_count = 0;
            for j in 0..n {
                if assignments[j] == c {
                    other_dist_sum += euclidean_distance(&features[i], &features[j]);
                    other_count += 1;
                }
            }
            if other_count > 0 {
                let avg = other_dist_sum / other_count as f64;
                if avg < b_i {
                    b_i = avg;
                }
            }
        }
        if b_i == f64::MAX {
            continue;
        }

        let max_ab = a_i.max(b_i);
        if max_ab > 1e-10 {
            total_silhouette += (b_i - a_i) / max_ab;
            valid_count += 1;
        }
    }

    if valid_count > 0 {
        total_silhouette / valid_count as f64
    } else {
        0.0
    }
}

/// Compute visual weight for a capture quadrant from edge density + embedding.
fn compute_visual_weight(quadrant_density: f64, embedding_segment: &[f32]) -> f64 {
    let energy: f64 = embedding_segment
        .iter()
        .map(|&v| (v as f64).powi(2))
        .sum::<f64>()
        / embedding_segment.len().max(1) as f64;
    let weight = quadrant_density * 0.6 + energy.sqrt() * 0.4;
    (weight * 100.0).round() / 100.0
}

/// Compute balance score from 4 quadrant weights.
/// Perfect balance = 1.0, complete imbalance = 0.0.
fn compute_balance_score(weights: [f64; 4]) -> f64 {
    let mean = weights.iter().sum::<f64>() / 4.0;
    if mean < 1e-10 {
        return 1.0; // All zeros is balanced
    }
    let variance = weights.iter().map(|&w| (w - mean).powi(2)).sum::<f64>() / 4.0;
    let cv = variance.sqrt() / mean; // coefficient of variation
    (1.0 - cv.min(1.0)).max(0.0)
}

// =========================================================================
// INVENTION 17: Visual DNA — 4 tools
// =========================================================================

// -- vision_dna_extract ---------------------------------------------------

pub fn definition_vision_dna_extract() -> ToolDefinition {
    ToolDefinition {
        name: "vision_dna_extract".to_string(),
        description: Some(
            "Extract visual fingerprint/DNA from a capture including color histogram, edge density, layout patterns, and text density"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture ID to extract DNA from" }
            }
        }),
    }
}

pub async fn execute_vision_dna_extract(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs = match store.observations.iter().find(|o| o.id == p.capture_id) {
        Some(o) => o,
        None => return Err(McpError::CaptureNotFound(p.capture_id)),
    };

    let dna = build_visual_dna(
        obs.id,
        &obs.embedding,
        obs.metadata.width,
        obs.metadata.height,
        &obs.metadata.labels,
        &obs.metadata.description,
    );

    Ok(ToolCallResult::json(&json!({
        "capture_id": dna.capture_id,
        "fingerprint_hash": format!("{:016x}", dna.fingerprint_hash),
        "color_profile": {
            "hue_distribution": dna.color_profile.hue_bins.iter()
                .enumerate()
                .map(|(i, &v)| json!({"bin": i, "weight": (v * 1000.0).round() / 1000.0}))
                .collect::<Vec<_>>(),
            "dominant_hue_bin": dna.color_profile.dominant_hue,
            "avg_saturation": (dna.color_profile.avg_saturation * 100.0).round() / 100.0,
            "avg_brightness": (dna.color_profile.avg_brightness * 100.0).round() / 100.0,
            "color_entropy": (dna.color_profile.entropy * 100.0).round() / 100.0,
        },
        "edge_profile": {
            "overall_density": (dna.edge_profile.overall_density * 100.0).round() / 100.0,
            "quadrant_density": {
                "top_left": (dna.edge_profile.quadrant_density[0] * 100.0).round() / 100.0,
                "top_right": (dna.edge_profile.quadrant_density[1] * 100.0).round() / 100.0,
                "bottom_left": (dna.edge_profile.quadrant_density[2] * 100.0).round() / 100.0,
                "bottom_right": (dna.edge_profile.quadrant_density[3] * 100.0).round() / 100.0,
            },
            "horizontal_vertical_ratio": (dna.edge_profile.h_v_ratio * 100.0).round() / 100.0,
            "estimated_blocks": dna.edge_profile.block_count,
        },
        "layout_pattern": dna.layout_pattern,
        "text_density": dna.text_density,
        "aspect_ratio": dna.aspect_ratio,
        "complexity_score": dna.complexity_score,
        "label_signature": dna.label_signature,
    })))
}

// -- vision_dna_compare ---------------------------------------------------

pub fn definition_vision_dna_compare() -> ToolDefinition {
    ToolDefinition {
        name: "vision_dna_compare".to_string(),
        description: Some(
            "Compare visual DNA profiles between two captures with structural similarity, color distance, and layout divergence scoring"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["capture_a", "capture_b"],
            "properties": {
                "capture_a": { "type": "number", "description": "First capture ID" },
                "capture_b": { "type": "number", "description": "Second capture ID" }
            }
        }),
    }
}

pub async fn execute_vision_dna_compare(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_a: u64,
        capture_b: u64,
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

    let dna_a = build_visual_dna(
        obs_a.id,
        &obs_a.embedding,
        obs_a.metadata.width,
        obs_a.metadata.height,
        &obs_a.metadata.labels,
        &obs_a.metadata.description,
    );
    let dna_b = build_visual_dna(
        obs_b.id,
        &obs_b.embedding,
        obs_b.metadata.width,
        obs_b.metadata.height,
        &obs_b.metadata.labels,
        &obs_b.metadata.description,
    );

    let overall_distance = dna_distance(&dna_a, &dna_b);
    let color_dist = color_histogram_distance(&dna_a.color_profile, &dna_b.color_profile);
    let edge_dist = edge_profile_distance(&dna_a.edge_profile, &dna_b.edge_profile);
    let layout_match = dna_a.layout_pattern == dna_b.layout_pattern;

    let similarity = ((1.0 - overall_distance) * 100.0).round() / 100.0;
    let assessment = if similarity > 0.85 {
        "nearly_identical"
    } else if similarity > 0.65 {
        "similar"
    } else if similarity > 0.40 {
        "somewhat_similar"
    } else if similarity > 0.20 {
        "different"
    } else {
        "very_different"
    };

    Ok(ToolCallResult::json(&json!({
        "capture_a": p.capture_a,
        "capture_b": p.capture_b,
        "overall_similarity": similarity,
        "overall_distance": (overall_distance * 100.0).round() / 100.0,
        "assessment": assessment,
        "component_distances": {
            "color_distance": (color_dist * 100.0).round() / 100.0,
            "edge_distance": (edge_dist * 100.0).round() / 100.0,
            "layout_match": layout_match,
            "text_density_diff": ((dna_a.text_density - dna_b.text_density).abs() * 100.0).round() / 100.0,
            "complexity_diff": ((dna_a.complexity_score - dna_b.complexity_score).abs() * 100.0).round() / 100.0,
            "aspect_ratio_diff": ((dna_a.aspect_ratio - dna_b.aspect_ratio).abs() * 100.0).round() / 100.0,
        },
        "dna_a": {
            "fingerprint": format!("{:016x}", dna_a.fingerprint_hash),
            "layout": dna_a.layout_pattern,
            "complexity": dna_a.complexity_score,
        },
        "dna_b": {
            "fingerprint": format!("{:016x}", dna_b.fingerprint_hash),
            "layout": dna_b.layout_pattern,
            "complexity": dna_b.complexity_score,
        },
    })))
}

// -- vision_dna_lineage ---------------------------------------------------

pub fn definition_vision_dna_lineage() -> ToolDefinition {
    ToolDefinition {
        name: "vision_dna_lineage".to_string(),
        description: Some(
            "Trace visual DNA evolution across captures in a session to detect how UI changed over time"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "number", "description": "Session to trace (default: current)" },
                "max_captures": { "type": "number", "description": "Max captures to analyze", "default": 50 }
            }
        }),
    }
}

pub async fn execute_vision_dna_lineage(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        session_id: Option<u32>,
        #[serde(default = "default_max_captures")]
        max_captures: usize,
    }
    fn default_max_captures() -> usize {
        50
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session_guard = session.lock().await;
    let store = session_guard.store();

    // Collect observations for the session, sorted by timestamp
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
            "lineage": [],
            "total_captures": 0,
            "evolution_summary": "No captures found for this session",
        })));
    }

    // Build DNA for each capture
    let dna_list: Vec<VisualDNA> = obs_list
        .iter()
        .map(|obs| {
            build_visual_dna(
                obs.id,
                &obs.embedding,
                obs.metadata.width,
                obs.metadata.height,
                &obs.metadata.labels,
                &obs.metadata.description,
            )
        })
        .collect();

    // Compute lineage transitions
    let mut transitions: Vec<Value> = Vec::new();
    let mut total_drift = 0.0;
    let mut max_drift = 0.0f64;
    let mut layout_changes = 0;

    for i in 1..dna_list.len() {
        let prev = &dna_list[i - 1];
        let curr = &dna_list[i];
        let distance = dna_distance(prev, curr);
        total_drift += distance;
        max_drift = max_drift.max(distance);

        let layout_changed = prev.layout_pattern != curr.layout_pattern;
        if layout_changed {
            layout_changes += 1;
        }

        let transition_type = if distance < 0.05 {
            "stable"
        } else if distance < 0.15 {
            "minor_change"
        } else if distance < 0.35 {
            "moderate_change"
        } else if distance < 0.60 {
            "major_change"
        } else {
            "radical_change"
        };

        transitions.push(json!({
            "from_capture": prev.capture_id,
            "to_capture": curr.capture_id,
            "distance": (distance * 100.0).round() / 100.0,
            "transition_type": transition_type,
            "layout_changed": layout_changed,
            "complexity_delta": ((curr.complexity_score - prev.complexity_score) * 100.0).round() / 100.0,
            "color_entropy_delta": ((curr.color_profile.entropy - prev.color_profile.entropy) * 100.0).round() / 100.0,
        }));
    }

    let avg_drift = if !transitions.is_empty() {
        total_drift / transitions.len() as f64
    } else {
        0.0
    };

    let stability = if avg_drift < 0.05 {
        "very_stable"
    } else if avg_drift < 0.15 {
        "stable"
    } else if avg_drift < 0.30 {
        "moderate_evolution"
    } else if avg_drift < 0.50 {
        "rapid_evolution"
    } else {
        "volatile"
    };

    Ok(ToolCallResult::json(&json!({
        "total_captures": dna_list.len(),
        "transitions": transitions,
        "evolution_metrics": {
            "average_drift": (avg_drift * 100.0).round() / 100.0,
            "max_drift": (max_drift * 100.0).round() / 100.0,
            "total_drift": (total_drift * 100.0).round() / 100.0,
            "layout_changes": layout_changes,
            "stability_assessment": stability,
        },
        "first_dna": {
            "capture_id": dna_list[0].capture_id,
            "fingerprint": format!("{:016x}", dna_list[0].fingerprint_hash),
            "layout": &dna_list[0].layout_pattern,
            "complexity": dna_list[0].complexity_score,
        },
        "last_dna": {
            "capture_id": dna_list.last().unwrap().capture_id,
            "fingerprint": format!("{:016x}", dna_list.last().unwrap().fingerprint_hash),
            "layout": &dna_list.last().unwrap().layout_pattern,
            "complexity": dna_list.last().unwrap().complexity_score,
        },
    })))
}

// -- vision_dna_mutate ----------------------------------------------------

pub fn definition_vision_dna_mutate() -> ToolDefinition {
    ToolDefinition {
        name: "vision_dna_mutate".to_string(),
        description: Some(
            "Detect mutations/unexpected changes in visual patterns by comparing a capture against a baseline DNA"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id", "baseline_capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Current capture to check for mutations" },
                "baseline_capture_id": { "type": "number", "description": "Baseline capture to compare against" },
                "mutation_threshold": { "type": "number", "description": "Distance threshold to flag mutation (0.0-1.0)", "default": 0.2 }
            }
        }),
    }
}

pub async fn execute_vision_dna_mutate(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
        baseline_capture_id: u64,
        #[serde(default = "default_mutation_threshold")]
        mutation_threshold: f64,
    }
    fn default_mutation_threshold() -> f64 {
        0.2
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs_current = store
        .observations
        .iter()
        .find(|o| o.id == p.capture_id)
        .ok_or(McpError::CaptureNotFound(p.capture_id))?;
    let obs_baseline = store
        .observations
        .iter()
        .find(|o| o.id == p.baseline_capture_id)
        .ok_or(McpError::CaptureNotFound(p.baseline_capture_id))?;

    let dna_current = build_visual_dna(
        obs_current.id,
        &obs_current.embedding,
        obs_current.metadata.width,
        obs_current.metadata.height,
        &obs_current.metadata.labels,
        &obs_current.metadata.description,
    );
    let dna_baseline = build_visual_dna(
        obs_baseline.id,
        &obs_baseline.embedding,
        obs_baseline.metadata.width,
        obs_baseline.metadata.height,
        &obs_baseline.metadata.labels,
        &obs_baseline.metadata.description,
    );

    let distance = dna_distance(&dna_current, &dna_baseline);
    let color_dist =
        color_histogram_distance(&dna_current.color_profile, &dna_baseline.color_profile);
    let edge_dist = edge_profile_distance(&dna_current.edge_profile, &dna_baseline.edge_profile);
    let threshold = p.mutation_threshold.max(0.01).min(1.0);
    let is_mutated = distance > threshold;

    // Identify specific mutations
    let mut mutations: Vec<Value> = Vec::new();
    if color_dist > threshold {
        mutations.push(json!({
            "type": "color_mutation",
            "severity": if color_dist > 0.5 { "high" } else { "moderate" },
            "distance": (color_dist * 100.0).round() / 100.0,
            "detail": format!("Color profile diverged by {:.1}%", color_dist * 100.0),
        }));
    }
    if edge_dist > threshold {
        mutations.push(json!({
            "type": "structure_mutation",
            "severity": if edge_dist > 0.5 { "high" } else { "moderate" },
            "distance": (edge_dist * 100.0).round() / 100.0,
            "detail": format!("Edge structure diverged by {:.1}%", edge_dist * 100.0),
        }));
    }
    if dna_current.layout_pattern != dna_baseline.layout_pattern {
        mutations.push(json!({
            "type": "layout_mutation",
            "severity": "high",
            "from": dna_baseline.layout_pattern,
            "to": dna_current.layout_pattern,
            "detail": format!("Layout changed from {} to {}", dna_baseline.layout_pattern, dna_current.layout_pattern),
        }));
    }
    let complexity_diff = (dna_current.complexity_score - dna_baseline.complexity_score).abs();
    if complexity_diff > threshold {
        mutations.push(json!({
            "type": "complexity_mutation",
            "severity": if complexity_diff > 0.4 { "high" } else { "moderate" },
            "delta": ((dna_current.complexity_score - dna_baseline.complexity_score) * 100.0).round() / 100.0,
            "detail": format!("Complexity changed by {:.1}%", complexity_diff * 100.0),
        }));
    }
    let text_diff = (dna_current.text_density - dna_baseline.text_density).abs();
    if text_diff > threshold {
        mutations.push(json!({
            "type": "text_density_mutation",
            "severity": if text_diff > 0.3 { "high" } else { "moderate" },
            "delta": ((dna_current.text_density - dna_baseline.text_density) * 100.0).round() / 100.0,
            "detail": format!("Text density changed by {:.1}%", text_diff * 100.0),
        }));
    }

    // Label mutations
    let baseline_labels: std::collections::HashSet<&String> =
        dna_baseline.label_signature.iter().collect();
    let current_labels: std::collections::HashSet<&String> =
        dna_current.label_signature.iter().collect();
    let added: Vec<&&String> = current_labels.difference(&baseline_labels).collect();
    let removed: Vec<&&String> = baseline_labels.difference(&current_labels).collect();
    if !added.is_empty() || !removed.is_empty() {
        mutations.push(json!({
            "type": "label_mutation",
            "severity": "moderate",
            "labels_added": added,
            "labels_removed": removed,
        }));
    }

    let severity = if distance > 0.7 {
        "critical"
    } else if distance > 0.5 {
        "severe"
    } else if distance > threshold {
        "moderate"
    } else {
        "none"
    };

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "baseline_capture_id": p.baseline_capture_id,
        "is_mutated": is_mutated,
        "overall_distance": (distance * 100.0).round() / 100.0,
        "threshold": threshold,
        "severity": severity,
        "mutation_count": mutations.len(),
        "mutations": mutations,
        "fingerprints": {
            "current": format!("{:016x}", dna_current.fingerprint_hash),
            "baseline": format!("{:016x}", dna_baseline.fingerprint_hash),
            "match": dna_current.fingerprint_hash == dna_baseline.fingerprint_hash,
        },
    })))
}

// =========================================================================
// INVENTION 18: Visual Composition — 4 tools
// =========================================================================

// -- vision_composition_analyze -------------------------------------------

pub fn definition_vision_composition_analyze() -> ToolDefinition {
    ToolDefinition {
        name: "vision_composition_analyze".to_string(),
        description: Some(
            "Analyze visual layout composition including grid detection, alignment patterns, whitespace balance, and visual weight distribution"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture to analyze" }
            }
        }),
    }
}

pub async fn execute_vision_composition_analyze(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs = store
        .observations
        .iter()
        .find(|o| o.id == p.capture_id)
        .ok_or(McpError::CaptureNotFound(p.capture_id))?;

    let edge = compute_edge_density(&obs.embedding, obs.metadata.width, obs.metadata.height);
    let color = compute_color_histogram(&obs.embedding, obs.metadata.width, obs.metadata.height);

    // Grid alignment detection
    let grid = detect_grid_alignment(&obs.embedding, &edge);

    // Visual weight map
    let weight_map = compute_weight_map(&obs.embedding, &edge);

    // Whitespace estimation
    let whitespace = estimate_whitespace(&edge, obs.metadata.width, obs.metadata.height);

    // Alignment score
    let alignment_score = compute_alignment_score(&edge, &grid);

    // Layout pattern
    let layout = detect_layout_pattern(
        obs.metadata.width,
        obs.metadata.height,
        &obs.metadata.labels,
        edge.overall_density,
    );

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "dimensions": {
            "width": obs.metadata.width,
            "height": obs.metadata.height,
            "aspect_ratio": if obs.metadata.height > 0 {
                (obs.metadata.width as f64 / obs.metadata.height as f64 * 100.0).round() / 100.0
            } else { 1.0 },
        },
        "grid": {
            "detected_columns": grid.columns,
            "detected_rows": grid.rows,
            "alignment_score": (grid.alignment_score * 100.0).round() / 100.0,
            "gutter_consistency": (grid.gutter_consistency * 100.0).round() / 100.0,
        },
        "visual_weight": {
            "top_left": weight_map.top_left,
            "top_right": weight_map.top_right,
            "bottom_left": weight_map.bottom_left,
            "bottom_right": weight_map.bottom_right,
            "center_of_mass_x": (weight_map.center_of_mass.0 * 100.0).round() / 100.0,
            "center_of_mass_y": (weight_map.center_of_mass.1 * 100.0).round() / 100.0,
            "balance_score": (weight_map.balance_score * 100.0).round() / 100.0,
        },
        "whitespace": {
            "ratio": (whitespace * 100.0).round() / 100.0,
            "assessment": if whitespace > 0.6 { "spacious" }
                else if whitespace > 0.3 { "balanced" }
                else { "dense" },
        },
        "alignment_score": (alignment_score * 100.0).round() / 100.0,
        "layout_pattern": layout.label(),
        "edge_density": (edge.overall_density * 100.0).round() / 100.0,
        "color_entropy": (color.entropy * 100.0).round() / 100.0,
    })))
}

/// Detect grid alignment from embedding and edge density.
fn detect_grid_alignment(embedding: &[f32], edge: &EdgeDensityProfile) -> GridAlignment {
    let dim = embedding.len();
    if dim < 8 {
        return GridAlignment {
            columns: 1,
            rows: 1,
            alignment_score: 0.5,
            gutter_consistency: 0.5,
        };
    }

    // Estimate columns from horizontal periodicity in the embedding
    let half = dim / 2;
    let mut best_period = 1;
    let mut best_correlation = 0.0f64;

    for period in 2..=6 {
        if period > half {
            break;
        }
        let mut correlation = 0.0;
        let mut count = 0;
        for i in 0..half.min(period * 8) {
            if i + period < dim {
                let similarity = 1.0
                    - (embedding[i] as f64 - embedding[i + period] as f64).abs()
                        / (embedding[i].abs().max(embedding[i + period].abs()) as f64 + 1e-10);
                correlation += similarity;
                count += 1;
            }
        }
        if count > 0 {
            correlation /= count as f64;
            if correlation > best_correlation {
                best_correlation = correlation;
                best_period = period;
            }
        }
    }

    let columns = if best_correlation > 0.6 {
        best_period as u32
    } else if edge.h_v_ratio > 1.5 {
        3
    } else if edge.h_v_ratio > 1.1 {
        2
    } else {
        1
    };

    // Estimate rows from vertical periodicity (second half of embedding)
    let rows = {
        let v_energy: f64 = edge.quadrant_density[0..2].iter().sum::<f64>()
            + edge.quadrant_density[2..4].iter().sum::<f64>();
        let v_diff = (edge.quadrant_density[0] + edge.quadrant_density[1])
            - (edge.quadrant_density[2] + edge.quadrant_density[3]);
        if v_diff.abs() < 0.2 && v_energy > 1.0 {
            3u32
        } else if v_energy > 0.5 {
            2
        } else {
            1
        }
    };

    let alignment_score = best_correlation.min(1.0);

    // Gutter consistency: how regular are the breaks
    let gutter_consistency = if columns > 1 {
        let mut regularity = 0.0f64;
        let chunk = dim / columns as usize;
        for c in 1..columns as usize {
            let boundary = c * chunk;
            if boundary < dim && boundary > 0 {
                let diff = (embedding[boundary] as f64 - embedding[boundary - 1] as f64).abs();
                regularity += (diff * 2.0).min(1.0);
            }
        }
        regularity / (columns - 1).max(1) as f64
    } else {
        0.5
    };

    GridAlignment {
        columns,
        rows,
        alignment_score: (alignment_score * 100.0).round() / 100.0,
        gutter_consistency: (gutter_consistency * 100.0).round() / 100.0,
    }
}

/// Compute visual weight map from embedding and edge density.
fn compute_weight_map(embedding: &[f32], edge: &EdgeDensityProfile) -> VisualWeightMap {
    let dim = embedding.len();
    let quarter = dim / 4;

    let weights: [f64; 4] = if quarter > 0 {
        [
            compute_visual_weight(edge.quadrant_density[0], &embedding[..quarter]),
            compute_visual_weight(edge.quadrant_density[1], &embedding[quarter..quarter * 2]),
            compute_visual_weight(
                edge.quadrant_density[2],
                &embedding[quarter * 2..quarter * 3],
            ),
            compute_visual_weight(edge.quadrant_density[3], &embedding[quarter * 3..]),
        ]
    } else {
        [0.25; 4]
    };

    let total_weight = weights.iter().sum::<f64>().max(1e-10);
    let cx = (weights[1] + weights[3]) / total_weight; // right-weighted
    let cy = (weights[2] + weights[3]) / total_weight; // bottom-weighted
    let balance = compute_balance_score(weights);

    VisualWeightMap {
        top_left: weights[0],
        top_right: weights[1],
        bottom_left: weights[2],
        bottom_right: weights[3],
        center_of_mass: (cx, cy),
        balance_score: balance,
    }
}

/// Estimate whitespace ratio from edge density.
fn estimate_whitespace(edge: &EdgeDensityProfile, _width: u32, _height: u32) -> f64 {
    // Whitespace is inverse of edge density, with adjustment for block count
    let base = 1.0 - edge.overall_density;
    let block_factor = 1.0 - (edge.block_count as f64 / 30.0).min(0.5);
    (base * 0.7 + block_factor * 0.3).min(1.0).max(0.0)
}

/// Compute alignment score from edge density and grid.
fn compute_alignment_score(edge: &EdgeDensityProfile, grid: &GridAlignment) -> f64 {
    let grid_factor = grid.alignment_score * 0.4;
    let gutter_factor = grid.gutter_consistency * 0.3;
    // Quadrant balance indicates consistent spacing
    let balance = compute_balance_score(edge.quadrant_density);
    let balance_factor = balance * 0.3;
    (grid_factor + gutter_factor + balance_factor).min(1.0)
}

// -- vision_composition_score ---------------------------------------------

pub fn definition_vision_composition_score() -> ToolDefinition {
    ToolDefinition {
        name: "vision_composition_score".to_string(),
        description: Some(
            "Score overall visual composition quality including balance, contrast, hierarchy, and consistency metrics"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture to score" }
            }
        }),
    }
}

pub async fn execute_vision_composition_score(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs = store
        .observations
        .iter()
        .find(|o| o.id == p.capture_id)
        .ok_or(McpError::CaptureNotFound(p.capture_id))?;

    let edge = compute_edge_density(&obs.embedding, obs.metadata.width, obs.metadata.height);
    let color = compute_color_histogram(&obs.embedding, obs.metadata.width, obs.metadata.height);
    let grid = detect_grid_alignment(&obs.embedding, &edge);
    let weight_map = compute_weight_map(&obs.embedding, &edge);
    let whitespace = estimate_whitespace(&edge, obs.metadata.width, obs.metadata.height);

    // Balance score (from weight distribution)
    let balance = weight_map.balance_score;

    // Contrast score (from color entropy and edge density)
    let contrast = (color.entropy / 2.08 * 0.6 + edge.overall_density * 0.4).min(1.0);

    // Hierarchy score (heuristic: good hierarchy = clear edge density variation across quadrants)
    let quad_variance = {
        let mean = edge.quadrant_density.iter().sum::<f64>() / 4.0;
        let var = edge
            .quadrant_density
            .iter()
            .map(|&d| (d - mean).powi(2))
            .sum::<f64>()
            / 4.0;
        var.sqrt()
    };
    let hierarchy = (quad_variance * 2.0).min(1.0); // Some variation = good hierarchy

    // Consistency score (grid alignment + gutter regularity)
    let consistency = grid.alignment_score * 0.5 + grid.gutter_consistency * 0.5;

    // Whitespace score (not too dense, not too sparse)
    let whitespace_score = 1.0 - (whitespace - 0.4).abs() * 2.0; // Peak at 0.4
    let whitespace_score = whitespace_score.max(0.0).min(1.0);

    // Overall composition score (weighted average)
    let overall = balance * 0.25
        + contrast * 0.20
        + hierarchy * 0.20
        + consistency * 0.20
        + whitespace_score * 0.15;

    let quality = if overall > 0.75 {
        CompositionQuality::Excellent
    } else if overall > 0.60 {
        CompositionQuality::Good
    } else if overall > 0.45 {
        CompositionQuality::Fair
    } else if overall > 0.30 {
        CompositionQuality::Poor
    } else {
        CompositionQuality::Unbalanced
    };

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "overall_score": (overall * 100.0).round() / 100.0,
        "quality": quality.label(),
        "component_scores": {
            "balance": (balance * 100.0).round() / 100.0,
            "contrast": (contrast * 100.0).round() / 100.0,
            "hierarchy": (hierarchy * 100.0).round() / 100.0,
            "consistency": (consistency * 100.0).round() / 100.0,
            "whitespace": (whitespace_score * 100.0).round() / 100.0,
        },
        "details": {
            "grid_columns": grid.columns,
            "grid_rows": grid.rows,
            "whitespace_ratio": (whitespace * 100.0).round() / 100.0,
            "edge_density": (edge.overall_density * 100.0).round() / 100.0,
            "color_entropy": (color.entropy * 100.0).round() / 100.0,
            "center_of_mass": {
                "x": (weight_map.center_of_mass.0 * 100.0).round() / 100.0,
                "y": (weight_map.center_of_mass.1 * 100.0).round() / 100.0,
            },
        },
    })))
}

// -- vision_composition_suggest -------------------------------------------

pub fn definition_vision_composition_suggest() -> ToolDefinition {
    ToolDefinition {
        name: "vision_composition_suggest".to_string(),
        description: Some("Suggest improvements based on detected composition issues".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture to analyze for suggestions" }
            }
        }),
    }
}

pub async fn execute_vision_composition_suggest(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs = store
        .observations
        .iter()
        .find(|o| o.id == p.capture_id)
        .ok_or(McpError::CaptureNotFound(p.capture_id))?;

    let edge = compute_edge_density(&obs.embedding, obs.metadata.width, obs.metadata.height);
    let color = compute_color_histogram(&obs.embedding, obs.metadata.width, obs.metadata.height);
    let weight_map = compute_weight_map(&obs.embedding, &edge);
    let whitespace = estimate_whitespace(&edge, obs.metadata.width, obs.metadata.height);
    let grid = detect_grid_alignment(&obs.embedding, &edge);

    let mut suggestions: Vec<Value> = Vec::new();
    let mut priority_counter = 1;

    // Balance suggestions
    if weight_map.balance_score < 0.5 {
        let heaviest = [
            ("top_left", weight_map.top_left),
            ("top_right", weight_map.top_right),
            ("bottom_left", weight_map.bottom_left),
            ("bottom_right", weight_map.bottom_right),
        ];
        let max_quadrant = heaviest
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| *name)
            .unwrap_or("unknown");

        suggestions.push(json!({
            "priority": priority_counter,
            "area": "balance",
            "issue": format!("Visual weight is concentrated in the {} quadrant", max_quadrant),
            "suggestion": "Redistribute visual elements more evenly across the layout to improve balance",
            "impact": "high",
            "current_score": (weight_map.balance_score * 100.0).round() / 100.0,
        }));
        priority_counter += 1;
    }

    // Whitespace suggestions
    if whitespace < 0.2 {
        suggestions.push(json!({
            "priority": priority_counter,
            "area": "whitespace",
            "issue": "Layout is too dense with insufficient whitespace",
            "suggestion": "Increase padding and margins between elements to improve readability",
            "impact": "high",
            "current_ratio": (whitespace * 100.0).round() / 100.0,
        }));
        priority_counter += 1;
    } else if whitespace > 0.7 {
        suggestions.push(json!({
            "priority": priority_counter,
            "area": "whitespace",
            "issue": "Layout has excessive whitespace",
            "suggestion": "Consider making better use of available space with content or meaningful imagery",
            "impact": "moderate",
            "current_ratio": (whitespace * 100.0).round() / 100.0,
        }));
        priority_counter += 1;
    }

    // Contrast suggestions
    let contrast = color.entropy / 2.08;
    if contrast < 0.3 {
        suggestions.push(json!({
            "priority": priority_counter,
            "area": "contrast",
            "issue": "Low color variety reduces visual interest",
            "suggestion": "Introduce accent colors or increase contrast between foreground and background elements",
            "impact": "moderate",
            "color_entropy": (color.entropy * 100.0).round() / 100.0,
        }));
        priority_counter += 1;
    }

    // Grid alignment suggestions
    if grid.alignment_score < 0.4 {
        suggestions.push(json!({
            "priority": priority_counter,
            "area": "alignment",
            "issue": "Elements do not follow a consistent grid alignment",
            "suggestion": "Align elements to a consistent grid system for better visual organization",
            "impact": "high",
            "alignment_score": (grid.alignment_score * 100.0).round() / 100.0,
        }));
        priority_counter += 1;
    }

    // Hierarchy suggestions
    let quad_variation = {
        let mean = edge.quadrant_density.iter().sum::<f64>() / 4.0;
        edge.quadrant_density
            .iter()
            .map(|&d| (d - mean).powi(2))
            .sum::<f64>()
            / 4.0
    };
    if quad_variation < 0.01 {
        suggestions.push(json!({
            "priority": priority_counter,
            "area": "hierarchy",
            "issue": "Uniform visual density suggests weak visual hierarchy",
            "suggestion": "Create clear focal points by varying element sizes, weights, and spacing",
            "impact": "high",
        }));
        priority_counter += 1;
    }

    // Edge density suggestion
    if edge.overall_density > 0.8 {
        suggestions.push(json!({
            "priority": priority_counter,
            "area": "complexity",
            "issue": "High edge density indicates visual complexity",
            "suggestion": "Simplify the visual layout by reducing the number of distinct visual elements",
            "impact": "moderate",
            "edge_density": (edge.overall_density * 100.0).round() / 100.0,
        }));
        let _ = priority_counter; // suppress unused variable warning
    }

    if suggestions.is_empty() {
        suggestions.push(json!({
            "priority": 1,
            "area": "overall",
            "issue": "none",
            "suggestion": "Composition looks well-balanced. No significant issues detected",
            "impact": "none",
        }));
    }

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "suggestion_count": suggestions.len(),
        "suggestions": suggestions,
    })))
}

// -- vision_composition_compare -------------------------------------------

pub fn definition_vision_composition_compare() -> ToolDefinition {
    ToolDefinition {
        name: "vision_composition_compare".to_string(),
        description: Some("Compare composition quality across multiple captures".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_ids"],
            "properties": {
                "capture_ids": {
                    "type": "array",
                    "items": { "type": "number" },
                    "description": "List of capture IDs to compare"
                }
            }
        }),
    }
}

pub async fn execute_vision_composition_compare(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_ids: Vec<u64>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    if p.capture_ids.is_empty() {
        return Ok(ToolCallResult::json(&json!({
            "error": "No capture IDs provided"
        })));
    }

    let session = session.lock().await;
    let store = session.store();

    let mut scores: Vec<Value> = Vec::new();
    let mut best_score = 0.0f64;
    let mut best_id = 0u64;
    let mut worst_score = 1.0f64;
    let mut worst_id = 0u64;

    for &cid in &p.capture_ids {
        let obs = match store.observations.iter().find(|o| o.id == cid) {
            Some(o) => o,
            None => continue,
        };

        let edge = compute_edge_density(&obs.embedding, obs.metadata.width, obs.metadata.height);
        let color =
            compute_color_histogram(&obs.embedding, obs.metadata.width, obs.metadata.height);
        let grid = detect_grid_alignment(&obs.embedding, &edge);
        let weight_map = compute_weight_map(&obs.embedding, &edge);
        let whitespace = estimate_whitespace(&edge, obs.metadata.width, obs.metadata.height);

        let balance = weight_map.balance_score;
        let contrast = (color.entropy / 2.08 * 0.6 + edge.overall_density * 0.4).min(1.0);
        let consistency = grid.alignment_score * 0.5 + grid.gutter_consistency * 0.5;
        let whitespace_score = (1.0 - (whitespace - 0.4).abs() * 2.0).max(0.0).min(1.0);
        let overall =
            balance * 0.30 + contrast * 0.25 + consistency * 0.25 + whitespace_score * 0.20;

        if overall > best_score {
            best_score = overall;
            best_id = cid;
        }
        if overall < worst_score {
            worst_score = overall;
            worst_id = cid;
        }

        scores.push(json!({
            "capture_id": cid,
            "overall": (overall * 100.0).round() / 100.0,
            "balance": (balance * 100.0).round() / 100.0,
            "contrast": (contrast * 100.0).round() / 100.0,
            "consistency": (consistency * 100.0).round() / 100.0,
            "whitespace": (whitespace_score * 100.0).round() / 100.0,
        }));
    }

    // Sort by overall score descending
    scores.sort_by(|a, b| {
        b["overall"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["overall"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let avg_score: f64 = scores
        .iter()
        .map(|s| s["overall"].as_f64().unwrap_or(0.0))
        .sum::<f64>()
        / scores.len().max(1) as f64;

    Ok(ToolCallResult::json(&json!({
        "compared_count": scores.len(),
        "average_score": (avg_score * 100.0).round() / 100.0,
        "best": { "capture_id": best_id, "score": (best_score * 100.0).round() / 100.0 },
        "worst": { "capture_id": worst_id, "score": (worst_score * 100.0).round() / 100.0 },
        "rankings": scores,
    })))
}

// =========================================================================
// INVENTION 19: Visual Clustering — 3 tools
// =========================================================================

// -- vision_cluster_captures ----------------------------------------------

pub fn definition_vision_cluster_captures() -> ToolDefinition {
    ToolDefinition {
        name: "vision_cluster_captures".to_string(),
        description: Some(
            "Cluster similar captures using k-means on visual DNA features with silhouette scoring"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "k": { "type": "number", "description": "Number of clusters (default: auto-detect)", "default": 0 },
                "session_id": { "type": "number", "description": "Session to cluster (default: all)" },
                "max_captures": { "type": "number", "description": "Max captures to include", "default": 100 }
            }
        }),
    }
}

pub async fn execute_vision_cluster_captures(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        #[serde(default)]
        k: usize,
        session_id: Option<u32>,
        #[serde(default = "default_max_cluster")]
        max_captures: usize,
    }
    fn default_max_cluster() -> usize {
        100
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session_guard = session.lock().await;
    let store = session_guard.store();

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
            "error": "Need at least 2 captures to cluster",
            "capture_count": n,
        })));
    }

    // Build DNA and extract features
    let dna_list: Vec<VisualDNA> = obs_list
        .iter()
        .map(|obs| {
            build_visual_dna(
                obs.id,
                &obs.embedding,
                obs.metadata.width,
                obs.metadata.height,
                &obs.metadata.labels,
                &obs.metadata.description,
            )
        })
        .collect();
    let features: Vec<Vec<f64>> = dna_list.iter().map(|d| dna_to_features(d)).collect();

    // Auto-detect k if not specified (try 2 to sqrt(n), pick best silhouette)
    let k = if p.k > 0 {
        p.k.min(n)
    } else {
        let max_k = ((n as f64).sqrt().ceil() as usize).max(2).min(8);
        let mut best_k = 2;
        let mut best_sil = -1.0f64;
        for candidate_k in 2..=max_k {
            if candidate_k >= n {
                break;
            }
            let (assignments, _centroids) = kmeans_cluster(&features, candidate_k, 50);
            let sil = silhouette_score(&features, &assignments, candidate_k);
            if sil > best_sil {
                best_sil = sil;
                best_k = candidate_k;
            }
        }
        best_k
    };

    let (assignments, centroids) = kmeans_cluster(&features, k, 100);
    let sil = silhouette_score(&features, &assignments, k);

    // Build cluster info
    let mut cluster_members: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, &cluster) in assignments.iter().enumerate() {
        cluster_members.entry(cluster).or_default().push(i);
    }

    let mut cluster_info: Vec<Value> = Vec::new();
    for c in 0..k {
        let members = cluster_members.get(&c).cloned().unwrap_or_default();
        let member_ids: Vec<u64> = members.iter().map(|&i| dna_list[i].capture_id).collect();

        // Common labels in this cluster
        let mut label_counts: HashMap<&String, usize> = HashMap::new();
        for &i in &members {
            for label in &dna_list[i].label_signature {
                *label_counts.entry(label).or_insert(0) += 1;
            }
        }
        let mut common_labels: Vec<String> = label_counts
            .iter()
            .filter(|(_, &count)| count > members.len() / 2)
            .map(|(&label, _)| label.clone())
            .collect();
        common_labels.sort();

        // Average intra-cluster distance
        let avg_intra = if members.len() > 1 {
            let mut total = 0.0;
            let mut count = 0;
            for i in 0..members.len() {
                for j in (i + 1)..members.len() {
                    total += euclidean_distance(&features[members[i]], &features[members[j]]);
                    count += 1;
                }
            }
            if count > 0 {
                total / count as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Representative capture (closest to centroid)
        let representative = if !members.is_empty() && c < centroids.len() {
            members
                .iter()
                .min_by(|&&a, &&b| {
                    euclidean_distance(&features[a], &centroids[c])
                        .partial_cmp(&euclidean_distance(&features[b], &centroids[c]))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|&i| dna_list[i].capture_id)
        } else {
            None
        };

        cluster_info.push(json!({
            "cluster_id": c,
            "member_count": members.len(),
            "member_capture_ids": member_ids,
            "representative_capture_id": representative,
            "common_labels": common_labels,
            "avg_intra_distance": (avg_intra * 100.0).round() / 100.0,
            "cohesion": if avg_intra < 0.5 { "tight" } else if avg_intra < 1.0 { "moderate" } else { "loose" },
        }));
    }

    Ok(ToolCallResult::json(&json!({
        "total_captures": n,
        "k": k,
        "silhouette_score": (sil * 100.0).round() / 100.0,
        "clustering_quality": if sil > 0.5 { "good" } else if sil > 0.25 { "fair" } else { "poor" },
        "clusters": cluster_info,
    })))
}

// -- vision_cluster_outliers ----------------------------------------------

pub fn definition_vision_cluster_outliers() -> ToolDefinition {
    ToolDefinition {
        name: "vision_cluster_outliers".to_string(),
        description: Some(
            "Detect captures that don't fit any cluster using distance-based anomaly detection"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "number", "description": "Session to check (default: all)" },
                "outlier_threshold": { "type": "number", "description": "Z-score threshold for outlier detection", "default": 1.5 },
                "max_captures": { "type": "number", "description": "Max captures to analyze", "default": 100 }
            }
        }),
    }
}

pub async fn execute_vision_cluster_outliers(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        session_id: Option<u32>,
        #[serde(default = "default_outlier_threshold")]
        outlier_threshold: f64,
        #[serde(default = "default_max_outlier")]
        max_captures: usize,
    }
    fn default_outlier_threshold() -> f64 {
        1.5
    }
    fn default_max_outlier() -> usize {
        100
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session_guard = session.lock().await;
    let store = session_guard.store();

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
            "error": "Need at least 3 captures for outlier detection",
            "capture_count": n,
        })));
    }

    let dna_list: Vec<VisualDNA> = obs_list
        .iter()
        .map(|obs| {
            build_visual_dna(
                obs.id,
                &obs.embedding,
                obs.metadata.width,
                obs.metadata.height,
                &obs.metadata.labels,
                &obs.metadata.description,
            )
        })
        .collect();
    let features: Vec<Vec<f64>> = dna_list.iter().map(|d| dna_to_features(d)).collect();

    // Compute average distance of each point to all others
    let mut avg_distances: Vec<f64> = Vec::with_capacity(n);
    for i in 0..n {
        let total: f64 = (0..n)
            .filter(|&j| j != i)
            .map(|j| euclidean_distance(&features[i], &features[j]))
            .sum();
        avg_distances.push(total / (n - 1) as f64);
    }

    // Compute mean and std of average distances
    let mean_dist: f64 = avg_distances.iter().sum::<f64>() / n as f64;
    let variance: f64 = avg_distances
        .iter()
        .map(|&d| (d - mean_dist).powi(2))
        .sum::<f64>()
        / n as f64;
    let std_dist = variance.sqrt();

    // Identify outliers using z-score
    let threshold = p.outlier_threshold.max(0.5);
    let mut outliers: Vec<Value> = Vec::new();
    let mut normal: Vec<Value> = Vec::new();

    for i in 0..n {
        let z_score = if std_dist > 1e-10 {
            (avg_distances[i] - mean_dist) / std_dist
        } else {
            0.0
        };
        let is_outlier = z_score > threshold;

        let entry = json!({
            "capture_id": dna_list[i].capture_id,
            "avg_distance": (avg_distances[i] * 100.0).round() / 100.0,
            "z_score": (z_score * 100.0).round() / 100.0,
            "is_outlier": is_outlier,
            "layout": &dna_list[i].layout_pattern,
            "complexity": dna_list[i].complexity_score,
        });

        if is_outlier {
            outliers.push(entry);
        } else {
            normal.push(entry);
        }
    }

    // Sort outliers by z_score descending
    outliers.sort_by(|a, b| {
        b["z_score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["z_score"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(ToolCallResult::json(&json!({
        "total_captures": n,
        "outlier_count": outliers.len(),
        "normal_count": normal.len(),
        "threshold_z_score": threshold,
        "distribution": {
            "mean_distance": (mean_dist * 100.0).round() / 100.0,
            "std_distance": (std_dist * 100.0).round() / 100.0,
        },
        "outliers": outliers,
        "normal_sample": normal.into_iter().take(5).collect::<Vec<_>>(),
    })))
}

// -- vision_cluster_timeline ----------------------------------------------

pub fn definition_vision_cluster_timeline() -> ToolDefinition {
    ToolDefinition {
        name: "vision_cluster_timeline".to_string(),
        description: Some("Show how visual clusters evolve over time within a session".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "number", "description": "Session to analyze (default: all)" },
                "k": { "type": "number", "description": "Number of clusters (default: auto)", "default": 0 },
                "max_captures": { "type": "number", "description": "Max captures", "default": 100 }
            }
        }),
    }
}

pub async fn execute_vision_cluster_timeline(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        session_id: Option<u32>,
        #[serde(default)]
        k: usize,
        #[serde(default = "default_max_timeline")]
        max_captures: usize,
    }
    fn default_max_timeline() -> usize {
        100
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session_guard = session.lock().await;
    let store = session_guard.store();

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
            "error": "Need at least 2 captures for timeline analysis",
            "capture_count": n,
        })));
    }

    let dna_list: Vec<VisualDNA> = obs_list
        .iter()
        .map(|obs| {
            build_visual_dna(
                obs.id,
                &obs.embedding,
                obs.metadata.width,
                obs.metadata.height,
                &obs.metadata.labels,
                &obs.metadata.description,
            )
        })
        .collect();
    let features: Vec<Vec<f64>> = dna_list.iter().map(|d| dna_to_features(d)).collect();

    // Determine k
    let k = if p.k > 0 {
        p.k.min(n)
    } else {
        let max_k = ((n as f64).sqrt().ceil() as usize).max(2).min(6);
        let mut best_k = 2;
        let mut best_sil = -1.0f64;
        for candidate_k in 2..=max_k {
            if candidate_k >= n {
                break;
            }
            let (assignments, _) = kmeans_cluster(&features, candidate_k, 50);
            let sil = silhouette_score(&features, &assignments, candidate_k);
            if sil > best_sil {
                best_sil = sil;
                best_k = candidate_k;
            }
        }
        best_k
    };

    let (assignments, _centroids) = kmeans_cluster(&features, k, 100);

    // Build timeline entries
    let mut timeline: Vec<Value> = Vec::new();
    let mut cluster_transitions = 0;
    let mut prev_cluster: Option<usize> = None;

    for (i, obs) in obs_list.iter().enumerate() {
        let cluster = assignments[i];
        let is_transition = prev_cluster.map(|pc| pc != cluster).unwrap_or(false);
        if is_transition {
            cluster_transitions += 1;
        }

        timeline.push(json!({
            "capture_id": obs.id,
            "timestamp": obs.timestamp,
            "cluster_id": cluster,
            "is_transition": is_transition,
            "previous_cluster": prev_cluster,
            "description": obs.metadata.description,
        }));

        prev_cluster = Some(cluster);
    }

    // Compute cluster durations
    let mut cluster_durations: HashMap<usize, u64> = HashMap::new();
    for i in 0..n.saturating_sub(1) {
        let c = assignments[i];
        let duration = obs_list[i + 1]
            .timestamp
            .saturating_sub(obs_list[i].timestamp);
        *cluster_durations.entry(c).or_insert(0) += duration;
    }

    let mut duration_info: Vec<Value> = cluster_durations
        .iter()
        .map(|(&c, &dur)| {
            json!({
                "cluster_id": c,
                "total_duration_seconds": dur,
                "member_count": assignments.iter().filter(|&&a| a == c).count(),
            })
        })
        .collect();
    duration_info
        .sort_by_key(|v| std::cmp::Reverse(v["total_duration_seconds"].as_u64().unwrap_or(0)));

    // Transition rate
    let transition_rate = if n > 1 {
        cluster_transitions as f64 / (n - 1) as f64
    } else {
        0.0
    };

    let stability = if transition_rate < 0.1 {
        "very_stable"
    } else if transition_rate < 0.25 {
        "stable"
    } else if transition_rate < 0.5 {
        "moderate"
    } else {
        "volatile"
    };

    Ok(ToolCallResult::json(&json!({
        "total_captures": n,
        "k": k,
        "cluster_transitions": cluster_transitions,
        "transition_rate": (transition_rate * 100.0).round() / 100.0,
        "stability": stability,
        "cluster_durations": duration_info,
        "timeline": timeline,
    })))
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_hash_deterministic() {
        let features = vec![1.0, 2.0, 3.0, 4.5];
        let h1 = feature_hash(&features);
        let h2 = feature_hash(&features);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_feature_hash_different() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 4.0];
        assert_ne!(feature_hash(&a), feature_hash(&b));
    }

    #[test]
    fn test_color_histogram_empty_embedding() {
        let hist = compute_color_histogram(&[], 100, 100);
        assert_eq!(hist.hue_bins, [0.0; 8]);
        assert_eq!(hist.entropy, 0.0);
    }

    #[test]
    fn test_color_histogram_uniform() {
        let embedding: Vec<f32> = (0..64).map(|i| (i as f32) * 0.1).collect();
        let hist = compute_color_histogram(&embedding, 800, 600);
        assert!(hist.entropy > 0.0);
        assert!(hist.avg_saturation >= 0.0 && hist.avg_saturation <= 1.0);
        assert!(hist.avg_brightness >= 0.0 && hist.avg_brightness <= 1.0);
    }

    #[test]
    fn test_color_histogram_distance_identical() {
        let embedding: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let h1 = compute_color_histogram(&embedding, 100, 100);
        let h2 = compute_color_histogram(&embedding, 100, 100);
        let dist = color_histogram_distance(&h1, &h2);
        assert!(dist < 0.001, "Identical histograms should have ~0 distance");
    }

    #[test]
    fn test_edge_density_empty() {
        let edge = compute_edge_density(&[], 100, 100);
        assert_eq!(edge.overall_density, 0.0);
        assert_eq!(edge.block_count, 1);
    }

    #[test]
    fn test_edge_density_has_blocks() {
        let embedding: Vec<f32> = (0..128)
            .map(|i| if i % 4 == 0 { 1.0 } else { -1.0 })
            .collect();
        let edge = compute_edge_density(&embedding, 800, 600);
        assert!(edge.block_count >= 1);
        assert!(edge.overall_density >= 0.0 && edge.overall_density <= 1.0);
    }

    #[test]
    fn test_detect_layout_pattern_basic() {
        let pattern = detect_layout_pattern(1920, 1080, &[], 0.5);
        assert!(matches!(
            pattern,
            LayoutPattern::TwoColumn | LayoutPattern::ThreeColumn | LayoutPattern::FreeForm
        ));
    }

    #[test]
    fn test_detect_layout_dashboard() {
        let labels = vec!["dashboard".to_string()];
        let pattern = detect_layout_pattern(1920, 1080, &labels, 0.5);
        assert!(matches!(pattern, LayoutPattern::Dashboard));
    }

    #[test]
    fn test_compute_text_density() {
        let labels = vec!["text".to_string(), "paragraph".to_string()];
        let desc = Some("This is a long article with lots of text content here".to_string());
        let density = compute_text_density(&labels, &desc);
        assert!(density > 0.0);
        assert!(density <= 1.0);
    }

    #[test]
    fn test_compute_complexity() {
        let low = compute_complexity(0.1, 0.5, 1, 2);
        let high = compute_complexity(0.9, 2.0, 10, 30);
        assert!(high > low, "Higher inputs should give higher complexity");
    }

    #[test]
    fn test_build_visual_dna() {
        let embedding: Vec<f32> = (0..64).map(|i| (i as f32) * 0.05).collect();
        let labels = vec!["ui".to_string(), "test".to_string()];
        let desc = Some("A test capture".to_string());
        let dna = build_visual_dna(1, &embedding, 800, 600, &labels, &desc);
        assert_eq!(dna.capture_id, 1);
        assert!(!dna.layout_pattern.is_empty());
        assert!(dna.complexity_score >= 0.0 && dna.complexity_score <= 1.0);
        assert!(dna.fingerprint_hash != 0);
    }

    #[test]
    fn test_dna_distance_identical() {
        let embedding: Vec<f32> = (0..64).map(|i| (i as f32) * 0.05).collect();
        let labels = vec!["ui".to_string()];
        let desc = Some("test".to_string());
        let d1 = build_visual_dna(1, &embedding, 800, 600, &labels, &desc);
        let d2 = build_visual_dna(2, &embedding, 800, 600, &labels, &desc);
        let dist = dna_distance(&d1, &d2);
        assert!(
            dist < 0.01,
            "Identical DNA should have near-zero distance: {}",
            dist
        );
    }

    #[test]
    fn test_dna_distance_different() {
        let emb_a: Vec<f32> = (0..64).map(|i| (i as f32) * 0.1).collect();
        let emb_b: Vec<f32> = (0..64).map(|i| ((63 - i) as f32) * 0.1).collect();
        let la = vec!["ui".to_string()];
        let lb = vec!["dashboard".to_string()];
        let d1 = build_visual_dna(1, &emb_a, 800, 600, &la, &None);
        let d2 = build_visual_dna(2, &emb_b, 400, 800, &lb, &None);
        let dist = dna_distance(&d1, &d2);
        assert!(
            dist > 0.05,
            "Different DNA should have positive distance: {}",
            dist
        );
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        let d = euclidean_distance(&a, &b);
        assert!((d - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_kmeans_basic() {
        let features = vec![
            vec![0.0, 0.0],
            vec![0.1, 0.1],
            vec![10.0, 10.0],
            vec![10.1, 10.1],
        ];
        let (assignments, centroids) = kmeans_cluster(&features, 2, 50);
        assert_eq!(assignments.len(), 4);
        assert_eq!(centroids.len(), 2);
        // Points 0,1 should be in one cluster, 2,3 in another
        assert_eq!(assignments[0], assignments[1]);
        assert_eq!(assignments[2], assignments[3]);
        assert_ne!(assignments[0], assignments[2]);
    }

    #[test]
    fn test_kmeans_single_cluster() {
        let features = vec![vec![1.0, 1.0], vec![1.1, 1.1], vec![0.9, 0.9]];
        let (assignments, _) = kmeans_cluster(&features, 1, 50);
        assert!(assignments.iter().all(|&a| a == 0));
    }

    #[test]
    fn test_silhouette_score_good_clustering() {
        let features = vec![
            vec![0.0, 0.0],
            vec![0.1, 0.0],
            vec![10.0, 10.0],
            vec![10.1, 10.0],
        ];
        let assignments = vec![0, 0, 1, 1];
        let sil = silhouette_score(&features, &assignments, 2);
        assert!(
            sil > 0.5,
            "Well-separated clusters should have high silhouette: {}",
            sil
        );
    }

    #[test]
    fn test_balance_score_equal() {
        let weights = [0.25, 0.25, 0.25, 0.25];
        let score = compute_balance_score(weights);
        assert!(
            (score - 1.0).abs() < 0.001,
            "Equal weights should give balance ~1.0"
        );
    }

    #[test]
    fn test_balance_score_imbalanced() {
        let weights = [1.0, 0.0, 0.0, 0.0];
        let score = compute_balance_score(weights);
        assert!(
            score < 0.5,
            "Imbalanced weights should give low score: {}",
            score
        );
    }

    #[test]
    fn test_grid_alignment_simple() {
        let embedding: Vec<f32> = (0..64).map(|i| (i as f32) * 0.1).collect();
        let edge = compute_edge_density(&embedding, 800, 600);
        let grid = detect_grid_alignment(&embedding, &edge);
        assert!(grid.columns >= 1);
        assert!(grid.rows >= 1);
        assert!(grid.alignment_score >= 0.0);
    }

    #[test]
    fn test_estimate_whitespace() {
        let edge_low = EdgeDensityProfile {
            overall_density: 0.1,
            quadrant_density: [0.1, 0.1, 0.1, 0.1],
            h_v_ratio: 1.0,
            block_count: 2,
        };
        let whitespace = estimate_whitespace(&edge_low, 800, 600);
        assert!(
            whitespace > 0.5,
            "Low edge density should indicate high whitespace"
        );
    }

    #[test]
    fn test_dna_to_features_length() {
        let embedding: Vec<f32> = (0..64).map(|i| (i as f32) * 0.05).collect();
        let dna = build_visual_dna(1, &embedding, 800, 600, &[], &None);
        let features = dna_to_features(&dna);
        assert!(
            features.len() > 15,
            "Feature vector should have enough dimensions"
        );
    }

    #[test]
    fn test_edge_profile_distance_identical() {
        let edge = EdgeDensityProfile {
            overall_density: 0.5,
            quadrant_density: [0.4, 0.5, 0.6, 0.5],
            h_v_ratio: 1.2,
            block_count: 5,
        };
        let dist = edge_profile_distance(&edge, &edge);
        assert!(dist < 0.001, "Identical profiles should have ~0 distance");
    }

    #[test]
    fn test_word_overlap_identical() {
        let score = word_overlap("hello world", "hello world");
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_word_overlap_no_match() {
        let score = word_overlap("hello world", "foo bar");
        assert!(score < 0.001);
    }

    #[test]
    fn test_composition_quality_label() {
        assert_eq!(CompositionQuality::Excellent.label(), "excellent");
        assert_eq!(CompositionQuality::Poor.label(), "poor");
    }

    #[test]
    fn test_layout_pattern_label() {
        assert_eq!(LayoutPattern::Dashboard.label(), "dashboard");
        assert_eq!(LayoutPattern::SingleColumn.label(), "single_column");
    }
}
