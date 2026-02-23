//! Tool: vision_query — Search visual memory.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

#[derive(Debug, Deserialize)]
struct QueryParams {
    #[serde(default)]
    session_ids: Vec<u32>,
    #[serde(default)]
    after: Option<u64>,
    #[serde(default)]
    before: Option<u64>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    description_contains: Option<String>,
    #[serde(default)]
    min_quality: Option<f32>,
    #[serde(default = "default_sort_by")]
    sort_by: String,
    #[serde(default = "default_max_results")]
    max_results: usize,
}

fn default_max_results() -> usize {
    20
}

fn default_sort_by() -> String {
    "recent".to_string()
}

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "vision_query".to_string(),
        description: Some("Search visual memory by filters".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "session_ids": { "type": "array", "items": { "type": "integer" } },
                "after": { "type": "integer", "description": "Unix timestamp" },
                "before": { "type": "integer", "description": "Unix timestamp" },
                "labels": { "type": "array", "items": { "type": "string" } },
                "description_contains": { "type": "string" },
                "min_quality": { "type": "number", "description": "Minimum quality score [0.0, 1.0]" },
                "sort_by": { "type": "string", "enum": ["recent", "quality"], "default": "recent" },
                "max_results": { "type": "integer", "default": 20 }
            }
        }),
    }
}

pub async fn execute(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    let params: QueryParams =
        serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    if let Some(min_q) = params.min_quality {
        if !(0.0..=1.0).contains(&min_q) {
            return Err(McpError::InvalidParams(
                "'min_quality' must be within [0.0, 1.0]".to_string(),
            ));
        }
    }

    if let (Some(after), Some(before)) = (params.after, params.before) {
        if after > before {
            return Err(McpError::InvalidParams(
                "'after' must be less than or equal to 'before'".to_string(),
            ));
        }
    }

    if params.max_results == 0 {
        return Err(McpError::InvalidParams(
            "'max_results' must be greater than 0".to_string(),
        ));
    }

    if params.sort_by != "recent" && params.sort_by != "quality" {
        return Err(McpError::InvalidParams(
            "'sort_by' must be one of: recent, quality".to_string(),
        ));
    }

    let session = session.lock().await;
    let store = session.store();

    let desc_contains = params
        .description_contains
        .as_ref()
        .map(|s| s.to_ascii_lowercase());

    let mut filtered: Vec<_> = store
        .observations
        .iter()
        .filter(|o| {
            if !params.session_ids.is_empty() && !params.session_ids.contains(&o.session_id) {
                return false;
            }
            if let Some(after) = params.after {
                if o.timestamp < after {
                    return false;
                }
            }
            if let Some(before) = params.before {
                if o.timestamp > before {
                    return false;
                }
            }
            if !params.labels.is_empty()
                && !params.labels.iter().any(|l| o.metadata.labels.contains(l))
            {
                return false;
            }
            if let Some(ref phrase) = desc_contains {
                let desc = o
                    .metadata
                    .description
                    .as_ref()
                    .map(|d| d.to_ascii_lowercase())
                    .unwrap_or_default();
                if !desc.contains(phrase) {
                    return false;
                }
            }
            if let Some(min_q) = params.min_quality {
                if o.metadata.quality_score < min_q {
                    return false;
                }
            }
            true
        })
        .collect();

    match params.sort_by.as_str() {
        "quality" => filtered.sort_by(|a, b| {
            b.metadata
                .quality_score
                .partial_cmp(&a.metadata.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.timestamp.cmp(&a.timestamp))
        }),
        "recent" => filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)),
        _ => unreachable!("sort_by is validated above"),
    }

    let results: Vec<Value> = filtered
        .into_iter()
        .take(params.max_results)
        .map(|o| {
            json!({
                "id": o.id,
                "timestamp": o.timestamp,
                "session_id": o.session_id,
                "dimensions": {
                    "width": o.metadata.original_width,
                    "height": o.metadata.original_height,
                },
                "labels": o.metadata.labels,
                "description": o.metadata.description,
                "quality_score": o.metadata.quality_score,
                "memory_link": o.memory_link,
            })
        })
        .collect();

    Ok(ToolCallResult::json(&json!({
        "total": results.len(),
        "observations": results,
    })))
}
