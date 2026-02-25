//! Tool: vision_workspace_add — Add an .avis file to a workspace.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::workspace::ContextRole;
use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct AddParams {
    workspace_id: String,
    path: String,
    #[serde(default = "default_role")]
    role: String,
    label: Option<String>,
}

fn default_role() -> String {
    "primary".to_string()
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_workspace_add".to_string(),
        description: Some("Add an .avis file to a vision workspace.".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["workspace_id", "path"],
            "properties": {
                "workspace_id": { "type": "string" },
                "path": { "type": "string", "description": "Path to .avis file" },
                "role": { "type": "string", "enum": ["primary", "secondary", "reference", "archive"], "default": "primary" },
                "label": { "type": "string" }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: AddParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let role = ContextRole::parse_str(&params.role)
        .ok_or_else(|| McpError::InvalidParams(format!("Invalid role: {}", params.role)))?;
    let mut session = session.lock().await;
    let ctx_id = session.workspace_manager_mut().add_context(
        &params.workspace_id,
        &params.path,
        role,
        params.label,
    )?;
    Ok(ToolCallResult::json(
        &json!({ "context_id": ctx_id, "workspace_id": params.workspace_id, "role": role.label(), "status": "added" }),
    ))
}
