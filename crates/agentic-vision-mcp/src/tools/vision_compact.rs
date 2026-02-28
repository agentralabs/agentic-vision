//! Compact MCP facade tools for low-token operation.
//!
//! These tools expose operation-based routing while preserving the existing
//! fine-grained tool surface for backward compatibility.

use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::session::VisionSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};

use super::registry::ToolRegistry;

fn op_schema(ops: &[&str], description: &str) -> Value {
    json!({
        "type": "object",
        "required": ["operation"],
        "properties": {
            "operation": {
                "type": "string",
                "enum": ops,
                "description": description
            },
            "params": {
                "type": "object",
                "description": "Arguments for the selected operation"
            }
        }
    })
}

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "vision_core".to_string(),
            description: Some(
                "Compact core facade: observation_log/capture/compare/query/ocr/similar/track/diff/health/link".to_string(),
            ),
            input_schema: op_schema(
                &[
                    "observation_log",
                    "capture",
                    "compare",
                    "query",
                    "ocr",
                    "similar",
                    "track",
                    "diff",
                    "health",
                    "link",
                ],
                "Core vision operation",
            ),
        },
        ToolDefinition {
            name: "vision_grounding".to_string(),
            description: Some("Compact grounding facade (v2 + v3 grounding tools)".to_string()),
            input_schema: op_schema(
                &[
                    "ground",
                    "evidence",
                    "suggest",
                    "ground_claim",
                    "verify_claim",
                    "cite",
                    "contradict",
                    "hallucination_check",
                    "hallucination_fix",
                    "truth_check",
                    "truth_refresh",
                    "truth_history",
                    "compare_contexts",
                    "compare_sites",
                    "compare_versions",
                    "compare_devices",
                ],
                "Grounding operation",
            ),
        },
        ToolDefinition {
            name: "vision_workspace".to_string(),
            description: Some(
                "Compact workspace facade: create/add/list/query/compare/xref".to_string(),
            ),
            input_schema: op_schema(
                &["create", "add", "list", "query", "compare", "xref"],
                "Workspace operation",
            ),
        },
        ToolDefinition {
            name: "vision_session".to_string(),
            description: Some("Compact session facade: start/end/resume".to_string()),
            input_schema: op_schema(&["start", "end", "resume"], "Session operation"),
        },
        ToolDefinition {
            name: "vision_temporal".to_string(),
            description: Some("Compact temporal facade".to_string()),
            input_schema: op_schema(
                &[
                    "at_time",
                    "timeline",
                    "reconstruct",
                    "archaeology_dig",
                    "archaeology_reconstruct",
                    "archaeology_report",
                    "consolidate",
                    "consolidate_preview",
                    "consolidate_policy",
                    "dejavu_check",
                    "dejavu_patterns",
                    "dejavu_alert",
                ],
                "Temporal operation",
            ),
        },
        ToolDefinition {
            name: "vision_prediction".to_string(),
            description: Some("Compact prediction facade".to_string()),
            input_schema: op_schema(
                &[
                    "prophecy",
                    "prophecy_diff",
                    "prophecy_compare",
                    "regression_predict",
                    "regression_test",
                    "regression_history",
                    "attention_predict",
                    "attention_optimize",
                    "attention_compare",
                    "phantom_create",
                    "phantom_compare",
                    "phantom_ab_test",
                ],
                "Prediction operation",
            ),
        },
        ToolDefinition {
            name: "vision_cognition".to_string(),
            description: Some("Compact cognition facade".to_string()),
            input_schema: op_schema(
                &[
                    "semantic_analyze",
                    "semantic_find",
                    "semantic_intent",
                    "reason",
                    "reason_about",
                    "reason_diagnose",
                    "bind_code",
                    "bind_memory",
                    "bind_identity",
                    "bind_time",
                    "traverse_binding",
                    "gestalt_analyze",
                    "gestalt_harmony",
                    "gestalt_improve",
                ],
                "Cognition operation",
            ),
        },
        ToolDefinition {
            name: "vision_synthesis".to_string(),
            description: Some("Compact synthesis facade".to_string()),
            input_schema: op_schema(
                &[
                    "dna_extract",
                    "dna_compare",
                    "dna_lineage",
                    "dna_mutate",
                    "composition_analyze",
                    "composition_score",
                    "composition_suggest",
                    "composition_compare",
                    "cluster_captures",
                    "cluster_outliers",
                    "cluster_timeline",
                ],
                "Synthesis operation",
            ),
        },
        ToolDefinition {
            name: "vision_forensics".to_string(),
            description: Some("Compact forensics facade".to_string()),
            input_schema: op_schema(
                &[
                    "forensic_diff",
                    "forensic_timeline",
                    "forensic_blame",
                    "forensic_reconstruct",
                    "anomaly_detect",
                    "anomaly_pattern",
                    "anomaly_baseline",
                    "anomaly_alert",
                    "regression_snapshot",
                    "regression_check",
                    "regression_report",
                ],
                "Forensics operation",
            ),
        },
    ]
}

fn decode_operation(args: Value) -> McpResult<(String, Value)> {
    let obj = args
        .as_object()
        .ok_or_else(|| McpError::InvalidParams("arguments must be an object".to_string()))?;
    let operation = obj
        .get("operation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("'operation' is required".to_string()))?
        .to_string();

    if let Some(params) = obj.get("params") {
        return Ok((operation, params.clone()));
    }

    let mut passthrough = obj.clone();
    passthrough.remove("operation");
    Ok((operation, Value::Object(passthrough)))
}

fn invalid(group: &str, operation: &str) -> McpError {
    McpError::InvalidParams(format!("Unknown {group} operation: {operation}"))
}

fn resolve_tool_name(group: &str, operation: &str) -> Result<String, McpError> {
    let name = match group {
        "vision_core" => match operation {
            "observation_log" => "observation_log",
            "capture" => "vision_capture",
            "compare" => "vision_compare",
            "query" => "vision_query",
            "ocr" => "vision_ocr",
            "similar" => "vision_similar",
            "track" => "vision_track",
            "diff" => "vision_diff",
            "health" => "vision_health",
            "link" => "vision_link",
            _ => return Err(invalid(group, operation)),
        },
        "vision_grounding" => match operation {
            "ground" => "vision_ground",
            "evidence" => "vision_evidence",
            "suggest" => "vision_suggest",
            "ground_claim" => "vision_ground_claim",
            "verify_claim" => "vision_verify_claim",
            "cite" => "vision_cite",
            "contradict" => "vision_contradict",
            "hallucination_check" => "vision_hallucination_check",
            "hallucination_fix" => "vision_hallucination_fix",
            "truth_check" => "vision_truth_check",
            "truth_refresh" => "vision_truth_refresh",
            "truth_history" => "vision_truth_history",
            "compare_contexts" => "vision_compare_contexts",
            "compare_sites" => "vision_compare_sites",
            "compare_versions" => "vision_compare_versions",
            "compare_devices" => "vision_compare_devices",
            _ => return Err(invalid(group, operation)),
        },
        "vision_workspace" => match operation {
            "create" => "vision_workspace_create",
            "add" => "vision_workspace_add",
            "list" => "vision_workspace_list",
            "query" => "vision_workspace_query",
            "compare" => "vision_workspace_compare",
            "xref" => "vision_workspace_xref",
            _ => return Err(invalid(group, operation)),
        },
        "vision_session" => match operation {
            "start" => "session_start",
            "end" => "session_end",
            "resume" => "vision_session_resume",
            _ => return Err(invalid(group, operation)),
        },
        "vision_temporal" => match operation {
            "at_time" => "vision_at_time",
            "timeline" => "vision_timeline",
            "reconstruct" => "vision_reconstruct",
            "archaeology_dig" => "vision_archaeology_dig",
            "archaeology_reconstruct" => "vision_archaeology_reconstruct",
            "archaeology_report" => "vision_archaeology_report",
            "consolidate" => "vision_consolidate",
            "consolidate_preview" => "vision_consolidate_preview",
            "consolidate_policy" => "vision_consolidate_policy",
            "dejavu_check" => "vision_dejavu_check",
            "dejavu_patterns" => "vision_dejavu_patterns",
            "dejavu_alert" => "vision_dejavu_alert",
            _ => return Err(invalid(group, operation)),
        },
        "vision_prediction" => match operation {
            "prophecy" => "vision_prophecy",
            "prophecy_diff" => "vision_prophecy_diff",
            "prophecy_compare" => "vision_prophecy_compare",
            "regression_predict" => "vision_regression_predict",
            "regression_test" => "vision_regression_test",
            "regression_history" => "vision_regression_history",
            "attention_predict" => "vision_attention_predict",
            "attention_optimize" => "vision_attention_optimize",
            "attention_compare" => "vision_attention_compare",
            "phantom_create" => "vision_phantom_create",
            "phantom_compare" => "vision_phantom_compare",
            "phantom_ab_test" => "vision_phantom_ab_test",
            _ => return Err(invalid(group, operation)),
        },
        "vision_cognition" => match operation {
            "semantic_analyze" => "vision_semantic_analyze",
            "semantic_find" => "vision_semantic_find",
            "semantic_intent" => "vision_semantic_intent",
            "reason" => "vision_reason",
            "reason_about" => "vision_reason_about",
            "reason_diagnose" => "vision_reason_diagnose",
            "bind_code" => "vision_bind_code",
            "bind_memory" => "vision_bind_memory",
            "bind_identity" => "vision_bind_identity",
            "bind_time" => "vision_bind_time",
            "traverse_binding" => "vision_traverse_binding",
            "gestalt_analyze" => "vision_gestalt_analyze",
            "gestalt_harmony" => "vision_gestalt_harmony",
            "gestalt_improve" => "vision_gestalt_improve",
            _ => return Err(invalid(group, operation)),
        },
        "vision_synthesis" => match operation {
            "dna_extract" => "vision_dna_extract",
            "dna_compare" => "vision_dna_compare",
            "dna_lineage" => "vision_dna_lineage",
            "dna_mutate" => "vision_dna_mutate",
            "composition_analyze" => "vision_composition_analyze",
            "composition_score" => "vision_composition_score",
            "composition_suggest" => "vision_composition_suggest",
            "composition_compare" => "vision_composition_compare",
            "cluster_captures" => "vision_cluster_captures",
            "cluster_outliers" => "vision_cluster_outliers",
            "cluster_timeline" => "vision_cluster_timeline",
            _ => return Err(invalid(group, operation)),
        },
        "vision_forensics" => match operation {
            "forensic_diff" => "vision_forensic_diff",
            "forensic_timeline" => "vision_forensic_timeline",
            "forensic_blame" => "vision_forensic_blame",
            "forensic_reconstruct" => "vision_forensic_reconstruct",
            "anomaly_detect" => "vision_anomaly_detect",
            "anomaly_pattern" => "vision_anomaly_pattern",
            "anomaly_baseline" => "vision_anomaly_baseline",
            "anomaly_alert" => "vision_anomaly_alert",
            "regression_snapshot" => "vision_regression_snapshot",
            "regression_check" => "vision_regression_check",
            "regression_report" => "vision_regression_report",
            _ => return Err(invalid(group, operation)),
        },
        _ => return Err(McpError::ToolNotFound(group.to_string())),
    };

    Ok(name.to_string())
}

pub async fn try_execute(
    name: &str,
    args: Value,
    session: &Arc<Mutex<VisionSessionManager>>,
) -> Option<McpResult<ToolCallResult>> {
    let (operation, params) = match decode_operation(args) {
        Ok(v) => v,
        Err(e) => return Some(Err(e)),
    };

    if !matches!(
        name,
        "vision_core"
            | "vision_grounding"
            | "vision_workspace"
            | "vision_session"
            | "vision_temporal"
            | "vision_prediction"
            | "vision_cognition"
            | "vision_synthesis"
            | "vision_forensics"
    ) {
        return None;
    }

    let resolved = match resolve_tool_name(name, &operation) {
        Ok(tool) => tool,
        Err(e) => return Some(Err(e)),
    };

    Some(ToolRegistry::call_legacy(&resolved, params, session).await)
}
