//! Tool: vision_evidence — Get detailed evidence for a visual claim.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct EvidenceParams {
    query: String,
    #[serde(default = "default_max")]
    max_results: usize,
}

fn default_max() -> usize {
    10
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_evidence".to_string(),
        description: Some(
            "Get detailed capture evidence for a visual claim. Returns matching \
             observations with timestamps, metadata, and visual details."
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["query"],
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The query to search evidence for"
                },
                "max_results": {
                    "type": "integer",
                    "default": 10,
                    "description": "Maximum number of evidence items"
                }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: EvidenceParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    if params.query.trim().is_empty() {
        return Ok(ToolCallResult::json(&json!({
            "count": 0,
            "evidence": []
        })));
    }

    let session = session.lock().await;
    let store = session.store();
    let query_lower = params.query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut evidence: Vec<(f32, Value)> = Vec::new();

    for obs in &store.observations {
        let mut score = 0.0f32;

        if let Some(ref desc) = obs.metadata.description {
            let desc_lower = desc.to_lowercase();
            let overlap = query_words
                .iter()
                .filter(|w| desc_lower.contains(**w))
                .count();
            score += overlap as f32 / query_words.len().max(1) as f32;
        }

        for label in &obs.metadata.labels {
            if query_lower.contains(&label.to_lowercase()) {
                score += 0.3;
            }
        }

        if score > 0.0 {
            let source_desc = match &obs.source {
                agentic_vision::CaptureSource::File { path } => format!("file:{path}"),
                agentic_vision::CaptureSource::Base64 { mime } => format!("base64:{mime}"),
                agentic_vision::CaptureSource::Screenshot { .. } => "screenshot".to_string(),
                agentic_vision::CaptureSource::Clipboard => "clipboard".to_string(),
            };

            evidence.push((
                score,
                json!({
                    "observation_id": obs.id,
                    "timestamp": obs.timestamp,
                    "session_id": obs.session_id,
                    "source": source_desc,
                    "labels": obs.metadata.labels,
                    "description": obs.metadata.description,
                    "quality_score": obs.metadata.quality_score,
                    "dimensions": {
                        "width": obs.metadata.original_width,
                        "height": obs.metadata.original_height,
                    },
                    "memory_link": obs.memory_link,
                    "score": score,
                }),
            ));
        }
    }

    evidence.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    evidence.truncate(params.max_results);

    let items: Vec<Value> = evidence.into_iter().map(|(_, v)| v).collect();

    Ok(ToolCallResult::json(&json!({
        "query": params.query,
        "count": items.len(),
        "evidence": items
    })))
}
