//! Tool registration and dispatch.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde_json::Value;

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

use super::{
    observation_log, session_end, session_start, vision_capture, vision_compare, vision_diff,
    vision_evidence, vision_ground, vision_health, vision_link, vision_ocr, vision_query,
    vision_similar, vision_suggest, vision_track, vision_workspace_add, vision_workspace_compare,
    vision_workspace_create, vision_workspace_list, vision_workspace_query, vision_workspace_xref,
};

pub struct ToolRegistry;

impl ToolRegistry {
    pub fn list_tools() -> Vec<ToolDefinition> {
        vec![
            observation_log::definition(),
            vision_capture::definition(),
            vision_compare::definition(),
            vision_query::definition(),
            vision_ocr::definition(),
            vision_similar::definition(),
            vision_track::definition(),
            vision_diff::definition(),
            vision_health::definition(),
            vision_link::definition(),
            // V2: Grounding (anti-hallucination)
            vision_ground::definition(),
            vision_evidence::definition(),
            vision_suggest::definition(),
            // V2: Multi-context workspaces
            vision_workspace_create::definition(),
            vision_workspace_add::definition(),
            vision_workspace_list::definition(),
            vision_workspace_query::definition(),
            vision_workspace_compare::definition(),
            vision_workspace_xref::definition(),
            // Session lifecycle
            session_start::definition(),
            session_end::definition(),
        ]
    }

    pub async fn call(
        name: &str,
        arguments: Option<Value>,
        session: &Arc<Mutex<VisionSessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let args = arguments.unwrap_or(Value::Object(serde_json::Map::new()));

        match name {
            "observation_log" => observation_log::execute(args, session).await,
            "vision_capture" => vision_capture::execute(args, session).await,
            "vision_compare" => vision_compare::execute(args, session).await,
            "vision_query" => vision_query::execute(args, session).await,
            "vision_ocr" => vision_ocr::execute(args, session).await,
            "vision_similar" => vision_similar::execute(args, session).await,
            "vision_track" => vision_track::execute(args, session).await,
            "vision_diff" => vision_diff::execute(args, session).await,
            "vision_health" => vision_health::execute(args, session).await,
            "vision_link" => vision_link::execute(args, session).await,
            // V2: Grounding
            "vision_ground" => vision_ground::execute(args, session).await,
            "vision_evidence" => vision_evidence::execute(args, session).await,
            "vision_suggest" => vision_suggest::execute(args, session).await,
            // V2: Workspaces
            "vision_workspace_create" => vision_workspace_create::execute(args, session).await,
            "vision_workspace_add" => vision_workspace_add::execute(args, session).await,
            "vision_workspace_list" => vision_workspace_list::execute(args, session).await,
            "vision_workspace_query" => vision_workspace_query::execute(args, session).await,
            "vision_workspace_compare" => vision_workspace_compare::execute(args, session).await,
            "vision_workspace_xref" => vision_workspace_xref::execute(args, session).await,
            // Session
            "session_start" => session_start::execute(args, session).await,
            "session_end" => session_end::execute(args, session).await,
            _ => Err(McpError::ToolNotFound(name.to_string())),
        }
    }
}
