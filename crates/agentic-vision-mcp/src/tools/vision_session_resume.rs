//! Tool: vision_session_resume — Load context from recent vision sessions.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct ResumeParams {
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    5
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_session_resume".to_string(),
        description: Some("Load context from previous vision sessions".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Maximum number of recent records", "default": 5 }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: ResumeParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let limit = params.limit.max(1);

    let notes: Vec<Value> = session
        .observation_notes()
        .iter()
        .rev()
        .take(limit)
        .map(|n| {
            json!({
                "id": n.id,
                "intent": n.intent,
                "observation": n.observation,
                "related_capture_id": n.related_capture_id,
                "topic": n.topic,
                "timestamp": n.timestamp
            })
        })
        .collect();

    let tool_calls: Vec<Value> = session
        .tool_call_log()
        .iter()
        .rev()
        .take(limit)
        .map(|r| {
            json!({
                "tool_name": r.tool_name,
                "summary": r.summary,
                "timestamp": r.timestamp,
                "capture_id": r.capture_id
            })
        })
        .collect();

    Ok(ToolCallResult::json(&json!({
        "session_id": session.current_session_id(),
        "session_count": session.store().session_count,
        "total_captures": session.store().count(),
        "recent_notes": notes,
        "recent_tool_calls": tool_calls,
        "temporal_chain_edges": session.temporal_chain().len()
    })))
}
