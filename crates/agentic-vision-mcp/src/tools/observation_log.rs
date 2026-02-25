//! Tool: observation_log — Log observation context and intent into the session.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::{ObservationNote, VisionSessionManager};
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct ObservationLogParams {
    intent: String,
    #[serde(default)]
    observation: Option<String>,
    #[serde(default)]
    related_capture_id: Option<u64>,
    #[serde(default)]
    topic: Option<String>,
}

/// Return the tool definition for observation_log.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "observation_log".to_string(),
        description: Some(
            "Log the intent and context behind a visual observation. \
             Call this to record WHY you are capturing or analyzing visual content. \
             Entries are linked into the session's temporal chain."
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "intent": {
                    "type": "string",
                    "description": "Why you are observing — the goal or reason for the visual action"
                },
                "observation": {
                    "type": "string",
                    "description": "What you noticed or concluded from the visual content"
                },
                "related_capture_id": {
                    "type": "integer",
                    "description": "Optional capture ID this observation relates to"
                },
                "topic": {
                    "type": "string",
                    "description": "Optional topic or category (e.g., 'ui-testing', 'layout-check')"
                }
            },
            "required": ["intent"]
        }),
    }
}

/// Execute the observation_log tool.
pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: ObservationLogParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    if params.intent.trim().is_empty() {
        return Err(McpError::InvalidParams(
            "'intent' must not be empty".to_string(),
        ));
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let note = ObservationNote {
        id: 0, // assigned by session manager
        intent: params.intent,
        observation: params.observation,
        related_capture_id: params.related_capture_id,
        topic: params.topic,
        timestamp: now,
    };

    let mut session = session.lock().await;
    let note_id = session.add_observation_note(note);

    Ok(ToolCallResult::json(&json!({
        "note_id": note_id,
        "message": "Observation context logged and linked to temporal chain"
    })))
}
