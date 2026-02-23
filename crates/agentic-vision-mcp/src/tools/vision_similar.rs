//! Tool: vision_similar — Find visually similar captures.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct SimilarParams {
    #[serde(default)]
    capture_id: Option<u64>,
    #[serde(default)]
    embedding: Option<Vec<f32>>,
    #[serde(default = "default_top_k")]
    top_k: usize,
    #[serde(default = "default_min_similarity")]
    min_similarity: f32,
}

fn default_top_k() -> usize {
    10
}

fn default_min_similarity() -> f32 {
    0.7
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_similar".to_string(),
        description: Some("Find visually similar captures by embedding".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "capture_id": { "type": "integer", "description": "Find similar to this capture" },
                "embedding": {
                    "type": "array",
                    "items": { "type": "number" },
                    "description": "Or provide embedding directly"
                },
                "top_k": { "type": "integer", "default": 10 },
                "min_similarity": { "type": "number", "default": 0.7 }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: SimilarParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    if params.top_k == 0 {
        return Err(McpError::InvalidParams(
            "'top_k' must be greater than 0".to_string(),
        ));
    }

    if !(0.0..=1.0).contains(&params.min_similarity) {
        return Err(McpError::InvalidParams(
            "'min_similarity' must be within [0.0, 1.0]".to_string(),
        ));
    }

    if params.capture_id.is_some() && params.embedding.is_some() {
        return Err(McpError::InvalidParams(
            "Provide only one of 'capture_id' or 'embedding'".to_string(),
        ));
    }

    if let Some(embedding) = &params.embedding {
        if embedding.is_empty() {
            return Err(McpError::InvalidParams(
                "'embedding' must not be empty".to_string(),
            ));
        }
    }

    let session = session.lock().await;

    let matches = if let Some(capture_id) = params.capture_id {
        session.find_similar(capture_id, params.top_k, params.min_similarity)?
    } else if let Some(embedding) = &params.embedding {
        session.find_similar_by_embedding(embedding, params.top_k, params.min_similarity)
    } else {
        return Err(McpError::InvalidParams(
            "Either 'capture_id' or 'embedding' is required".to_string(),
        ));
    };

    let results: Vec<Value> = matches
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "similarity": m.similarity,
            })
        })
        .collect();

    Ok(ToolCallResult::json(&json!({
        "total": results.len(),
        "matches": results,
    })))
}
