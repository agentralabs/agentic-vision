//! Tool: vision_suggest — Find similar captures for corrections/suggestions.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct SuggestParams {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    5
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_suggest".to_string(),
        description: Some(
            "Find similar captures when a visual claim doesn't match exactly. \
             Suggests related observations based on labels, descriptions, and timing."
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["query"],
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The query to find suggestions for"
                },
                "limit": {
                    "type": "integer",
                    "default": 5,
                    "description": "Maximum number of suggestions"
                }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: SuggestParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    if params.query.trim().is_empty() {
        return Ok(ToolCallResult::json(&json!({
            "query": params.query,
            "count": 0,
            "suggestions": []
        })));
    }

    let session = session.lock().await;
    let store = session.store();
    let query_lower = params.query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut suggestions: Vec<(f32, Value)> = Vec::new();

    for obs in &store.observations {
        let mut score = 0.0f32;

        if let Some(ref desc) = obs.metadata.description {
            let desc_lower = desc.to_lowercase();
            let overlap = query_words
                .iter()
                .filter(|w| desc_lower.contains(**w))
                .count();
            if overlap > 0 {
                score += overlap as f32 / query_words.len().max(1) as f32;
            }
        }

        for label in &obs.metadata.labels {
            let label_lower = label.to_lowercase();
            for word in &query_words {
                if label_lower.contains(*word) || word.contains(label_lower.as_str()) {
                    score += 0.2;
                }
            }
        }

        if score > 0.0 {
            suggestions.push((
                score,
                json!({
                    "observation_id": obs.id,
                    "labels": obs.metadata.labels,
                    "description": obs.metadata.description,
                    "quality_score": obs.metadata.quality_score,
                    "relevance_score": score,
                    "session_id": obs.session_id,
                    "timestamp": obs.timestamp,
                }),
            ));
        }
    }

    suggestions.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    suggestions.truncate(params.limit);

    let items: Vec<Value> = suggestions.into_iter().map(|(_, v)| v).collect();

    Ok(ToolCallResult::json(&json!({
        "query": params.query,
        "count": items.len(),
        "suggestions": items
    })))
}
