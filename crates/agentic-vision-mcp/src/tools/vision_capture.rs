//! Tool: vision_capture — Capture and store an image.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct CaptureParams {
    source: SourceParam,
    #[serde(default)]
    extract_ocr: bool,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SourceParam {
    #[serde(rename = "type")]
    source_type: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    data: Option<String>,
    #[serde(default)]
    mime: Option<String>,
    #[serde(default)]
    region: Option<RegionParam>,
}

/// Screen region for screenshot captures.
#[derive(Debug, Deserialize)]
struct RegionParam {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_capture".to_string(),
        description: Some("Capture an image and store it in visual memory".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "source": {
                    "type": "object",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["file", "base64", "screenshot", "clipboard"],
                            "description": "Source type"
                        },
                        "path": { "type": "string", "description": "File path (for type=file)" },
                        "data": { "type": "string", "description": "Base64 data (for type=base64)" },
                        "mime": { "type": "string", "description": "MIME type (for type=base64)" },
                        "region": {
                            "type": "object",
                            "description": "Screen region to capture (for type=screenshot). Omit for full screen.",
                            "properties": {
                                "x": { "type": "integer", "description": "X coordinate" },
                                "y": { "type": "integer", "description": "Y coordinate" },
                                "w": { "type": "integer", "description": "Width" },
                                "h": { "type": "integer", "description": "Height" }
                            },
                            "required": ["x", "y", "w", "h"]
                        }
                    },
                    "required": ["type"]
                },
                "extract_ocr": { "type": "boolean", "default": false },
                "description": { "type": "string" },
                "labels": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["source"]
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: CaptureParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let mut session = session.lock().await;

    let result = match params.source.source_type.as_str() {
        "file" => {
            let path = params.source.path.as_deref().ok_or_else(|| {
                McpError::InvalidParams("'path' required for file source".to_string())
            })?;
            session.capture(
                "file",
                path,
                params.source.mime.as_deref(),
                params.labels,
                params.description,
                params.extract_ocr,
            )?
        }
        "base64" => {
            let data = params.source.data.as_deref().ok_or_else(|| {
                McpError::InvalidParams("'data' required for base64 source".to_string())
            })?;
            session.capture(
                "base64",
                data,
                params.source.mime.as_deref(),
                params.labels,
                params.description,
                params.extract_ocr,
            )?
        }
        "screenshot" => {
            let region = params.source.region.map(|r| agentic_vision::Rect {
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
            });
            session.capture_screenshot(
                region,
                params.labels,
                params.description,
                params.extract_ocr,
            )?
        }
        "clipboard" => {
            session.capture_clipboard(params.labels, params.description, params.extract_ocr)?
        }
        other => {
            return Err(McpError::InvalidParams(format!(
                "Unsupported source type: {other}. Use 'file', 'base64', 'screenshot', or 'clipboard'."
            )));
        }
    };

    Ok(ToolCallResult::json(&json!({
        "capture_id": result.capture_id,
        "timestamp": result.timestamp,
        "dimensions": {
            "width": result.width,
            "height": result.height
        },
        "embedding_dims": result.embedding_dims,
        "quality_score": result.quality_score
    })))
}
