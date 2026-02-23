//! Tool: vision_track — Start tracking a UI region for changes.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct TrackParams {
    region: RegionParam,
    #[serde(default = "default_interval")]
    interval_ms: u64,
    #[serde(default = "default_threshold")]
    on_change_threshold: f32,
    #[serde(default = "default_max_captures")]
    max_captures: u32,
}

#[derive(Debug, Deserialize)]
struct RegionParam {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

fn default_interval() -> u64 {
    1000
}

fn default_threshold() -> f32 {
    0.95
}

fn default_max_captures() -> u32 {
    100
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_track".to_string(),
        description: Some(
            "Configure tracking for a UI region (captures must be triggered externally)"
                .to_string(),
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "region": {
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer" },
                        "y": { "type": "integer" },
                        "w": { "type": "integer" },
                        "h": { "type": "integer" }
                    },
                    "required": ["x", "y", "w", "h"]
                },
                "interval_ms": { "type": "integer", "default": 1000 },
                "on_change_threshold": { "type": "number", "default": 0.95 },
                "max_captures": { "type": "integer", "default": 100 }
            },
            "required": ["region"]
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: TrackParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    if params.interval_ms == 0 {
        return Err(McpError::InvalidParams(
            "'interval_ms' must be greater than 0".to_string(),
        ));
    }

    if !(0.0..=1.0).contains(&params.on_change_threshold) {
        return Err(McpError::InvalidParams(
            "'on_change_threshold' must be within [0.0, 1.0]".to_string(),
        ));
    }

    if params.max_captures == 0 {
        return Err(McpError::InvalidParams(
            "'max_captures' must be greater than 0".to_string(),
        ));
    }

    if params.region.w == 0 || params.region.h == 0 {
        return Err(McpError::InvalidParams(
            "'region.w' and 'region.h' must be greater than 0".to_string(),
        ));
    }

    let _session = session.lock().await;
    let tracking_id = uuid::Uuid::new_v4().to_string();

    Ok(ToolCallResult::json(&json!({
        "tracking_id": tracking_id,
        "status": "configured",
        "region": {
            "x": params.region.x,
            "y": params.region.y,
            "w": params.region.w,
            "h": params.region.h,
        },
        "interval_ms": params.interval_ms,
        "on_change_threshold": params.on_change_threshold,
        "max_captures": params.max_captures,
        "message": "Tracking configured. Use vision_capture to take snapshots and vision_compare to detect changes."
    })))
}
