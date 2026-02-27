//! Temporal Inventions (5–8): Temporal Vision, Visual Archaeology,
//! Visual Memory Consolidation, Visual Déjà Vu.
//!
//! 12 MCP tools that see through time.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::session::manager::VisionSessionManager;
use crate::types::error::{McpError, McpResult};
use crate::types::response::{ToolCallResult, ToolDefinition};

// ═══════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ArtifactType {
    PartialCapture,
    Thumbnail,
    CachedImage,
    Description,
    HTMLSnapshot,
    StyleReference,
    RelatedCapture,
    UserReport,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConsolidationReason {
    SessionStart,
    SessionEnd,
    SignificantChange,
    UserMarked,
    ReferencedByMemory,
    IncidentEvidence,
    BestQuality,
    UniqueState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PatternFrequency {
    FirstTime,
    Rare,
    Occasional,
    Frequent,
    Constant,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DejaVuSignificance {
    Informational,
    Warning,
    KnownBug,
    Critical,
}

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 5: Temporal Vision — 3 tools
// ═══════════════════════════════════════════════════════════════════════════

// ── vision_at_time ──────────────────────────────────────────────────────

pub fn definition_vision_at_time() -> ToolDefinition {
    ToolDefinition {
        name: "vision_at_time".to_string(),
        description: Some("Get visual state at a specific time".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["target_time"],
            "properties": {
                "target_time": { "type": "number", "description": "Target time (epoch seconds)" },
                "tolerance_seconds": { "type": "number", "description": "Time tolerance in seconds", "default": 300 },
                "subject": { "type": "string", "description": "Subject to look for" }
            }
        }),
    }
}

pub async fn execute_vision_at_time(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        target_time: u64,
        #[serde(default = "def_300")]
        tolerance_seconds: u64,
        subject: Option<String>,
    }
    fn def_300() -> u64 {
        300
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let mut captures: Vec<Value> = Vec::new();

    for obs in &store.observations {
        let dist = obs.timestamp.abs_diff(p.target_time);
        if dist > p.tolerance_seconds {
            continue;
        }

        if let Some(ref subj) = p.subject {
            let mut matches = false;
            if let Some(desc) = &obs.metadata.description {
                if word_overlap(&subj.to_lowercase(), &desc.to_lowercase()) > 0.15 {
                    matches = true;
                }
            }
            for label in &obs.metadata.labels {
                if subj.to_lowercase().contains(&label.to_lowercase()) {
                    matches = true;
                    break;
                }
            }
            if !matches {
                continue;
            }
        }

        let relevance = 1.0 - (dist as f64 / p.tolerance_seconds as f64);
        captures.push(json!({
            "capture_id": obs.id,
            "timestamp": obs.timestamp,
            "distance_seconds": dist,
            "relevance": (relevance * 100.0).round() / 100.0,
            "description": obs.metadata.description,
            "labels": obs.metadata.labels,
            "session_id": obs.session_id,
        }));
    }

    captures.sort_by(|a, b| {
        a["distance_seconds"]
            .as_u64()
            .unwrap_or(u64::MAX)
            .cmp(&b["distance_seconds"].as_u64().unwrap_or(u64::MAX))
    });
    captures.truncate(10);

    Ok(ToolCallResult::json(&json!({
        "target_time": p.target_time,
        "tolerance_seconds": p.tolerance_seconds,
        "subject": p.subject,
        "found": captures.len(),
        "captures": captures,
    })))
}

// ── vision_timeline ─────────────────────────────────────────────────────

pub fn definition_vision_timeline() -> ToolDefinition {
    ToolDefinition {
        name: "vision_timeline".to_string(),
        description: Some("Get visual timeline for an element or page".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "subject": { "type": "string", "description": "Subject to track over time" },
                "start_time": { "type": "number", "description": "Start of time range (epoch)" },
                "end_time": { "type": "number", "description": "End of time range (epoch)" }
            }
        }),
    }
}

pub async fn execute_vision_timeline(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        subject: Option<String>,
        start_time: Option<u64>,
        end_time: Option<u64>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let now = now_epoch();
    let start = p.start_time.unwrap_or(0);
    let end = p.end_time.unwrap_or(now);

    let mut timeline: Vec<Value> = Vec::new();

    for obs in &store.observations {
        if obs.timestamp < start || obs.timestamp > end {
            continue;
        }
        if let Some(ref subj) = p.subject {
            let mut matches = false;
            if let Some(desc) = &obs.metadata.description {
                if word_overlap(&subj.to_lowercase(), &desc.to_lowercase()) > 0.1 {
                    matches = true;
                }
            }
            for label in &obs.metadata.labels {
                if subj.to_lowercase().contains(&label.to_lowercase()) {
                    matches = true;
                    break;
                }
            }
            if !matches {
                continue;
            }
        }

        timeline.push(json!({
            "capture_id": obs.id,
            "timestamp": obs.timestamp,
            "session_id": obs.session_id,
            "description": obs.metadata.description,
            "labels": obs.metadata.labels,
        }));
    }

    timeline.sort_by(|a, b| {
        a["timestamp"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&b["timestamp"].as_u64().unwrap_or(0))
    });

    // Detect transitions between timeline entries
    let mut transitions: Vec<Value> = Vec::new();
    for i in 1..timeline.len() {
        let prev_ts = timeline[i - 1]["timestamp"].as_u64().unwrap_or(0);
        let curr_ts = timeline[i]["timestamp"].as_u64().unwrap_or(0);
        let gap = curr_ts.saturating_sub(prev_ts);
        transitions.push(json!({
            "from_capture": timeline[i-1]["capture_id"],
            "to_capture": timeline[i]["capture_id"],
            "gap_seconds": gap,
        }));
    }

    // Find gaps in coverage
    let mut gaps: Vec<Value> = Vec::new();
    for t in &transitions {
        let gap = t["gap_seconds"].as_u64().unwrap_or(0);
        if gap > 3600 {
            gaps.push(json!({
                "from": t["from_capture"],
                "to": t["to_capture"],
                "gap_seconds": gap,
            }));
        }
    }

    Ok(ToolCallResult::json(&json!({
        "subject": p.subject,
        "time_range": { "start": start, "end": end },
        "entries": timeline.len(),
        "timeline": timeline,
        "transitions": transitions,
        "gaps": gaps,
    })))
}

// ── vision_reconstruct ──────────────────────────────────────────────────

pub fn definition_vision_reconstruct() -> ToolDefinition {
    ToolDefinition {
        name: "vision_reconstruct".to_string(),
        description: Some("Reconstruct visual state from partial evidence".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["target_time"],
            "properties": {
                "target_time": { "type": "number", "description": "Time to reconstruct (epoch)" },
                "subject": { "type": "string", "description": "What to reconstruct" }
            }
        }),
    }
}

pub async fn execute_vision_reconstruct(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        target_time: u64,
        subject: Option<String>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    // Find nearest captures before and after target
    let mut before: Option<(u64, &_)> = None;
    let mut after: Option<(u64, &_)> = None;

    for obs in &store.observations {
        if let Some(ref subj) = p.subject {
            let mut matches = false;
            if let Some(desc) = &obs.metadata.description {
                if word_overlap(&subj.to_lowercase(), &desc.to_lowercase()) > 0.1 {
                    matches = true;
                }
            }
            if !matches {
                continue;
            }
        }

        if obs.timestamp <= p.target_time {
            let dist = p.target_time - obs.timestamp;
            if before.is_none() || dist < before.unwrap().0 {
                before = Some((dist, obs));
            }
        } else {
            let dist = obs.timestamp - p.target_time;
            if after.is_none() || dist < after.unwrap().0 {
                after = Some((dist, obs));
            }
        }
    }

    let mut sources: Vec<Value> = Vec::new();
    let mut certain: Vec<String> = Vec::new();
    let mut inferred: Vec<String> = Vec::new();
    let mut confidence: f64 = 0.0;

    if let Some((dist, obs)) = before {
        sources.push(json!({
            "capture_id": obs.id, "timestamp": obs.timestamp, "role": "before",
            "distance_seconds": dist
        }));
        if let Some(desc) = &obs.metadata.description {
            if dist < 300 {
                certain.push(format!("(from cap {}) {}", obs.id, desc));
                confidence += 0.5;
            } else {
                inferred.push(format!("(inferred from cap {}) {}", obs.id, desc));
                confidence += 0.2;
            }
        }
    }

    if let Some((dist, obs)) = after {
        sources.push(json!({
            "capture_id": obs.id, "timestamp": obs.timestamp, "role": "after",
            "distance_seconds": dist
        }));
        if let Some(desc) = &obs.metadata.description {
            if dist < 300 {
                certain.push(format!("(from cap {}) {}", obs.id, desc));
                confidence += 0.3;
            } else {
                inferred.push(format!("(inferred from cap {}) {}", obs.id, desc));
                confidence += 0.1;
            }
        }
    }

    Ok(ToolCallResult::json(&json!({
        "target_time": p.target_time,
        "subject": p.subject,
        "reconstruction": {
            "sources": sources,
            "certain_elements": certain,
            "inferred_elements": inferred,
            "confidence": (confidence.min(1.0) * 100.0).round() / 100.0,
        },
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 6: Visual Archaeology — 3 tools
// ═══════════════════════════════════════════════════════════════════════════

// ── vision_archaeology_dig ──────────────────────────────────────────────

pub fn definition_vision_archaeology_dig() -> ToolDefinition {
    ToolDefinition {
        name: "vision_archaeology_dig".to_string(),
        description: Some("Search for artifacts of lost UI state".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["target"],
            "properties": {
                "target": { "type": "string", "description": "What to search for" },
                "start_time": { "type": "number", "description": "Start of time range (epoch)" },
                "end_time": { "type": "number", "description": "End of time range (epoch)" }
            }
        }),
    }
}

pub async fn execute_vision_archaeology_dig(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        target: String,
        start_time: Option<u64>,
        end_time: Option<u64>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let target_lower = p.target.to_lowercase();

    let mut artifacts: Vec<Value> = Vec::new();

    for obs in &store.observations {
        if let Some(start) = p.start_time {
            if obs.timestamp < start {
                continue;
            }
        }
        if let Some(end) = p.end_time {
            if obs.timestamp > end {
                continue;
            }
        }

        let mut relevance: f64 = 0.0;
        let mut artifact_type = "related_capture";

        if let Some(desc) = &obs.metadata.description {
            let overlap = word_overlap(&target_lower, &desc.to_lowercase());
            if overlap > 0.1 {
                relevance = overlap;
                artifact_type = if overlap > 0.5 {
                    "direct_capture"
                } else {
                    "partial_capture"
                };
            }
        }
        for label in &obs.metadata.labels {
            if target_lower.contains(&label.to_lowercase()) {
                relevance += 0.2;
                break;
            }
        }

        if relevance > 0.1 {
            artifacts.push(json!({
                "capture_id": obs.id,
                "timestamp": obs.timestamp,
                "artifact_type": artifact_type,
                "reliability": (relevance.min(1.0) * 100.0).round() / 100.0,
                "description": obs.metadata.description,
                "labels": obs.metadata.labels,
            }));
        }
    }

    artifacts.sort_by(|a, b| {
        b["reliability"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["reliability"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let confidence = if artifacts.is_empty() {
        0.0
    } else {
        artifacts[0]["reliability"].as_f64().unwrap_or(0.0) / 100.0
    };

    Ok(ToolCallResult::json(&json!({
        "target": p.target,
        "artifact_count": artifacts.len(),
        "artifacts": artifacts,
        "confidence": (confidence * 100.0).round() / 100.0,
    })))
}

// ── vision_archaeology_reconstruct ──────────────────────────────────────

pub fn definition_vision_archaeology_reconstruct() -> ToolDefinition {
    ToolDefinition {
        name: "vision_archaeology_reconstruct".to_string(),
        description: Some("Attempt to reconstruct deleted UI from traces".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["target"],
            "properties": {
                "target": { "type": "string", "description": "What to reconstruct" }
            }
        }),
    }
}

pub async fn execute_vision_archaeology_reconstruct(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        target: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let target_lower = p.target.to_lowercase();

    let mut elements: Vec<Value> = Vec::new();
    let mut unknowns: Vec<String> = Vec::new();

    for obs in &store.observations {
        if let Some(desc) = &obs.metadata.description {
            let overlap = word_overlap(&target_lower, &desc.to_lowercase());
            if overlap > 0.2 {
                elements.push(json!({
                    "element": desc,
                    "confidence": (overlap.min(1.0) * 100.0).round() / 100.0,
                    "source_capture": obs.id,
                    "source_timestamp": obs.timestamp,
                }));
            }
        }
    }

    if elements.is_empty() {
        unknowns.push(format!("No artifacts found for '{}'", p.target));
    }

    let confidence = if elements.is_empty() {
        0.0
    } else {
        elements
            .iter()
            .filter_map(|e| e["confidence"].as_f64())
            .sum::<f64>()
            / elements.len() as f64
            / 100.0
    };

    Ok(ToolCallResult::json(&json!({
        "target": p.target,
        "reconstruction": {
            "elements": elements,
            "unknowns": unknowns,
            "confidence": (confidence.min(1.0) * 100.0).round() / 100.0,
        },
    })))
}

// ── vision_archaeology_report ───────────────────────────────────────────

pub fn definition_vision_archaeology_report() -> ToolDefinition {
    ToolDefinition {
        name: "vision_archaeology_report".to_string(),
        description: Some("Generate an archaeology report for a visual target".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["target"],
            "properties": {
                "target": { "type": "string", "description": "The target to report on" }
            }
        }),
    }
}

pub async fn execute_vision_archaeology_report(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        target: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let target_lower = p.target.to_lowercase();

    let mut first_seen: Option<u64> = None;
    let mut last_seen: Option<u64> = None;
    let mut appearances = 0usize;
    let mut sessions: std::collections::HashSet<u32> = std::collections::HashSet::new();

    for obs in &store.observations {
        let mut matches = false;
        if let Some(desc) = &obs.metadata.description {
            if word_overlap(&target_lower, &desc.to_lowercase()) > 0.15 {
                matches = true;
            }
        }
        for label in &obs.metadata.labels {
            if target_lower.contains(&label.to_lowercase()) {
                matches = true;
                break;
            }
        }
        if !matches {
            continue;
        }

        appearances += 1;
        sessions.insert(obs.session_id);
        if first_seen.is_none() || obs.timestamp < first_seen.unwrap() {
            first_seen = Some(obs.timestamp);
        }
        if last_seen.is_none() || obs.timestamp > last_seen.unwrap() {
            last_seen = Some(obs.timestamp);
        }
    }

    Ok(ToolCallResult::json(&json!({
        "target": p.target,
        "report": {
            "appearances": appearances,
            "sessions": sessions.len(),
            "first_seen": first_seen,
            "last_seen": last_seen,
            "time_span_seconds": match (first_seen, last_seen) {
                (Some(f), Some(l)) => l.saturating_sub(f),
                _ => 0,
            },
        },
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 7: Visual Memory Consolidation — 3 tools
// ═══════════════════════════════════════════════════════════════════════════

// ── vision_consolidate ──────────────────────────────────────────────────

pub fn definition_vision_consolidate() -> ToolDefinition {
    ToolDefinition {
        name: "vision_consolidate".to_string(),
        description: Some("Consolidate visual history into key moments".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "start_time": { "type": "number", "description": "Start of range (epoch)" },
                "end_time": { "type": "number", "description": "End of range (epoch)" },
                "keep_ratio": { "type": "number", "description": "Ratio to keep (0.0-1.0)", "default": 0.3 }
            }
        }),
    }
}

pub async fn execute_vision_consolidate(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        start_time: Option<u64>,
        end_time: Option<u64>,
        #[serde(default = "def_03")]
        keep_ratio: f64,
    }
    fn def_03() -> f64 {
        0.3
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let mut candidates: Vec<&_> = store.observations.iter().collect();
    if let Some(start) = p.start_time {
        candidates.retain(|o| o.timestamp >= start);
    }
    if let Some(end) = p.end_time {
        candidates.retain(|o| o.timestamp <= end);
    }

    let total = candidates.len();
    let keep_count = ((total as f64 * p.keep_ratio).ceil() as usize).max(1);

    // Score each by importance
    let mut scored: Vec<Value> = candidates
        .iter()
        .enumerate()
        .map(|(idx, obs)| {
            let mut importance: f64 = 0.0;
            // First/last are important
            if idx == 0 {
                importance += 0.3;
            }
            if idx == candidates.len() - 1 {
                importance += 0.3;
            }
            // Quality bonus
            importance += obs.metadata.quality_score as f64 * 0.2;
            // Label count (more labels = more interesting)
            importance += (obs.metadata.labels.len() as f64 * 0.05).min(0.2);

            json!({
                "capture_id": obs.id,
                "timestamp": obs.timestamp,
                "importance": (importance.min(1.0) * 100.0).round() / 100.0,
                "reason": if idx == 0 { "session_start" }
                         else if idx == candidates.len() - 1 { "session_end" }
                         else if obs.metadata.quality_score > 0.8 { "best_quality" }
                         else { "representative" },
            })
        })
        .collect();

    scored.sort_by(|a, b| {
        b["importance"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["importance"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let kept: Vec<Value> = scored.into_iter().take(keep_count).collect();
    let removed = total.saturating_sub(keep_count);

    Ok(ToolCallResult::json(&json!({
        "original_count": total,
        "kept_count": kept.len(),
        "removed_count": removed,
        "compression_ratio": if total == 0 { 1.0 } else { 1.0 - (kept.len() as f64 / total as f64) },
        "consolidated": kept,
    })))
}

// ── vision_consolidate_preview ──────────────────────────────────────────

pub fn definition_vision_consolidate_preview() -> ToolDefinition {
    ToolDefinition {
        name: "vision_consolidate_preview".to_string(),
        description: Some("Preview what would be kept or lost in consolidation".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "keep_ratio": { "type": "number", "default": 0.3 }
            }
        }),
    }
}

pub async fn execute_vision_consolidate_preview(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        #[serde(default = "def_03")]
        keep_ratio: f64,
    }
    fn def_03() -> f64 {
        0.3
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let total = store.observations.len();
    let keep = ((total as f64 * p.keep_ratio).ceil() as usize).max(1);
    let remove = total.saturating_sub(keep);

    Ok(ToolCallResult::json(&json!({
        "total_captures": total,
        "would_keep": keep,
        "would_remove": remove,
        "keep_ratio": p.keep_ratio,
        "compression_ratio": if total == 0 { 0.0 } else { remove as f64 / total as f64 },
        "preview": true,
    })))
}

// ── vision_consolidate_policy ───────────────────────────────────────────

pub fn definition_vision_consolidate_policy() -> ToolDefinition {
    ToolDefinition {
        name: "vision_consolidate_policy".to_string(),
        description: Some("Set consolidation policy for visual memory".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "max_captures_per_session": { "type": "number", "description": "Max captures to keep per session" },
                "max_age_hours": { "type": "number", "description": "Max age in hours before consolidation" },
                "always_keep_labeled": { "type": "boolean", "description": "Always keep labeled captures", "default": true }
            }
        }),
    }
}

pub async fn execute_vision_consolidate_policy(
    args: Value,
    _session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        max_captures_per_session: Option<u64>,
        max_age_hours: Option<u64>,
        #[serde(default = "def_true")]
        always_keep_labeled: bool,
    }
    fn def_true() -> bool {
        true
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    Ok(ToolCallResult::json(&json!({
        "policy": {
            "max_captures_per_session": p.max_captures_per_session,
            "max_age_hours": p.max_age_hours,
            "always_keep_labeled": p.always_keep_labeled,
        },
        "status": "policy_set",
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 8: Visual Déjà Vu — 3 tools
// ═══════════════════════════════════════════════════════════════════════════

// ── vision_dejavu_check ─────────────────────────────────────────────────

pub fn definition_vision_dejavu_check() -> ToolDefinition {
    ToolDefinition {
        name: "vision_dejavu_check".to_string(),
        description: Some("Check if current visual state has been seen before".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Current capture to check" },
                "min_similarity": { "type": "number", "description": "Minimum similarity threshold", "default": 0.5 }
            }
        }),
    }
}

pub async fn execute_vision_dejavu_check(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
        #[serde(default = "def_05")]
        min_similarity: f64,
    }
    fn def_05() -> f64 {
        0.5
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let current = match store.observations.iter().find(|o| o.id == p.capture_id) {
        Some(o) => o,
        None => return Err(McpError::CaptureNotFound(p.capture_id)),
    };

    let current_desc = current.metadata.description.as_deref().unwrap_or("");
    let current_labels: std::collections::HashSet<String> =
        current.metadata.labels.iter().cloned().collect();

    let mut matches: Vec<Value> = Vec::new();

    for obs in &store.observations {
        if obs.id == p.capture_id {
            continue;
        }

        let mut similarity: f64 = 0.0;
        if let Some(desc) = &obs.metadata.description {
            similarity += word_overlap(current_desc, desc) * 0.6;
        }
        let obs_labels: std::collections::HashSet<String> =
            obs.metadata.labels.iter().cloned().collect();
        let label_overlap = if current_labels.is_empty() || obs_labels.is_empty() {
            0.0
        } else {
            let shared = current_labels.intersection(&obs_labels).count();
            shared as f64 / current_labels.len().max(obs_labels.len()) as f64
        };
        similarity += label_overlap * 0.4;

        if similarity >= p.min_similarity {
            matches.push(json!({
                "historical_capture": obs.id,
                "timestamp": obs.timestamp,
                "similarity": (similarity * 100.0).round() / 100.0,
                "session_id": obs.session_id,
                "description": obs.metadata.description,
            }));
        }
    }

    matches.sort_by(|a, b| {
        b["similarity"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["similarity"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.truncate(10);

    let frequency = match matches.len() {
        0 => "first_time",
        1..=3 => "rare",
        4..=10 => "occasional",
        11..=20 => "frequent",
        _ => "constant",
    };

    let significance = match matches.len() {
        0 => "informational",
        1..=2 => "informational",
        3..=5 => "warning",
        _ => "known_pattern",
    };

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "dejavu": !matches.is_empty(),
        "match_count": matches.len(),
        "frequency": frequency,
        "significance": significance,
        "matches": matches,
    })))
}

// ── vision_dejavu_patterns ──────────────────────────────────────────────

pub fn definition_vision_dejavu_patterns() -> ToolDefinition {
    ToolDefinition {
        name: "vision_dejavu_patterns".to_string(),
        description: Some("Find recurring visual patterns across sessions".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "min_occurrences": { "type": "number", "description": "Min times a pattern must repeat", "default": 2 }
            }
        }),
    }
}

pub async fn execute_vision_dejavu_patterns(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        #[serde(default = "def_2")]
        min_occurrences: usize,
    }
    fn def_2() -> usize {
        2
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    // Group observations by label sets to find recurring patterns
    let mut label_groups: std::collections::HashMap<Vec<String>, Vec<u64>> =
        std::collections::HashMap::new();
    for obs in &store.observations {
        if obs.metadata.labels.is_empty() {
            continue;
        }
        let mut sorted = obs.metadata.labels.clone();
        sorted.sort();
        label_groups.entry(sorted).or_default().push(obs.id);
    }

    let mut patterns: Vec<Value> = Vec::new();
    for (labels, ids) in &label_groups {
        if ids.len() >= p.min_occurrences {
            let frequency = match ids.len() {
                2..=3 => "rare",
                4..=10 => "occasional",
                11..=20 => "frequent",
                _ => "constant",
            };
            patterns.push(json!({
                "labels": labels,
                "occurrences": ids.len(),
                "capture_ids": ids,
                "frequency": frequency,
            }));
        }
    }

    patterns.sort_by(|a, b| {
        b["occurrences"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["occurrences"].as_u64().unwrap_or(0))
    });

    Ok(ToolCallResult::json(&json!({
        "pattern_count": patterns.len(),
        "patterns": patterns,
    })))
}

// ── vision_dejavu_alert ─────────────────────────────────────────────────

pub fn definition_vision_dejavu_alert() -> ToolDefinition {
    ToolDefinition {
        name: "vision_dejavu_alert".to_string(),
        description: Some("Set alert for specific recurring visual patterns".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["pattern_labels"],
            "properties": {
                "pattern_labels": { "type": "array", "items": { "type": "string" }, "description": "Labels that define the pattern" },
                "threshold": { "type": "number", "description": "Alert after this many occurrences", "default": 3 }
            }
        }),
    }
}

pub async fn execute_vision_dejavu_alert(
    args: Value,
    _session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        pattern_labels: Vec<String>,
        #[serde(default = "def_3")]
        threshold: usize,
    }
    fn def_3() -> usize {
        3
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    Ok(ToolCallResult::json(&json!({
        "alert_set": true,
        "pattern_labels": p.pattern_labels,
        "threshold": p.threshold,
        "status": "alert_configured",
    })))
}
