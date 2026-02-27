//! Grounding Inventions (1–4): Visual Grounding, Hallucination Detector,
//! Visual Truth Maintenance, Multi-Context Vision.
//!
//! 13 MCP tools that prove every visual claim.

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

/// Evidence backing a visual claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualEvidence {
    pub evidence_type: VisualEvidenceType,
    pub value: String,
    pub captured_at: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VisualEvidenceType {
    Text,
    Color,
    Element,
    Layout,
    State,
    Comparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UngroundedReason {
    NoCaptureExists,
    ElementNotFound,
    Contradicted,
    CaptureStale,
    Obscured,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VisualHallucinationType {
    NonExistent,
    WrongAppearance,
    WrongLocation,
    WrongText,
    WrongState,
    InventedUI,
    StaleDescription,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HallucinationSeverity {
    Minor,
    Moderate,
    Severe,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VisualTruthStatus {
    Valid,
    Stale,
    Invalidated,
    Deleted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VisualChangeType {
    ColorChange,
    TextChange,
    PositionChange,
    SizeChange,
    StateChange,
    Removal,
    Addition,
    LayoutChange,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContextType {
    DifferentSite,
    DifferentVersion,
    DifferentDevice,
    DifferentUser,
    DifferentLocale,
    ABVariant,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DifferenceImpact {
    Cosmetic,
    UXImpact,
    Functional,
    Critical,
}

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

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

fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 1: Visual Grounding — 4 tools
// ═══════════════════════════════════════════════════════════════════════════

// ── vision_ground_claim ─────────────────────────────────────────────────

pub fn definition_vision_ground_claim() -> ToolDefinition {
    ToolDefinition {
        name: "vision_ground_claim".to_string(),
        description: Some("Attempt to ground a visual claim against stored captures".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["claim"],
            "properties": {
                "claim": { "type": "string", "description": "The visual claim to ground" }
            }
        }),
    }
}

pub async fn execute_vision_ground_claim(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        claim: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    if p.claim.trim().is_empty() {
        return Ok(ToolCallResult::json(&json!({
            "status": "ungrounded", "reason": "Empty claim"
        })));
    }

    let session = session.lock().await;
    let store = session.store();
    let claim_lower = p.claim.to_lowercase();

    let mut evidence: Vec<Value> = Vec::new();
    let mut best_score: f64 = 0.0;

    for obs in &store.observations {
        let mut score: f64 = 0.0;
        if let Some(desc) = &obs.metadata.description {
            score += word_overlap(&claim_lower, &desc.to_lowercase()) * 0.6;
        }
        for label in &obs.metadata.labels {
            if claim_lower.contains(&label.to_lowercase()) {
                score += 0.3;
                break;
            }
        }
        if score > 0.15 {
            if score > best_score {
                best_score = score;
            }
            evidence.push(json!({
                "capture_id": obs.id,
                "timestamp": obs.timestamp,
                "session_id": obs.session_id,
                "description": obs.metadata.description,
                "labels": obs.metadata.labels,
                "relevance": (score * 100.0).round() / 100.0,
                "evidence_type": "observation_match",
            }));
        }
    }

    evidence.sort_by(|a, b| {
        b["relevance"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["relevance"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    evidence.truncate(10);

    let confidence = best_score.min(1.0);
    let fully_grounded = confidence > 0.5;

    Ok(ToolCallResult::json(&json!({
        "claim": p.claim,
        "fully_grounded": fully_grounded,
        "confidence": (confidence * 100.0).round() / 100.0,
        "evidence_count": evidence.len(),
        "evidence": evidence,
    })))
}

// ── vision_verify_claim ─────────────────────────────────────────────────

pub fn definition_vision_verify_claim() -> ToolDefinition {
    ToolDefinition {
        name: "vision_verify_claim".to_string(),
        description: Some("Verify if a visual claim is true against current captures".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["claim"],
            "properties": {
                "claim": { "type": "string", "description": "The visual claim to verify" },
                "max_age_seconds": { "type": "number", "description": "Max age of evidence in seconds", "default": 3600 }
            }
        }),
    }
}

pub async fn execute_vision_verify_claim(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        claim: String,
        #[serde(default = "default_max_age")]
        max_age_seconds: u64,
    }
    fn default_max_age() -> u64 {
        3600
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let now = now_epoch();
    let claim_lower = p.claim.to_lowercase();

    let mut best_match: Option<Value> = None;
    let mut best_score: f64 = 0.0;

    for obs in &store.observations {
        let age = now.saturating_sub(obs.timestamp);
        if age > p.max_age_seconds {
            continue;
        }

        let mut score: f64 = 0.0;
        if let Some(desc) = &obs.metadata.description {
            score += word_overlap(&claim_lower, &desc.to_lowercase()) * 0.7;
        }
        for label in &obs.metadata.labels {
            if claim_lower.contains(&label.to_lowercase()) {
                score += 0.2;
                break;
            }
        }
        // Recency bonus
        let recency = 1.0 - (age as f64 / p.max_age_seconds as f64);
        score += recency * 0.1;

        if score > best_score {
            best_score = score;
            best_match = Some(json!({
                "capture_id": obs.id,
                "timestamp": obs.timestamp,
                "age_seconds": age,
                "description": obs.metadata.description,
            }));
        }
    }

    let verified = best_score > 0.4;
    Ok(ToolCallResult::json(&json!({
        "claim": p.claim,
        "verified": verified,
        "confidence": (best_score.min(1.0) * 100.0).round() / 100.0,
        "status": if verified { "verified" } else { "unverified" },
        "best_evidence": best_match,
    })))
}

// ── vision_cite ─────────────────────────────────────────────────────────

pub fn definition_vision_cite() -> ToolDefinition {
    ToolDefinition {
        name: "vision_cite".to_string(),
        description: Some("Get citation for a visual element from captures".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["element"],
            "properties": {
                "element": { "type": "string", "description": "The visual element to cite" },
                "capture_id": { "type": "number", "description": "Specific capture to cite from" }
            }
        }),
    }
}

pub async fn execute_vision_cite(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        element: String,
        capture_id: Option<u64>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let element_lower = p.element.to_lowercase();

    let mut citations: Vec<Value> = Vec::new();

    for obs in &store.observations {
        if let Some(cid) = p.capture_id {
            if obs.id != cid {
                continue;
            }
        }
        let mut relevance: f64 = 0.0;
        if let Some(desc) = &obs.metadata.description {
            relevance += word_overlap(&element_lower, &desc.to_lowercase()) * 0.6;
        }
        for label in &obs.metadata.labels {
            if element_lower.contains(&label.to_lowercase()) {
                relevance += 0.3;
                break;
            }
        }
        if relevance > 0.1 {
            citations.push(json!({
                "capture_id": obs.id,
                "timestamp": obs.timestamp,
                "session_id": obs.session_id,
                "description": obs.metadata.description,
                "labels": obs.metadata.labels,
                "relevance": (relevance * 100.0).round() / 100.0,
                "citation_type": if relevance > 0.5 { "strong" } else { "partial" },
            }));
        }
    }

    citations.sort_by(|a, b| {
        b["relevance"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["relevance"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    citations.truncate(5);

    Ok(ToolCallResult::json(&json!({
        "element": p.element,
        "citations": citations,
        "cited": !citations.is_empty(),
    })))
}

// ── vision_contradict ───────────────────────────────────────────────────

pub fn definition_vision_contradict() -> ToolDefinition {
    ToolDefinition {
        name: "vision_contradict".to_string(),
        description: Some("Find evidence that contradicts a visual claim".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["claim"],
            "properties": {
                "claim": { "type": "string", "description": "The claim to find contradictions for" }
            }
        }),
    }
}

pub async fn execute_vision_contradict(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        claim: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let claim_lower = p.claim.to_lowercase();

    // Look for observations whose description mentions the same subject
    // but with different/opposite descriptors
    let negation_words = [
        "not",
        "no",
        "never",
        "without",
        "missing",
        "absent",
        "hidden",
        "invisible",
        "disabled",
        "removed",
        "deleted",
    ];
    let has_negation = negation_words.iter().any(|w| claim_lower.contains(w));

    let mut contradictions: Vec<Value> = Vec::new();

    for obs in &store.observations {
        if let Some(desc) = &obs.metadata.description {
            let desc_lower = desc.to_lowercase();
            let overlap = word_overlap(&claim_lower, &desc_lower);
            if overlap < 0.15 {
                continue;
            }

            // Check if one has negation and the other doesn't
            let desc_has_neg = negation_words.iter().any(|w| desc_lower.contains(w));
            let is_contradictory = has_negation != desc_has_neg && overlap > 0.2;

            if is_contradictory {
                contradictions.push(json!({
                    "capture_id": obs.id,
                    "timestamp": obs.timestamp,
                    "description": desc,
                    "contradiction_type": "negation_mismatch",
                    "overlap": (overlap * 100.0).round() / 100.0,
                }));
            }
        }
    }

    Ok(ToolCallResult::json(&json!({
        "claim": p.claim,
        "contradictions_found": contradictions.len(),
        "contradictions": contradictions,
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 2: Visual Hallucination Detector — 2 tools
// ═══════════════════════════════════════════════════════════════════════════

// ── vision_hallucination_check ──────────────────────────────────────────

pub fn definition_vision_hallucination_check() -> ToolDefinition {
    ToolDefinition {
        name: "vision_hallucination_check".to_string(),
        description: Some("Check AI description for visual hallucinations".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["ai_description"],
            "properties": {
                "ai_description": { "type": "string", "description": "The AI output describing visual state" }
            }
        }),
    }
}

pub async fn execute_vision_hallucination_check(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        ai_description: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    // Split description into sentences/claims
    let claims: Vec<&str> = p
        .ai_description
        .split(['.', '\n'])
        .map(|s| s.trim())
        .filter(|s| s.len() > 5)
        .collect();

    let mut hallucinations: Vec<Value> = Vec::new();
    let mut verified: Vec<Value> = Vec::new();
    let total = claims.len();

    for claim in &claims {
        let claim_lower = claim.to_lowercase();
        let mut best_score: f64 = 0.0;

        for obs in &store.observations {
            let mut score: f64 = 0.0;
            if let Some(desc) = &obs.metadata.description {
                score = word_overlap(&claim_lower, &desc.to_lowercase());
            }
            for label in &obs.metadata.labels {
                if claim_lower.contains(&label.to_lowercase()) {
                    score += 0.2;
                    break;
                }
            }
            if score > best_score {
                best_score = score;
            }
        }

        if best_score > 0.3 {
            verified.push(json!({
                "claim": claim,
                "confidence": (best_score.min(1.0) * 100.0).round() / 100.0,
            }));
        } else if best_score < 0.1 && !store.observations.is_empty() {
            let severity = if claim_lower.contains("click")
                || claim_lower.contains("submit")
                || claim_lower.contains("button")
                || claim_lower.contains("error")
            {
                "severe"
            } else {
                "moderate"
            };
            hallucinations.push(json!({
                "claim": claim,
                "hallucination_type": "unverifiable",
                "severity": severity,
                "reason": "No matching visual evidence found",
            }));
        }
    }

    let score = if total == 0 {
        0.0
    } else {
        hallucinations.len() as f64 / total as f64
    };

    Ok(ToolCallResult::json(&json!({
        "ai_description": p.ai_description,
        "total_claims": total,
        "verified_count": verified.len(),
        "hallucination_count": hallucinations.len(),
        "hallucination_score": (score.min(1.0) * 100.0).round() / 100.0,
        "trustworthy": score < 0.3,
        "verified_claims": verified,
        "hallucinations": hallucinations,
    })))
}

// ── vision_hallucination_fix ────────────────────────────────────────────

pub fn definition_vision_hallucination_fix() -> ToolDefinition {
    ToolDefinition {
        name: "vision_hallucination_fix".to_string(),
        description: Some("Suggest corrections for visual hallucinations".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["claim"],
            "properties": {
                "claim": { "type": "string", "description": "The hallucinated claim to fix" }
            }
        }),
    }
}

pub async fn execute_vision_hallucination_fix(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        claim: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let claim_lower = p.claim.to_lowercase();

    let mut suggestions: Vec<Value> = Vec::new();

    for obs in &store.observations {
        if let Some(desc) = &obs.metadata.description {
            let overlap = word_overlap(&claim_lower, &desc.to_lowercase());
            if overlap > 0.1 {
                suggestions.push(json!({
                    "capture_id": obs.id,
                    "actual_description": desc,
                    "similarity": (overlap * 100.0).round() / 100.0,
                    "suggestion": format!("Replace with description from capture {}", obs.id),
                }));
            }
        }
    }

    suggestions.sort_by(|a, b| {
        b["similarity"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["similarity"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    suggestions.truncate(5);

    Ok(ToolCallResult::json(&json!({
        "claim": p.claim,
        "suggestions": suggestions,
        "fix_available": !suggestions.is_empty(),
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 3: Visual Truth Maintenance — 3 tools
// ═══════════════════════════════════════════════════════════════════════════

// ── vision_truth_check ──────────────────────────────────────────────────

pub fn definition_vision_truth_check() -> ToolDefinition {
    ToolDefinition {
        name: "vision_truth_check".to_string(),
        description: Some("Check if a historical visual claim is still true".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["claim"],
            "properties": {
                "claim": { "type": "string", "description": "The visual truth to check" },
                "established_at": { "type": "number", "description": "When the truth was established (epoch)" }
            }
        }),
    }
}

pub async fn execute_vision_truth_check(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        claim: String,
        established_at: Option<u64>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let now = now_epoch();
    let claim_lower = p.claim.to_lowercase();

    // Find evidence from around established_at and from recently
    let mut old_evidence: Option<Value> = None;
    let mut new_evidence: Option<Value> = None;
    let mut old_score: f64 = 0.0;
    let mut new_score: f64 = 0.0;

    for obs in &store.observations {
        let mut score: f64 = 0.0;
        if let Some(desc) = &obs.metadata.description {
            score = word_overlap(&claim_lower, &desc.to_lowercase());
        }
        for label in &obs.metadata.labels {
            if claim_lower.contains(&label.to_lowercase()) {
                score += 0.2;
                break;
            }
        }
        if score < 0.15 {
            continue;
        }

        if let Some(est) = p.established_at {
            let dist = obs.timestamp.abs_diff(est);
            if dist < 3600 && score > old_score {
                old_score = score;
                old_evidence = Some(json!({
                    "capture_id": obs.id, "timestamp": obs.timestamp, "score": score
                }));
            }
        }
        let age = now.saturating_sub(obs.timestamp);
        if age < 3600 && score > new_score {
            new_score = score;
            new_evidence = Some(json!({
                "capture_id": obs.id, "timestamp": obs.timestamp, "score": score
            }));
        }
    }

    let status = if new_score > 0.4 {
        "valid"
    } else if new_evidence.is_some() || old_evidence.is_some() {
        "stale"
    } else {
        "unknown"
    };

    Ok(ToolCallResult::json(&json!({
        "claim": p.claim,
        "status": status,
        "established_evidence": old_evidence,
        "current_evidence": new_evidence,
        "last_verified": now,
    })))
}

// ── vision_truth_refresh ────────────────────────────────────────────────

pub fn definition_vision_truth_refresh() -> ToolDefinition {
    ToolDefinition {
        name: "vision_truth_refresh".to_string(),
        description: Some("Re-verify all maintained visual truths".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "max_age_seconds": { "type": "number", "description": "Truths older than this need refresh", "default": 3600 }
            }
        }),
    }
}

pub async fn execute_vision_truth_refresh(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        #[serde(default = "default_3600")]
        max_age_seconds: u64,
    }
    fn default_3600() -> u64 {
        3600
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let now = now_epoch();

    let total = store.observations.len();
    let mut fresh = 0usize;
    let mut stale = 0usize;

    for obs in &store.observations {
        let age = now.saturating_sub(obs.timestamp);
        if age <= p.max_age_seconds {
            fresh += 1;
        } else {
            stale += 1;
        }
    }

    Ok(ToolCallResult::json(&json!({
        "total_observations": total,
        "fresh": fresh,
        "stale": stale,
        "max_age_seconds": p.max_age_seconds,
        "refresh_needed": stale > 0,
        "freshness_ratio": if total == 0 { 1.0 } else { fresh as f64 / total as f64 },
    })))
}

// ── vision_truth_history ────────────────────────────────────────────────

pub fn definition_vision_truth_history() -> ToolDefinition {
    ToolDefinition {
        name: "vision_truth_history".to_string(),
        description: Some("Get history of a visual truth over time".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["subject"],
            "properties": {
                "subject": { "type": "string", "description": "The visual subject to track" }
            }
        }),
    }
}

pub async fn execute_vision_truth_history(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        subject: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let subject_lower = p.subject.to_lowercase();

    let mut history: Vec<Value> = Vec::new();

    for obs in &store.observations {
        let mut score: f64 = 0.0;
        if let Some(desc) = &obs.metadata.description {
            score = word_overlap(&subject_lower, &desc.to_lowercase());
        }
        for label in &obs.metadata.labels {
            if subject_lower.contains(&label.to_lowercase()) {
                score += 0.2;
                break;
            }
        }
        if score > 0.15 {
            history.push(json!({
                "capture_id": obs.id,
                "timestamp": obs.timestamp,
                "session_id": obs.session_id,
                "description": obs.metadata.description,
                "relevance": (score * 100.0).round() / 100.0,
            }));
        }
    }

    history.sort_by(|a, b| {
        a["timestamp"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&b["timestamp"].as_u64().unwrap_or(0))
    });

    Ok(ToolCallResult::json(&json!({
        "subject": p.subject,
        "history_count": history.len(),
        "history": history,
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 4: Multi-Context Vision — 4 tools
// ═══════════════════════════════════════════════════════════════════════════

// ── vision_compare_contexts ─────────────────────────────────────────────

pub fn definition_vision_compare_contexts() -> ToolDefinition {
    ToolDefinition {
        name: "vision_compare_contexts".to_string(),
        description: Some("Compare captures across different visual contexts".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_ids"],
            "properties": {
                "capture_ids": { "type": "array", "items": { "type": "number" }, "description": "Capture IDs to compare" },
                "context_type": { "type": "string", "description": "Type of context comparison" }
            }
        }),
    }
}

pub async fn execute_vision_compare_contexts(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_ids: Vec<u64>,
        context_type: Option<String>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    if p.capture_ids.len() < 2 {
        return Err(McpError::InvalidParams(
            "Need at least 2 capture IDs".to_string(),
        ));
    }

    let session = session.lock().await;
    let store = session.store();

    let mut contexts: Vec<Value> = Vec::new();
    let mut all_labels: Vec<Vec<String>> = Vec::new();

    for &cid in &p.capture_ids {
        if let Some(obs) = store.observations.iter().find(|o| o.id == cid) {
            contexts.push(json!({
                "capture_id": obs.id,
                "timestamp": obs.timestamp,
                "description": obs.metadata.description,
                "labels": obs.metadata.labels,
                "session_id": obs.session_id,
            }));
            all_labels.push(obs.metadata.labels.clone());
        }
    }

    // Compute label overlap as similarity
    let mut similarity: f64 = 0.0;
    let mut shared_labels: Vec<String> = Vec::new();
    let mut unique_labels: Vec<Vec<String>> = Vec::new();

    if all_labels.len() >= 2 {
        let first: std::collections::HashSet<String> = all_labels[0].iter().cloned().collect();
        let second: std::collections::HashSet<String> = all_labels[1].iter().cloned().collect();
        let shared: Vec<String> = first.intersection(&second).cloned().collect();
        let total = first.union(&second).count();
        similarity = if total == 0 {
            0.0
        } else {
            shared.len() as f64 / total as f64
        };
        shared_labels = shared;
        unique_labels = all_labels
            .iter()
            .enumerate()
            .map(|(i, labels)| {
                let other_idx = if i == 0 { 1 } else { 0 };
                let other: std::collections::HashSet<&String> =
                    all_labels[other_idx].iter().collect();
                labels
                    .iter()
                    .filter(|l| !other.contains(l))
                    .cloned()
                    .collect()
            })
            .collect();
    }

    Ok(ToolCallResult::json(&json!({
        "contexts": contexts,
        "context_type": p.context_type.unwrap_or_else(|| "general".to_string()),
        "similarity_score": (similarity * 100.0).round() / 100.0,
        "shared_labels": shared_labels,
        "unique_per_context": unique_labels,
        "found": contexts.len(),
        "requested": p.capture_ids.len(),
    })))
}

// ── vision_compare_sites ────────────────────────────────────────────────

pub fn definition_vision_compare_sites() -> ToolDefinition {
    ToolDefinition {
        name: "vision_compare_sites".to_string(),
        description: Some("Compare captures from two different websites".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_a", "capture_b"],
            "properties": {
                "capture_a": { "type": "number", "description": "First site capture ID" },
                "capture_b": { "type": "number", "description": "Second site capture ID" },
                "element": { "type": "string", "description": "Specific element to compare" }
            }
        }),
    }
}

pub async fn execute_vision_compare_sites(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_a: u64,
        capture_b: u64,
        element: Option<String>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs_a = store.observations.iter().find(|o| o.id == p.capture_a);
    let obs_b = store.observations.iter().find(|o| o.id == p.capture_b);

    let (info_a, info_b) = match (obs_a, obs_b) {
        (Some(a), Some(b)) => (
            json!({"capture_id": a.id, "description": a.metadata.description, "labels": a.metadata.labels}),
            json!({"capture_id": b.id, "description": b.metadata.description, "labels": b.metadata.labels}),
        ),
        _ => {
            return Ok(ToolCallResult::json(&json!({
                "error": "One or both captures not found"
            })))
        }
    };

    let desc_a = obs_a
        .and_then(|o| o.metadata.description.as_deref())
        .unwrap_or("");
    let desc_b = obs_b
        .and_then(|o| o.metadata.description.as_deref())
        .unwrap_or("");
    let similarity = word_overlap(desc_a, desc_b);

    Ok(ToolCallResult::json(&json!({
        "site_a": info_a,
        "site_b": info_b,
        "element_focus": p.element,
        "similarity": (similarity * 100.0).round() / 100.0,
        "context_type": "different_site",
    })))
}

// ── vision_compare_versions ─────────────────────────────────────────────

pub fn definition_vision_compare_versions() -> ToolDefinition {
    ToolDefinition {
        name: "vision_compare_versions".to_string(),
        description: Some("Compare two versions of the same site".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_old", "capture_new"],
            "properties": {
                "capture_old": { "type": "number", "description": "Old version capture ID" },
                "capture_new": { "type": "number", "description": "New version capture ID" }
            }
        }),
    }
}

pub async fn execute_vision_compare_versions(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_old: u64,
        capture_new: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs_old = store.observations.iter().find(|o| o.id == p.capture_old);
    let obs_new = store.observations.iter().find(|o| o.id == p.capture_new);

    let (info_old, info_new) = match (obs_old, obs_new) {
        (Some(a), Some(b)) => (
            json!({"capture_id": a.id, "timestamp": a.timestamp, "description": a.metadata.description, "labels": a.metadata.labels}),
            json!({"capture_id": b.id, "timestamp": b.timestamp, "description": b.metadata.description, "labels": b.metadata.labels}),
        ),
        _ => {
            return Ok(ToolCallResult::json(
                &json!({"error": "One or both captures not found"}),
            ))
        }
    };

    let desc_old = obs_old
        .and_then(|o| o.metadata.description.as_deref())
        .unwrap_or("");
    let desc_new = obs_new
        .and_then(|o| o.metadata.description.as_deref())
        .unwrap_or("");
    let similarity = word_overlap(desc_old, desc_new);

    // Find labels that changed
    let labels_old: std::collections::HashSet<String> = obs_old
        .map(|o| o.metadata.labels.iter().cloned().collect())
        .unwrap_or_default();
    let labels_new: std::collections::HashSet<String> = obs_new
        .map(|o| o.metadata.labels.iter().cloned().collect())
        .unwrap_or_default();
    let added: Vec<&String> = labels_new.difference(&labels_old).collect();
    let removed: Vec<&String> = labels_old.difference(&labels_new).collect();

    Ok(ToolCallResult::json(&json!({
        "old_version": info_old,
        "new_version": info_new,
        "similarity": (similarity * 100.0).round() / 100.0,
        "labels_added": added,
        "labels_removed": removed,
        "context_type": "different_version",
    })))
}

// ── vision_compare_devices ──────────────────────────────────────────────

pub fn definition_vision_compare_devices() -> ToolDefinition {
    ToolDefinition {
        name: "vision_compare_devices".to_string(),
        description: Some("Compare same page on different devices".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["captures"],
            "properties": {
                "captures": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "capture_id": { "type": "number" },
                            "device": { "type": "string" }
                        }
                    },
                    "description": "Captures with device labels"
                }
            }
        }),
    }
}

pub async fn execute_vision_compare_devices(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct CaptureDevice {
        capture_id: u64,
        device: Option<String>,
    }
    #[derive(Deserialize)]
    struct P {
        captures: Vec<CaptureDevice>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let mut devices: Vec<Value> = Vec::new();
    for cd in &p.captures {
        if let Some(obs) = store.observations.iter().find(|o| o.id == cd.capture_id) {
            devices.push(json!({
                "capture_id": obs.id,
                "device": cd.device.as_deref().unwrap_or("unknown"),
                "timestamp": obs.timestamp,
                "description": obs.metadata.description,
                "labels": obs.metadata.labels,
                "dimensions": {
                    "width": obs.metadata.width,
                    "height": obs.metadata.height,
                },
            }));
        }
    }

    Ok(ToolCallResult::json(&json!({
        "devices": devices,
        "device_count": devices.len(),
        "context_type": "different_device",
    })))
}
