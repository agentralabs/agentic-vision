//! Tool: vision_health — Visual memory quality and operational status summary.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct HealthParams {
    #[serde(default = "default_stale_after_hours")]
    stale_after_hours: u64,
    #[serde(default = "default_low_quality")]
    low_quality_threshold: f32,
    #[serde(default = "default_limit")]
    max_examples: usize,
}

fn default_stale_after_hours() -> u64 {
    24 * 7
}
fn default_low_quality() -> f32 {
    0.45
}
fn default_limit() -> usize {
    20
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_health".to_string(),
        description: Some(
            "Summarize visual memory quality, staleness, and linkage coverage".to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "stale_after_hours": { "type": "integer", "default": 168 },
                "low_quality_threshold": { "type": "number", "default": 0.45 },
                "max_examples": { "type": "integer", "default": 20 }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: HealthParams = serde_json::from_value(args)
        .map_err(|e| McpError::InvalidParams(format!("invalid params: {e}")))?;

    let session = session.lock().await;
    let store = session.store();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let stale_cutoff = now.saturating_sub(params.stale_after_hours.saturating_mul(3600));

    let mut low_quality_ids = Vec::new();
    let mut stale_ids = Vec::new();
    let mut unlabeled_ids = Vec::new();
    let mut unlinked_ids = Vec::new();

    for obs in &store.observations {
        if obs.metadata.quality_score < params.low_quality_threshold {
            low_quality_ids.push(obs.id);
        }
        if obs.timestamp < stale_cutoff {
            stale_ids.push(obs.id);
        }
        if obs.metadata.labels.is_empty() {
            unlabeled_ids.push(obs.id);
        }
        if obs.memory_link.is_none() {
            unlinked_ids.push(obs.id);
        }
    }

    let total = store.observations.len().max(1);
    let low_quality_ratio = low_quality_ids.len() as f32 / total as f32;
    let unlinked_ratio = unlinked_ids.len() as f32 / total as f32;
    let low_quality_count = low_quality_ids.len();
    let stale_count = stale_ids.len();
    let unlabeled_count = unlabeled_ids.len();
    let unlinked_memory_count = unlinked_ids.len();
    let status = if low_quality_ratio > 0.50 || unlinked_ratio > 0.70 {
        "fail"
    } else if low_quality_ratio > 0.25 || !stale_ids.is_empty() || !unlabeled_ids.is_empty() {
        "warn"
    } else {
        "pass"
    };

    low_quality_ids.truncate(params.max_examples.max(1));
    stale_ids.truncate(params.max_examples.max(1));
    unlabeled_ids.truncate(params.max_examples.max(1));
    unlinked_ids.truncate(params.max_examples.max(1));

    Ok(ToolCallResult::json(&json!({
        "status": status,
        "summary": {
            "capture_count": store.observations.len(),
            "session_count": store.session_count,
            "low_quality_count": low_quality_count,
            "stale_count": stale_count,
            "unlabeled_count": unlabeled_count,
            "unlinked_memory_count": unlinked_memory_count
        },
        "examples": {
            "low_quality_ids": low_quality_ids,
            "stale_ids": stale_ids,
            "unlabeled_ids": unlabeled_ids,
            "unlinked_memory_ids": unlinked_ids
        }
    })))
}
