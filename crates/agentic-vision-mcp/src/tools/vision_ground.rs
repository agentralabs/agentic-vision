//! Tool: vision_ground — Verify a visual claim has capture backing (anti-hallucination).

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct GroundParams {
    claim: String,
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_ground".to_string(),
        description: Some(
            "Verify a visual claim has capture backing. Prevents hallucination about \
             what was seen, captured, or visually observed."
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["claim"],
            "properties": {
                "claim": {
                    "type": "string",
                    "description": "The visual claim to verify against stored captures"
                }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: GroundParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    if params.claim.trim().is_empty() {
        return Ok(ToolCallResult::json(&json!({
            "status": "ungrounded",
            "claim": params.claim,
            "reason": "Empty claim",
            "suggestions": []
        })));
    }

    let session = session.lock().await;
    let store = session.store();
    let claim_lower = params.claim.to_lowercase();
    let claim_words: Vec<&str> = claim_lower.split_whitespace().collect();

    // Search observations by description and labels
    let mut evidence: Vec<Value> = Vec::new();

    for obs in &store.observations {
        let mut score = 0.0f32;

        // Check description match
        if let Some(ref desc) = obs.metadata.description {
            let desc_lower = desc.to_lowercase();
            let word_overlap = claim_words
                .iter()
                .filter(|w| desc_lower.contains(**w))
                .count();
            if word_overlap > 0 {
                score += word_overlap as f32 / claim_words.len().max(1) as f32;
            }
        }

        // Check label match
        for label in &obs.metadata.labels {
            let label_lower = label.to_lowercase();
            if claim_lower.contains(&label_lower) || label_lower.contains(&claim_lower) {
                score += 0.5;
            }
        }

        if score > 0.0 {
            evidence.push(json!({
                "observation_id": obs.id,
                "timestamp": obs.timestamp,
                "session_id": obs.session_id,
                "labels": obs.metadata.labels,
                "description": obs.metadata.description,
                "quality_score": obs.metadata.quality_score,
                "score": score,
            }));
        }
    }

    // Sort by relevance score
    evidence.sort_by(|a, b| {
        let sa = a["score"].as_f64().unwrap_or(0.0);
        let sb = b["score"].as_f64().unwrap_or(0.0);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
    evidence.truncate(10);

    if evidence.is_empty() {
        let suggestions = suggest_similar(store, &claim_lower, &claim_words);
        return Ok(ToolCallResult::json(&json!({
            "status": "ungrounded",
            "claim": params.claim,
            "reason": "No visual captures match this claim",
            "suggestions": suggestions
        })));
    }

    let confidence = evidence[0]["score"].as_f64().unwrap_or(0.0).min(1.0);

    Ok(ToolCallResult::json(&json!({
        "status": "verified",
        "claim": params.claim,
        "confidence": confidence,
        "evidence_count": evidence.len(),
        "evidence": evidence
    })))
}

fn suggest_similar(
    store: &agentic_vision::VisualMemoryStore,
    _claim_lower: &str,
    claim_words: &[&str],
) -> Vec<String> {
    let mut suggestions: Vec<(f32, String)> = Vec::new();

    for obs in &store.observations {
        if let Some(ref desc) = obs.metadata.description {
            let desc_lower = desc.to_lowercase();
            let overlap = claim_words
                .iter()
                .filter(|w| desc_lower.contains(**w))
                .count();
            if overlap > 0 {
                let score = overlap as f32 / claim_words.len().max(1) as f32;
                let preview = if desc.len() > 80 {
                    format!("{}...", &desc[..80])
                } else {
                    desc.clone()
                };
                suggestions.push((score, preview));
            }
        }

        for label in &obs.metadata.labels {
            suggestions.push((0.2, format!("label: {}", label)));
        }
    }

    suggestions.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    suggestions.dedup_by(|a, b| a.1 == b.1);
    suggestions.truncate(5);
    suggestions.into_iter().map(|(_, s)| s).collect()
}
