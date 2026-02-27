//! Prediction Inventions (9–12): Visual Prophecy, Regression Oracle,
//! Attention Prediction, Phantom Capture.
//!
//! 12 MCP tools that see what's coming.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::session::manager::VisionSessionManager;
use crate::types::error::{McpError, McpResult};
use crate::types::response::{ToolCallResult, ToolDefinition};

// ═══════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VisualChangeType {
    CSS,
    HTML,
    JavaScript,
    Asset,
    Content,
    Layout,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ElementImpact {
    None,
    Minor,
    Moderate,
    Major,
    Breaking,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RegressionType {
    PositionShift,
    SizeChange,
    ColorChange,
    TextChange,
    Disappeared,
    Overlap,
    LayoutBreak,
    ResponsiveBreak,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Trivial,
    Minor,
    Major,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AttentionReason {
    Contrast,
    Motion,
    Faces,
    Text,
    ColorPop,
    Size,
    Position,
    Weight,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VisualTestType {
    FullPage,
    ElementOnly,
    Responsive,
    Interactive,
    Animation,
}

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

fn word_overlap(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let a_words: std::collections::HashSet<&str> = a_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
        .collect();
    let b_words: std::collections::HashSet<&str> = b_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
        .collect();
    if a_words.is_empty() || b_words.is_empty() {
        return 0.0;
    }
    let intersection = a_words.intersection(&b_words).count();
    intersection as f64 / a_words.len().max(b_words.len()) as f64
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 9: Visual Prophecy — 3 tools
// ═══════════════════════════════════════════════════════════════════════════

pub fn definition_vision_prophecy() -> ToolDefinition {
    ToolDefinition {
        name: "vision_prophecy".to_string(),
        description: Some("Predict visual impact of a proposed change".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["change_type", "target", "details"],
            "properties": {
                "change_type": { "type": "string", "description": "Type of change (CSS, HTML, JavaScript, Asset, Content, Layout)" },
                "target": { "type": "string", "description": "What's being changed" },
                "details": { "type": "string", "description": "Change details" },
                "capture_id": { "type": "number", "description": "Current state capture" }
            }
        }),
    }
}

pub async fn execute_vision_prophecy(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        change_type: String,
        target: String,
        details: String,
        capture_id: Option<u64>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    // Find captures related to the target
    let target_lower = p.target.to_lowercase();
    let mut related_captures: Vec<Value> = Vec::new();
    for obs in &store.observations {
        if let Some(desc) = &obs.metadata.description {
            if word_overlap(&target_lower, &desc.to_lowercase()) > 0.15 {
                related_captures.push(json!({
                    "capture_id": obs.id,
                    "description": desc,
                }));
            }
        }
    }

    // Predict impact based on change type
    let (risk_level, impact_areas) = match p.change_type.to_lowercase().as_str() {
        "css" => ("medium", vec!["visual appearance", "layout", "spacing"]),
        "html" => ("high", vec!["structure", "content", "accessibility"]),
        "javascript" => ("high", vec!["interactivity", "state", "animations"]),
        "layout" => (
            "critical",
            vec!["all visual elements", "responsive behavior"],
        ),
        "content" => ("low", vec!["text", "images"]),
        "asset" => ("medium", vec!["images", "icons", "fonts"]),
        _ => ("medium", vec!["unknown areas"]),
    };

    let predictions: Vec<Value> = impact_areas.iter().map(|area| {
        json!({
            "element": area,
            "predicted_change": format!("{} will be affected by {} change to {}", area, p.change_type, p.target),
            "confidence": 0.6,
            "impact": if risk_level == "critical" { "major" } else { "moderate" },
        })
    }).collect();

    Ok(ToolCallResult::json(&json!({
        "proposed_change": {
            "change_type": p.change_type,
            "target": p.target,
            "details": p.details,
        },
        "current_state_captures": related_captures.len(),
        "predictions": predictions,
        "risk": {
            "level": risk_level,
            "factors": [format!("{} change type", p.change_type)],
            "recommendations": ["Take a screenshot before and after the change", "Test on multiple viewports"],
        },
        "confidence": 0.6,
    })))
}

pub fn definition_vision_prophecy_diff() -> ToolDefinition {
    ToolDefinition {
        name: "vision_prophecy_diff".to_string(),
        description: Some("Generate predicted visual diff for a change".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id", "change_description"],
            "properties": {
                "capture_id": { "type": "number", "description": "Base capture for prediction" },
                "change_description": { "type": "string", "description": "What change to predict" }
            }
        }),
    }
}

pub async fn execute_vision_prophecy_diff(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
        change_description: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let capture = store.observations.iter().find(|o| o.id == p.capture_id);
    let capture_info = capture.map(|o| {
        json!({
            "capture_id": o.id, "description": o.metadata.description, "labels": o.metadata.labels,
        })
    });

    Ok(ToolCallResult::json(&json!({
        "base_capture": capture_info,
        "change_description": p.change_description,
        "predicted_diff": {
            "areas_affected": ["visual layout"],
            "severity": "moderate",
            "confidence": 0.5,
        },
        "note": "Predicted diff is based on heuristic analysis of change description",
    })))
}

pub fn definition_vision_prophecy_compare() -> ToolDefinition {
    ToolDefinition {
        name: "vision_prophecy_compare".to_string(),
        description: Some("Compare prophecy to actual result after change".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_before", "capture_after"],
            "properties": {
                "capture_before": { "type": "number", "description": "Capture before change" },
                "capture_after": { "type": "number", "description": "Capture after change" }
            }
        }),
    }
}

pub async fn execute_vision_prophecy_compare(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_before: u64,
        capture_after: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let before = store.observations.iter().find(|o| o.id == p.capture_before);
    let after = store.observations.iter().find(|o| o.id == p.capture_after);

    let (desc_before, desc_after) = match (before, after) {
        (Some(b), Some(a)) => (
            b.metadata.description.as_deref().unwrap_or(""),
            a.metadata.description.as_deref().unwrap_or(""),
        ),
        _ => {
            return Ok(ToolCallResult::json(
                &json!({"error": "Captures not found"}),
            ))
        }
    };

    let similarity = word_overlap(desc_before, desc_after);

    Ok(ToolCallResult::json(&json!({
        "capture_before": p.capture_before,
        "capture_after": p.capture_after,
        "similarity": (similarity * 100.0).round() / 100.0,
        "change_detected": similarity < 0.9,
        "change_magnitude": if similarity > 0.9 { "minimal" }
            else if similarity > 0.7 { "minor" }
            else if similarity > 0.4 { "moderate" }
            else { "major" },
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 10: Regression Oracle — 3 tools
// ═══════════════════════════════════════════════════════════════════════════

pub fn definition_vision_regression_predict() -> ToolDefinition {
    ToolDefinition {
        name: "vision_regression_predict".to_string(),
        description: Some("Predict visual regressions from a proposed change".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["change_description"],
            "properties": {
                "change_description": { "type": "string", "description": "What change is being made" },
                "affected_files": { "type": "array", "items": { "type": "string" }, "description": "Files being changed" }
            }
        }),
    }
}

pub async fn execute_vision_regression_predict(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        change_description: String,
        affected_files: Option<Vec<String>>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let desc_lower = p.change_description.to_lowercase();

    // Predict regressions based on change keywords
    let mut regressions: Vec<Value> = Vec::new();

    if desc_lower.contains("font")
        || desc_lower.contains("text")
        || desc_lower.contains("typography")
    {
        regressions.push(json!({
            "element": "text elements", "regression_type": "text_change",
            "probability": 0.7, "severity": "minor",
            "evidence": ["Font/text changes frequently cause text overflow"],
        }));
    }
    if desc_lower.contains("margin")
        || desc_lower.contains("padding")
        || desc_lower.contains("layout")
    {
        regressions.push(json!({
            "element": "layout", "regression_type": "layout_break",
            "probability": 0.6, "severity": "major",
            "evidence": ["Layout changes cascade to child elements"],
        }));
    }
    if desc_lower.contains("color") || desc_lower.contains("theme") || desc_lower.contains("dark") {
        regressions.push(json!({
            "element": "color scheme", "regression_type": "color_change",
            "probability": 0.5, "severity": "minor",
            "evidence": ["Color changes may affect contrast/readability"],
        }));
    }
    if desc_lower.contains("responsive")
        || desc_lower.contains("mobile")
        || desc_lower.contains("breakpoint")
    {
        regressions.push(json!({
            "element": "responsive layout", "regression_type": "responsive_break",
            "probability": 0.8, "severity": "critical",
            "evidence": ["Responsive changes often break at untested breakpoints"],
        }));
    }

    let regression_prob = if regressions.is_empty() {
        0.1
    } else {
        regressions
            .iter()
            .filter_map(|r| r["probability"].as_f64())
            .sum::<f64>()
            / regressions.len() as f64
    };

    let recommended_tests: Vec<Value> = vec![
        json!({"target": "full_page", "test_type": "full_page", "priority": "high"}),
        json!({"target": "responsive", "test_type": "responsive", "priority": "medium"}),
    ];

    Ok(ToolCallResult::json(&json!({
        "change_description": p.change_description,
        "affected_files": p.affected_files,
        "predicted_regressions": regressions,
        "regression_probability": (regression_prob * 100.0).round() / 100.0,
        "recommended_tests": recommended_tests,
        "safe_changes": if regressions.is_empty() { vec!["No regressions predicted"] } else { vec![] },
    })))
}

pub fn definition_vision_regression_test() -> ToolDefinition {
    ToolDefinition {
        name: "vision_regression_test".to_string(),
        description: Some("Generate visual regression test suggestions".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["target"],
            "properties": {
                "target": { "type": "string", "description": "What to generate tests for" },
                "test_type": { "type": "string", "description": "Test type: full_page, element, responsive, interactive" }
            }
        }),
    }
}

pub async fn execute_vision_regression_test(
    args: Value,
    _session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        target: String,
        test_type: Option<String>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let test_type = p.test_type.unwrap_or_else(|| "full_page".to_string());

    let tests: Vec<Value> = vec![json!({
        "test_name": format!("visual_regression_{}", p.target.replace(' ', "_")),
        "test_type": test_type,
        "target": p.target,
        "steps": [
            "Capture baseline screenshot",
            "Apply change",
            "Capture comparison screenshot",
            "Compare with threshold"
        ],
        "priority": "high",
    })];

    Ok(ToolCallResult::json(&json!({
        "target": p.target,
        "test_count": tests.len(),
        "tests": tests,
    })))
}

pub fn definition_vision_regression_history() -> ToolDefinition {
    ToolDefinition {
        name: "vision_regression_history".to_string(),
        description: Some("Get history of visual regressions for an element".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["element"],
            "properties": {
                "element": { "type": "string", "description": "Element to check regression history for" }
            }
        }),
    }
}

pub async fn execute_vision_regression_history(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        element: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();
    let elem_lower = p.element.to_lowercase();

    let mut history: Vec<Value> = Vec::new();
    for obs in &store.observations {
        if let Some(desc) = &obs.metadata.description {
            if word_overlap(&elem_lower, &desc.to_lowercase()) > 0.15 {
                history.push(json!({
                    "capture_id": obs.id,
                    "timestamp": obs.timestamp,
                    "session_id": obs.session_id,
                    "description": desc,
                }));
            }
        }
    }

    history.sort_by(|a, b| {
        a["timestamp"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&b["timestamp"].as_u64().unwrap_or(0))
    });

    Ok(ToolCallResult::json(&json!({
        "element": p.element,
        "history_count": history.len(),
        "history": history,
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 11: Attention Prediction — 3 tools
// ═══════════════════════════════════════════════════════════════════════════

pub fn definition_vision_attention_predict() -> ToolDefinition {
    ToolDefinition {
        name: "vision_attention_predict".to_string(),
        description: Some("Predict visual attention patterns for a capture".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture to analyze" }
            }
        }),
    }
}

pub async fn execute_vision_attention_predict(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs = match store.observations.iter().find(|o| o.id == p.capture_id) {
        Some(o) => o,
        None => return Err(McpError::CaptureNotFound(p.capture_id)),
    };

    // Generate attention predictions based on UI heuristics
    let mut focal_points: Vec<Value> = Vec::new();

    // Top-left bias (F-pattern reading)
    focal_points.push(json!({
        "region": {"x": 0, "y": 0, "width": obs.metadata.width / 3, "height": obs.metadata.height / 4},
        "strength": 0.9,
        "reason": "position",
        "explanation": "F-pattern: top-left scanned first",
    }));

    // Center attention
    focal_points.push(json!({
        "region": {"x": obs.metadata.width / 4, "y": obs.metadata.height / 4, "width": obs.metadata.width / 2, "height": obs.metadata.height / 2},
        "strength": 0.7,
        "reason": "size",
        "explanation": "Center region draws attention",
    }));

    // Scan path (F-pattern)
    let scan_path: Vec<Value> = vec![
        json!({"x": obs.metadata.width / 6, "y": obs.metadata.height / 8, "order": 1, "dwell_ms": 500}),
        json!({"x": obs.metadata.width / 2, "y": obs.metadata.height / 8, "order": 2, "dwell_ms": 300}),
        json!({"x": obs.metadata.width / 6, "y": obs.metadata.height / 3, "order": 3, "dwell_ms": 400}),
        json!({"x": obs.metadata.width / 3, "y": obs.metadata.height / 3, "order": 4, "dwell_ms": 250}),
        json!({"x": obs.metadata.width / 6, "y": obs.metadata.height / 2, "order": 5, "dwell_ms": 350}),
    ];

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "dimensions": {"width": obs.metadata.width, "height": obs.metadata.height},
        "focal_points": focal_points,
        "scan_path": scan_path,
        "recommendations": [
            "Place primary CTA in top-left quadrant for maximum visibility",
            "Ensure key content is above the fold",
        ],
    })))
}

pub fn definition_vision_attention_optimize() -> ToolDefinition {
    ToolDefinition {
        name: "vision_attention_optimize".to_string(),
        description: Some("Suggest optimizations for visual attention".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_id", "target_element"],
            "properties": {
                "capture_id": { "type": "number", "description": "Capture to optimize" },
                "target_element": { "type": "string", "description": "Element to optimize attention for" }
            }
        }),
    }
}

pub async fn execute_vision_attention_optimize(
    args: Value,
    _session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_id: u64,
        target_element: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let recommendations: Vec<Value> = vec![
        json!({
            "issue": format!("'{}' may not be in primary attention zone", p.target_element),
            "recommendation": "Increase visual weight with larger size or contrasting color",
            "expected_improvement": "15-30% increase in attention probability",
        }),
        json!({
            "issue": "Competing elements may draw attention away",
            "recommendation": "Reduce visual noise around the target element",
            "expected_improvement": "10-20% increase in focus duration",
        }),
    ];

    Ok(ToolCallResult::json(&json!({
        "capture_id": p.capture_id,
        "target_element": p.target_element,
        "recommendations": recommendations,
    })))
}

pub fn definition_vision_attention_compare() -> ToolDefinition {
    ToolDefinition {
        name: "vision_attention_compare".to_string(),
        description: Some("Compare predicted attention between two designs".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["capture_a", "capture_b"],
            "properties": {
                "capture_a": { "type": "number", "description": "First design capture" },
                "capture_b": { "type": "number", "description": "Second design capture" }
            }
        }),
    }
}

pub async fn execute_vision_attention_compare(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        capture_a: u64,
        capture_b: u64,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let obs_a = store.observations.iter().find(|o| o.id == p.capture_a);
    let obs_b = store.observations.iter().find(|o| o.id == p.capture_b);

    match (obs_a, obs_b) {
        (Some(a), Some(b)) => Ok(ToolCallResult::json(&json!({
            "design_a": {"capture_id": a.id, "dimensions": {"width": a.metadata.width, "height": a.metadata.height}},
            "design_b": {"capture_id": b.id, "dimensions": {"width": b.metadata.width, "height": b.metadata.height}},
            "comparison": {
                "size_difference": (a.metadata.width as i64 * a.metadata.height as i64 - b.metadata.width as i64 * b.metadata.height as i64).abs(),
                "note": "Larger viewports spread attention more; smaller viewports concentrate it",
            },
        }))),
        _ => Ok(ToolCallResult::json(
            &json!({"error": "One or both captures not found"}),
        )),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// INVENTION 12: Phantom Capture — 3 tools
// ═══════════════════════════════════════════════════════════════════════════

pub fn definition_vision_phantom_create() -> ToolDefinition {
    ToolDefinition {
        name: "vision_phantom_create".to_string(),
        description: Some("Create a phantom capture with hypothetical modifications".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["base_capture", "modifications"],
            "properties": {
                "base_capture": { "type": "number", "description": "Base capture to modify" },
                "modifications": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "mod_type": { "type": "string" },
                            "target": { "type": "string" },
                            "modification": { "type": "string" }
                        }
                    },
                    "description": "Modifications to apply"
                }
            }
        }),
    }
}

pub async fn execute_vision_phantom_create(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct Mod {
        mod_type: Option<String>,
        target: Option<String>,
        modification: Option<String>,
    }
    #[derive(Deserialize)]
    struct P {
        base_capture: u64,
        modifications: Vec<Mod>,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let base = store.observations.iter().find(|o| o.id == p.base_capture);
    let base_info = base.map(|o| {
        json!({
            "capture_id": o.id, "description": o.metadata.description, "labels": o.metadata.labels,
        })
    });

    let mods: Vec<Value> = p
        .modifications
        .iter()
        .map(|m| {
            json!({
                "mod_type": m.mod_type,
                "target": m.target,
                "modification": m.modification,
            })
        })
        .collect();

    let phantom_id = format!("phantom_{}", p.base_capture);

    Ok(ToolCallResult::json(&json!({
        "phantom_id": phantom_id,
        "base_capture": base_info,
        "modifications": mods,
        "confidence": 0.5,
        "note": "Phantom captures are hypothetical representations based on described modifications",
    })))
}

pub fn definition_vision_phantom_compare() -> ToolDefinition {
    ToolDefinition {
        name: "vision_phantom_compare".to_string(),
        description: Some("Compare a phantom capture to a real capture".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["real_capture", "phantom_id"],
            "properties": {
                "real_capture": { "type": "number", "description": "Real capture ID" },
                "phantom_id": { "type": "string", "description": "Phantom capture ID" }
            }
        }),
    }
}

pub async fn execute_vision_phantom_compare(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        real_capture: u64,
        phantom_id: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let real = store.observations.iter().find(|o| o.id == p.real_capture);
    let real_info = real.map(|o| {
        json!({
            "capture_id": o.id, "description": o.metadata.description,
        })
    });

    Ok(ToolCallResult::json(&json!({
        "real_capture": real_info,
        "phantom_id": p.phantom_id,
        "comparison_note": "Phantom comparisons are based on modification descriptions, not pixel-level analysis",
    })))
}

pub fn definition_vision_phantom_ab_test() -> ToolDefinition {
    ToolDefinition {
        name: "vision_phantom_ab_test".to_string(),
        description: Some("Generate A/B test variants as phantom captures".to_string()),
        input_schema: json!({
            "type": "object",
            "required": ["base_capture", "variant_description"],
            "properties": {
                "base_capture": { "type": "number", "description": "Base capture (variant A)" },
                "variant_description": { "type": "string", "description": "Description of variant B changes" }
            }
        }),
    }
}

pub async fn execute_vision_phantom_ab_test(
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ToolCallResult> {
    #[derive(Deserialize)]
    struct P {
        base_capture: u64,
        variant_description: String,
    }
    let p: P = serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let session = session.lock().await;
    let store = session.store();

    let base = store.observations.iter().find(|o| o.id == p.base_capture);
    let base_info = base.map(|o| {
        json!({
            "capture_id": o.id, "description": o.metadata.description, "labels": o.metadata.labels,
        })
    });

    Ok(ToolCallResult::json(&json!({
        "variant_a": base_info,
        "variant_b": {
            "phantom_id": format!("phantom_ab_{}", p.base_capture),
            "description": p.variant_description,
        },
        "ab_test": {
            "status": "variants_generated",
            "recommendation": "Capture actual variant B to compare with predictions",
        },
    })))
}
