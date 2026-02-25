//! Tool: vision_workspace_xref — Cross-reference a topic across vision contexts.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct XrefParams {
    workspace_id: String,
    item: String,
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_workspace_xref".to_string(),
        description: Some("Find which vision contexts contain a visual element.".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["workspace_id", "item"],
            "properties": {
                "workspace_id": { "type": "string" },
                "item": { "type": "string" }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: XrefParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let session = session.lock().await;
    let xref = session
        .workspace_manager()
        .cross_reference(&params.workspace_id, &params.item)?;
    Ok(ToolCallResult::json(&json!({
        "item": xref.item,
        "present_in": xref.present_in,
        "absent_from": xref.absent_from,
        "coverage": format!("{}/{}", xref.present_in.len(), xref.present_in.len() + xref.absent_from.len())
    })))
}
