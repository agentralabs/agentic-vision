//! Tool: vision_workspace_compare — Compare a topic across vision contexts.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct CompareParams {
    workspace_id: String,
    item: String,
    #[serde(default = "default_max")]
    max_per_context: usize,
}

fn default_max() -> usize {
    5
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_workspace_compare".to_string(),
        description: Some("Compare a visual element across contexts.".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["workspace_id", "item"],
            "properties": {
                "workspace_id": { "type": "string" },
                "item": { "type": "string" },
                "max_per_context": { "type": "integer", "default": 5 }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: CompareParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let session = session.lock().await;
    let cmp = session.workspace_manager().compare(
        &params.workspace_id,
        &params.item,
        params.max_per_context,
    )?;
    let details: Vec<Value> = cmp.matches_per_context.iter().map(|(label, matches)| {
        let items: Vec<Value> = matches.iter().map(|m| {
            json!({ "observation_id": m.observation_id, "description": m.description, "labels": m.labels, "score": m.score })
        }).collect();
        json!({ "context": label, "matches": items })
    }).collect();
    Ok(ToolCallResult::json(
        &json!({ "item": cmp.item, "found_in": cmp.found_in, "missing_from": cmp.missing_from, "details": details }),
    ))
}
