//! Tool: vision_workspace_list — List loaded vision contexts in a workspace.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct ListParams {
    workspace_id: String,
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_workspace_list".to_string(),
        description: Some("List all loaded vision contexts in a workspace.".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["workspace_id"],
            "properties": {
                "workspace_id": { "type": "string" }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: ListParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let session = session.lock().await;
    let contexts = session.workspace_manager().list(&params.workspace_id)?;
    let items: Vec<Value> = contexts
        .iter()
        .map(|ctx| {
            json!({
                "context_id": ctx.id,
                "role": ctx.role.label(),
                "path": ctx.path,
                "label": ctx.label,
                "observation_count": ctx.store.observations.len(),
            })
        })
        .collect();
    Ok(ToolCallResult::json(&json!({ "workspace_id": params.workspace_id, "count": items.len(), "contexts": items })))
}
