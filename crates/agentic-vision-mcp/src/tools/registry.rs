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

// V3: 16 Perception Inventions
use super::{invention_cognition, invention_grounding, invention_prediction, invention_temporal};

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
            // ── V3: Invention 1 — Visual Grounding ──
            invention_grounding::definition_vision_ground_claim(),
            invention_grounding::definition_vision_verify_claim(),
            invention_grounding::definition_vision_cite(),
            invention_grounding::definition_vision_contradict(),
            // ── V3: Invention 2 — Hallucination Detector ──
            invention_grounding::definition_vision_hallucination_check(),
            invention_grounding::definition_vision_hallucination_fix(),
            // ── V3: Invention 3 — Truth Maintenance ──
            invention_grounding::definition_vision_truth_check(),
            invention_grounding::definition_vision_truth_refresh(),
            invention_grounding::definition_vision_truth_history(),
            // ── V3: Invention 4 — Multi-Context Vision ──
            invention_grounding::definition_vision_compare_contexts(),
            invention_grounding::definition_vision_compare_sites(),
            invention_grounding::definition_vision_compare_versions(),
            invention_grounding::definition_vision_compare_devices(),
            // ── V3: Invention 5 — Temporal Vision ──
            invention_temporal::definition_vision_at_time(),
            invention_temporal::definition_vision_timeline(),
            invention_temporal::definition_vision_reconstruct(),
            // ── V3: Invention 6 — Visual Archaeology ──
            invention_temporal::definition_vision_archaeology_dig(),
            invention_temporal::definition_vision_archaeology_reconstruct(),
            invention_temporal::definition_vision_archaeology_report(),
            // ── V3: Invention 7 — Memory Consolidation ──
            invention_temporal::definition_vision_consolidate(),
            invention_temporal::definition_vision_consolidate_preview(),
            invention_temporal::definition_vision_consolidate_policy(),
            // ── V3: Invention 8 — Visual Déjà Vu ──
            invention_temporal::definition_vision_dejavu_check(),
            invention_temporal::definition_vision_dejavu_patterns(),
            invention_temporal::definition_vision_dejavu_alert(),
            // ── V3: Invention 9 — Visual Prophecy ──
            invention_prediction::definition_vision_prophecy(),
            invention_prediction::definition_vision_prophecy_diff(),
            invention_prediction::definition_vision_prophecy_compare(),
            // ── V3: Invention 10 — Regression Oracle ──
            invention_prediction::definition_vision_regression_predict(),
            invention_prediction::definition_vision_regression_test(),
            invention_prediction::definition_vision_regression_history(),
            // ── V3: Invention 11 — Attention Prediction ──
            invention_prediction::definition_vision_attention_predict(),
            invention_prediction::definition_vision_attention_optimize(),
            invention_prediction::definition_vision_attention_compare(),
            // ── V3: Invention 12 — Phantom Capture ──
            invention_prediction::definition_vision_phantom_create(),
            invention_prediction::definition_vision_phantom_compare(),
            invention_prediction::definition_vision_phantom_ab_test(),
            // ── V3: Invention 13 — Semantic Vision ──
            invention_cognition::definition_vision_semantic_analyze(),
            invention_cognition::definition_vision_semantic_find(),
            invention_cognition::definition_vision_semantic_intent(),
            // ── V3: Invention 14 — Visual Reasoning ──
            invention_cognition::definition_vision_reason(),
            invention_cognition::definition_vision_reason_about(),
            invention_cognition::definition_vision_reason_diagnose(),
            // ── V3: Invention 15 — Cross-Modal Binding ──
            invention_cognition::definition_vision_bind_code(),
            invention_cognition::definition_vision_bind_memory(),
            invention_cognition::definition_vision_bind_identity(),
            invention_cognition::definition_vision_bind_time(),
            invention_cognition::definition_vision_traverse_binding(),
            // ── V3: Invention 16 — Visual Gestalt ──
            invention_cognition::definition_vision_gestalt_analyze(),
            invention_cognition::definition_vision_gestalt_harmony(),
            invention_cognition::definition_vision_gestalt_improve(),
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
            // ── V3: Grounding Inventions (1–4) ──
            "vision_ground_claim" => {
                invention_grounding::execute_vision_ground_claim(args, session).await
            }
            "vision_verify_claim" => {
                invention_grounding::execute_vision_verify_claim(args, session).await
            }
            "vision_cite" => invention_grounding::execute_vision_cite(args, session).await,
            "vision_contradict" => {
                invention_grounding::execute_vision_contradict(args, session).await
            }
            "vision_hallucination_check" => {
                invention_grounding::execute_vision_hallucination_check(args, session).await
            }
            "vision_hallucination_fix" => {
                invention_grounding::execute_vision_hallucination_fix(args, session).await
            }
            "vision_truth_check" => {
                invention_grounding::execute_vision_truth_check(args, session).await
            }
            "vision_truth_refresh" => {
                invention_grounding::execute_vision_truth_refresh(args, session).await
            }
            "vision_truth_history" => {
                invention_grounding::execute_vision_truth_history(args, session).await
            }
            "vision_compare_contexts" => {
                invention_grounding::execute_vision_compare_contexts(args, session).await
            }
            "vision_compare_sites" => {
                invention_grounding::execute_vision_compare_sites(args, session).await
            }
            "vision_compare_versions" => {
                invention_grounding::execute_vision_compare_versions(args, session).await
            }
            "vision_compare_devices" => {
                invention_grounding::execute_vision_compare_devices(args, session).await
            }
            // ── V3: Temporal Inventions (5–8) ──
            "vision_at_time" => invention_temporal::execute_vision_at_time(args, session).await,
            "vision_timeline" => invention_temporal::execute_vision_timeline(args, session).await,
            "vision_reconstruct" => {
                invention_temporal::execute_vision_reconstruct(args, session).await
            }
            "vision_archaeology_dig" => {
                invention_temporal::execute_vision_archaeology_dig(args, session).await
            }
            "vision_archaeology_reconstruct" => {
                invention_temporal::execute_vision_archaeology_reconstruct(args, session).await
            }
            "vision_archaeology_report" => {
                invention_temporal::execute_vision_archaeology_report(args, session).await
            }
            "vision_consolidate" => {
                invention_temporal::execute_vision_consolidate(args, session).await
            }
            "vision_consolidate_preview" => {
                invention_temporal::execute_vision_consolidate_preview(args, session).await
            }
            "vision_consolidate_policy" => {
                invention_temporal::execute_vision_consolidate_policy(args, session).await
            }
            "vision_dejavu_check" => {
                invention_temporal::execute_vision_dejavu_check(args, session).await
            }
            "vision_dejavu_patterns" => {
                invention_temporal::execute_vision_dejavu_patterns(args, session).await
            }
            "vision_dejavu_alert" => {
                invention_temporal::execute_vision_dejavu_alert(args, session).await
            }
            // ── V3: Prediction Inventions (9–12) ──
            "vision_prophecy" => invention_prediction::execute_vision_prophecy(args, session).await,
            "vision_prophecy_diff" => {
                invention_prediction::execute_vision_prophecy_diff(args, session).await
            }
            "vision_prophecy_compare" => {
                invention_prediction::execute_vision_prophecy_compare(args, session).await
            }
            "vision_regression_predict" => {
                invention_prediction::execute_vision_regression_predict(args, session).await
            }
            "vision_regression_test" => {
                invention_prediction::execute_vision_regression_test(args, session).await
            }
            "vision_regression_history" => {
                invention_prediction::execute_vision_regression_history(args, session).await
            }
            "vision_attention_predict" => {
                invention_prediction::execute_vision_attention_predict(args, session).await
            }
            "vision_attention_optimize" => {
                invention_prediction::execute_vision_attention_optimize(args, session).await
            }
            "vision_attention_compare" => {
                invention_prediction::execute_vision_attention_compare(args, session).await
            }
            "vision_phantom_create" => {
                invention_prediction::execute_vision_phantom_create(args, session).await
            }
            "vision_phantom_compare" => {
                invention_prediction::execute_vision_phantom_compare(args, session).await
            }
            "vision_phantom_ab_test" => {
                invention_prediction::execute_vision_phantom_ab_test(args, session).await
            }
            // ── V3: Cognition Inventions (13–16) ──
            "vision_semantic_analyze" => {
                invention_cognition::execute_vision_semantic_analyze(args, session).await
            }
            "vision_semantic_find" => {
                invention_cognition::execute_vision_semantic_find(args, session).await
            }
            "vision_semantic_intent" => {
                invention_cognition::execute_vision_semantic_intent(args, session).await
            }
            "vision_reason" => invention_cognition::execute_vision_reason(args, session).await,
            "vision_reason_about" => {
                invention_cognition::execute_vision_reason_about(args, session).await
            }
            "vision_reason_diagnose" => {
                invention_cognition::execute_vision_reason_diagnose(args, session).await
            }
            "vision_bind_code" => {
                invention_cognition::execute_vision_bind_code(args, session).await
            }
            "vision_bind_memory" => {
                invention_cognition::execute_vision_bind_memory(args, session).await
            }
            "vision_bind_identity" => {
                invention_cognition::execute_vision_bind_identity(args, session).await
            }
            "vision_bind_time" => {
                invention_cognition::execute_vision_bind_time(args, session).await
            }
            "vision_traverse_binding" => {
                invention_cognition::execute_vision_traverse_binding(args, session).await
            }
            "vision_gestalt_analyze" => {
                invention_cognition::execute_vision_gestalt_analyze(args, session).await
            }
            "vision_gestalt_harmony" => {
                invention_cognition::execute_vision_gestalt_harmony(args, session).await
            }
            "vision_gestalt_improve" => {
                invention_cognition::execute_vision_gestalt_improve(args, session).await
            }
            _ => Err(McpError::ToolNotFound(name.to_string())),
        }
    }
}
