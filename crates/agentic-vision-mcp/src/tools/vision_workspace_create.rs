//! Tool: vision_workspace_create — Create a multi-vision workspace.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct CreateParams {
    name: String,
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_workspace_create".to_string(),
        description: Some(
            "Create a multi-vision workspace for loading and querying multiple .avis files."
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "required": ["name"],
            "properties": {
                "name": { "type": "string", "description": "Name for the workspace" }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: CreateParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let mut session = session.lock().await;
    let id = session.workspace_manager_mut().create(&params.name);
    Ok(ToolCallResult::json(
        &json!({ "workspace_id": id, "name": params.name, "status": "created" }),
    ))
}
