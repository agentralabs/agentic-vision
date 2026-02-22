//! Tool registration and dispatch.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde_json::Value;

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

use super::{
    session_end, session_start, vision_capture, vision_compare, vision_diff, vision_health,
    vision_link, vision_ocr, vision_query, vision_similar, vision_track,
};

pub struct ToolRegistry;

impl ToolRegistry {
    pub fn list_tools() -> Vec<ToolDefinition> {
        vec![
            vision_capture::definition(),
            vision_compare::definition(),
            vision_query::definition(),
            vision_ocr::definition(),
            vision_similar::definition(),
            vision_track::definition(),
            vision_diff::definition(),
            vision_health::definition(),
            vision_link::definition(),
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
            "vision_capture" => vision_capture::execute(args, session).await,
            "vision_compare" => vision_compare::execute(args, session).await,
            "vision_query" => vision_query::execute(args, session).await,
            "vision_ocr" => vision_ocr::execute(args, session).await,
            "vision_similar" => vision_similar::execute(args, session).await,
            "vision_track" => vision_track::execute(args, session).await,
            "vision_diff" => vision_diff::execute(args, session).await,
            "vision_health" => vision_health::execute(args, session).await,
            "vision_link" => vision_link::execute(args, session).await,
            "session_start" => session_start::execute(args, session).await,
            "session_end" => session_end::execute(args, session).await,
            _ => Err(McpError::ToolNotFound(name.to_string())),
        }
    }
}
