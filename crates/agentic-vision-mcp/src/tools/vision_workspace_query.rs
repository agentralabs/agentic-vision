//! Tool: vision_workspace_query — Query across all vision contexts.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct QueryParams {
    workspace_id: String,
    query: String,
    #[serde(default = "default_max")]
    max_per_context: usize,
}

fn default_max() -> usize {
    10
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_workspace_query".to_string(),
        description: Some("Search across all vision contexts in a workspace.".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["workspace_id", "query"],
            "properties": {
                "workspace_id": { "type": "string" },
                "query": { "type": "string" },
                "max_per_context": { "type": "integer", "default": 10 }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: QueryParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let session = session.lock().await;
    let results = session.workspace_manager().query_all(
        &params.workspace_id,
        &params.query,
        params.max_per_context,
    )?;
    let total: usize = results.iter().map(|r| r.matches.len()).sum();
    let items: Vec<Value> = results
        .iter()
        .map(|cr| {
            let matches: Vec<Value> = cr.matches.iter().map(|m| {
                json!({ "observation_id": m.observation_id, "description": m.description, "labels": m.labels, "score": m.score })
            }).collect();
            json!({ "context_id": cr.context_id, "context_role": cr.context_role.label(), "match_count": cr.matches.len(), "matches": matches })
        })
        .collect();
    Ok(ToolCallResult::json(
        &json!({ "workspace_id": params.workspace_id, "query": params.query, "total_matches": total, "results": items }),
    ))
}
