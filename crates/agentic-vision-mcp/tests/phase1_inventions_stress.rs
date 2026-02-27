//! Phase 1 Stress Tests: 16 Perception Inventions — 51 new tools.
//! Tests tool dispatch, basic arg handling, and scale for all invention tools.

use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use agentic_vision_mcp::protocol::ProtocolHandler;
use agentic_vision_mcp::session::VisionSessionManager;
use agentic_vision_mcp::types::*;

// ─────────────────────── helpers ───────────────────────

fn temp_session(dir: &tempfile::TempDir) -> VisionSessionManager {
    let path = dir.path().join("inv_stress.avis");
    VisionSessionManager::open(path.to_str().unwrap(), None).unwrap()
}

fn arc_session(dir: &tempfile::TempDir) -> Arc<Mutex<VisionSessionManager>> {
    Arc::new(Mutex::new(temp_session(dir)))
}

fn mcp_request(id: i64, method: &str, params: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    })
}

fn init_request() -> Value {
    mcp_request(
        0,
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "inv-stress-test", "version": "1.0" }
        }),
    )
}

async fn send(handler: &ProtocolHandler, msg: Value) -> Option<Value> {
    let parsed: JsonRpcMessage = serde_json::from_value(msg).unwrap();
    handler.handle_message(parsed).await
}

async fn send_unwrap(handler: &ProtocolHandler, msg: Value) -> Value {
    send(handler, msg).await.expect("expected response")
}

fn tiny_png() -> Vec<u8> {
    let img = image::DynamicImage::new_rgb8(1, 1);
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    img.write_with_encoder(encoder).unwrap();
    buf
}

use base64::Engine;
fn tiny_png_base64() -> String {
    base64::engine::general_purpose::STANDARD.encode(tiny_png())
}

async fn capture_via_protocol(handler: &ProtocolHandler, id: i64) -> u64 {
    let b64 = tiny_png_base64();
    let resp = send_unwrap(
        handler,
        mcp_request(
            id,
            "tools/call",
            json!({
                "name": "vision_capture",
                "arguments": {
                    "source": { "type": "base64", "data": b64, "mime": "image/png" },
                    "labels": ["test", "login-page"],
                    "description": format!("Test capture {id}")
                }
            }),
        ),
    )
    .await;
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    parsed["capture_id"].as_u64().unwrap()
}

/// Call a tool and assert it returns a result (not an error).
async fn call_tool_ok(handler: &ProtocolHandler, id: i64, name: &str, args: Value) -> Value {
    let resp = send_unwrap(
        handler,
        mcp_request(id, "tools/call", json!({ "name": name, "arguments": args })),
    )
    .await;
    assert!(
        resp.get("result").is_some(),
        "Tool '{}' should succeed, got: {}",
        name,
        resp
    );
    resp
}

/// Setup: init + capture a few images for the tests.
async fn setup_with_captures(n: usize) -> (tempfile::TempDir, ProtocolHandler, Vec<u64>) {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let mut ids = Vec::new();
    for i in 0..n {
        ids.push(capture_via_protocol(&handler, (i + 1) as i64).await);
    }
    (dir, handler, ids)
}

// ============================================================================
// 1. Tool list includes all 51 invention tools
// ============================================================================

#[tokio::test]
async fn test_tool_list_includes_all_inventions() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(&handler, mcp_request(1, "tools/list", json!({}))).await;
    let tools = resp["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    let expected = vec![
        // Grounding inventions (1–4)
        "vision_ground_claim",
        "vision_verify_claim",
        "vision_cite",
        "vision_contradict",
        "vision_hallucination_check",
        "vision_hallucination_fix",
        "vision_truth_check",
        "vision_truth_refresh",
        "vision_truth_history",
        "vision_compare_contexts",
        "vision_compare_sites",
        "vision_compare_versions",
        "vision_compare_devices",
        // Temporal inventions (5–8)
        "vision_at_time",
        "vision_timeline",
        "vision_reconstruct",
        "vision_archaeology_dig",
        "vision_archaeology_reconstruct",
        "vision_archaeology_report",
        "vision_consolidate",
        "vision_consolidate_preview",
        "vision_consolidate_policy",
        "vision_dejavu_check",
        "vision_dejavu_patterns",
        "vision_dejavu_alert",
        // Prediction inventions (9–12)
        "vision_prophecy",
        "vision_prophecy_diff",
        "vision_prophecy_compare",
        "vision_regression_predict",
        "vision_regression_test",
        "vision_regression_history",
        "vision_attention_predict",
        "vision_attention_optimize",
        "vision_attention_compare",
        "vision_phantom_create",
        "vision_phantom_compare",
        "vision_phantom_ab_test",
        // Cognition inventions (13–16)
        "vision_semantic_analyze",
        "vision_semantic_find",
        "vision_semantic_intent",
        "vision_reason",
        "vision_reason_about",
        "vision_reason_diagnose",
        "vision_bind_code",
        "vision_bind_memory",
        "vision_bind_identity",
        "vision_bind_time",
        "vision_traverse_binding",
        "vision_gestalt_analyze",
        "vision_gestalt_harmony",
        "vision_gestalt_improve",
    ];

    for tool_name in &expected {
        assert!(
            names.contains(tool_name),
            "Missing invention tool: {}. Found: {:?}",
            tool_name,
            names
        );
    }
    assert_eq!(
        expected.len(),
        51,
        "Should expect exactly 51 invention tools"
    );
}

// ============================================================================
// 2. Grounding Inventions (1–4) — Smoke Tests
// ============================================================================

#[tokio::test]
async fn test_grounding_invention_tools() {
    let (_dir, handler, ids) = setup_with_captures(3).await;

    // Invention 1: Visual Grounding
    // vision_ground_claim: required ["claim"]
    call_tool_ok(
        &handler,
        10,
        "vision_ground_claim",
        json!({
            "claim": "The login page has a submit button"
        }),
    )
    .await;

    // vision_verify_claim: required ["claim"]
    call_tool_ok(
        &handler,
        11,
        "vision_verify_claim",
        json!({
            "claim": "Test capture exists"
        }),
    )
    .await;

    // vision_cite: required ["element"]
    call_tool_ok(
        &handler,
        12,
        "vision_cite",
        json!({
            "element": "submit button"
        }),
    )
    .await;

    // vision_contradict: required ["claim"]
    call_tool_ok(
        &handler,
        13,
        "vision_contradict",
        json!({
            "claim": "There is a purple elephant on the page"
        }),
    )
    .await;

    // Invention 2: Hallucination Detector
    // vision_hallucination_check: required ["ai_description"]
    call_tool_ok(
        &handler,
        14,
        "vision_hallucination_check",
        json!({
            "ai_description": "The login page shows a large red warning banner with error text"
        }),
    )
    .await;

    // vision_hallucination_fix: required ["claim"]
    call_tool_ok(
        &handler,
        15,
        "vision_hallucination_fix",
        json!({
            "claim": "I see a dashboard with graphs"
        }),
    )
    .await;

    // Invention 3: Truth Maintenance
    // vision_truth_check: required ["claim"]
    call_tool_ok(
        &handler,
        16,
        "vision_truth_check",
        json!({
            "claim": "The button is blue"
        }),
    )
    .await;

    // vision_truth_refresh: no required fields (only optional max_age_seconds)
    call_tool_ok(&handler, 17, "vision_truth_refresh", json!({})).await;

    // vision_truth_history: required ["subject"]
    call_tool_ok(
        &handler,
        18,
        "vision_truth_history",
        json!({
            "subject": "The page layout"
        }),
    )
    .await;

    // Invention 4: Multi-Context Vision
    // vision_compare_contexts: required ["capture_ids"] (array of numbers)
    call_tool_ok(
        &handler,
        19,
        "vision_compare_contexts",
        json!({
            "capture_ids": [ids[0], ids[1]]
        }),
    )
    .await;

    // vision_compare_sites: required ["capture_a", "capture_b"] (numbers)
    call_tool_ok(
        &handler,
        20,
        "vision_compare_sites",
        json!({
            "capture_a": ids[0], "capture_b": ids[1]
        }),
    )
    .await;

    // vision_compare_versions: required ["capture_old", "capture_new"] (numbers)
    call_tool_ok(
        &handler,
        21,
        "vision_compare_versions",
        json!({
            "capture_old": ids[0], "capture_new": ids[1]
        }),
    )
    .await;

    // vision_compare_devices: required ["captures"] (array of {capture_id, device})
    call_tool_ok(
        &handler,
        22,
        "vision_compare_devices",
        json!({
            "captures": [
                { "capture_id": ids[0], "device": "iPhone 15" },
                { "capture_id": ids[1], "device": "Pixel 8" }
            ]
        }),
    )
    .await;
}

// ============================================================================
// 3. Temporal Inventions (5–8) — Smoke Tests
// ============================================================================

#[tokio::test]
async fn test_temporal_invention_tools() {
    let (_dir, handler, ids) = setup_with_captures(3).await;

    // Invention 5: Temporal Vision
    // vision_at_time: required ["target_time"]
    call_tool_ok(
        &handler,
        30,
        "vision_at_time",
        json!({
            "target_time": 1700000000
        }),
    )
    .await;

    // vision_timeline: no required fields (optional subject, start_time, end_time)
    call_tool_ok(
        &handler,
        31,
        "vision_timeline",
        json!({
            "subject": "login"
        }),
    )
    .await;

    // vision_reconstruct: required ["target_time"]
    call_tool_ok(
        &handler,
        32,
        "vision_reconstruct",
        json!({
            "target_time": 1700000000
        }),
    )
    .await;

    // Invention 6: Visual Archaeology
    // vision_archaeology_dig: required ["target"]
    call_tool_ok(
        &handler,
        33,
        "vision_archaeology_dig",
        json!({
            "target": "login page"
        }),
    )
    .await;

    // vision_archaeology_reconstruct: required ["target"]
    call_tool_ok(
        &handler,
        34,
        "vision_archaeology_reconstruct",
        json!({
            "target": "button layout"
        }),
    )
    .await;

    // vision_archaeology_report: required ["target"]
    call_tool_ok(
        &handler,
        35,
        "vision_archaeology_report",
        json!({
            "target": "test"
        }),
    )
    .await;

    // Invention 7: Memory Consolidation
    // vision_consolidate: no required fields
    call_tool_ok(&handler, 36, "vision_consolidate", json!({})).await;

    // vision_consolidate_preview: no required fields
    call_tool_ok(&handler, 37, "vision_consolidate_preview", json!({})).await;

    // vision_consolidate_policy: no required fields (all optional)
    call_tool_ok(
        &handler,
        38,
        "vision_consolidate_policy",
        json!({
            "max_age_hours": 24
        }),
    )
    .await;

    // Invention 8: Visual Deja Vu
    // vision_dejavu_check: required ["capture_id"]
    call_tool_ok(
        &handler,
        39,
        "vision_dejavu_check",
        json!({
            "capture_id": ids[0]
        }),
    )
    .await;

    // vision_dejavu_patterns: no required fields (optional min_occurrences)
    call_tool_ok(&handler, 40, "vision_dejavu_patterns", json!({})).await;

    // vision_dejavu_alert: required ["pattern_labels"]
    call_tool_ok(
        &handler,
        41,
        "vision_dejavu_alert",
        json!({
            "pattern_labels": ["error", "login"]
        }),
    )
    .await;
}

// ============================================================================
// 4. Prediction Inventions (9–12) — Smoke Tests
// ============================================================================

#[tokio::test]
async fn test_prediction_invention_tools() {
    let (_dir, handler, ids) = setup_with_captures(2).await;

    // Invention 9: Visual Prophecy
    // vision_prophecy: required ["change_type", "target", "details"]
    call_tool_ok(
        &handler,
        50,
        "vision_prophecy",
        json!({
            "change_type": "CSS", "target": "button", "details": "Change primary color to red"
        }),
    )
    .await;

    // vision_prophecy_diff: required ["capture_id", "change_description"]
    call_tool_ok(
        &handler,
        51,
        "vision_prophecy_diff",
        json!({
            "capture_id": ids[0], "change_description": "Add new banner"
        }),
    )
    .await;

    // vision_prophecy_compare: required ["capture_before", "capture_after"]
    call_tool_ok(
        &handler,
        52,
        "vision_prophecy_compare",
        json!({
            "capture_before": ids[0], "capture_after": ids[1]
        }),
    )
    .await;

    // Invention 10: Regression Oracle
    // vision_regression_predict: required ["change_description"]
    call_tool_ok(
        &handler,
        53,
        "vision_regression_predict",
        json!({
            "change_description": "Update font size from 14px to 16px"
        }),
    )
    .await;

    // vision_regression_test: required ["target"]
    call_tool_ok(
        &handler,
        54,
        "vision_regression_test",
        json!({
            "target": "Login form"
        }),
    )
    .await;

    // vision_regression_history: required ["element"]
    call_tool_ok(
        &handler,
        55,
        "vision_regression_history",
        json!({
            "element": "login form"
        }),
    )
    .await;

    // Invention 11: Attention Prediction
    // vision_attention_predict: required ["capture_id"]
    call_tool_ok(
        &handler,
        56,
        "vision_attention_predict",
        json!({
            "capture_id": ids[0]
        }),
    )
    .await;

    // vision_attention_optimize: required ["capture_id", "target_element"]
    call_tool_ok(
        &handler,
        57,
        "vision_attention_optimize",
        json!({
            "capture_id": ids[0], "target_element": "call-to-action button"
        }),
    )
    .await;

    // vision_attention_compare: required ["capture_a", "capture_b"]
    call_tool_ok(
        &handler,
        58,
        "vision_attention_compare",
        json!({
            "capture_a": ids[0], "capture_b": ids[1]
        }),
    )
    .await;

    // Invention 12: Phantom Capture
    // vision_phantom_create: required ["base_capture", "modifications"]
    call_tool_ok(&handler, 59, "vision_phantom_create", json!({
        "base_capture": ids[0],
        "modifications": [
            { "mod_type": "color", "target": "background", "modification": "change to dark mode" }
        ]
    })).await;

    // vision_phantom_compare: required ["real_capture", "phantom_id"]
    call_tool_ok(
        &handler,
        60,
        "vision_phantom_compare",
        json!({
            "real_capture": ids[0], "phantom_id": "phantom_1"
        }),
    )
    .await;

    // vision_phantom_ab_test: required ["base_capture", "variant_description"]
    call_tool_ok(
        &handler,
        61,
        "vision_phantom_ab_test",
        json!({
            "base_capture": ids[0], "variant_description": "Proposed redesign with larger CTA"
        }),
    )
    .await;
}

// ============================================================================
// 5. Cognition Inventions (13–16) — Smoke Tests
// ============================================================================

#[tokio::test]
async fn test_cognition_invention_tools() {
    let (_dir, handler, ids) = setup_with_captures(2).await;

    // Invention 13: Semantic Vision
    // vision_semantic_analyze: required ["capture_id"]
    call_tool_ok(
        &handler,
        70,
        "vision_semantic_analyze",
        json!({
            "capture_id": ids[0]
        }),
    )
    .await;

    // vision_semantic_find: required ["role"]
    call_tool_ok(
        &handler,
        71,
        "vision_semantic_find",
        json!({
            "role": "navigation"
        }),
    )
    .await;

    // vision_semantic_intent: required ["capture_id"]
    call_tool_ok(
        &handler,
        72,
        "vision_semantic_intent",
        json!({
            "capture_id": ids[0]
        }),
    )
    .await;

    // Invention 14: Visual Reasoning
    // vision_reason: required ["observation"]
    call_tool_ok(
        &handler,
        73,
        "vision_reason",
        json!({
            "observation": "The button appears disabled"
        }),
    )
    .await;

    // vision_reason_about: required ["question"]
    call_tool_ok(
        &handler,
        74,
        "vision_reason_about",
        json!({
            "question": "What is the user flow?"
        }),
    )
    .await;

    // vision_reason_diagnose: required ["symptoms"] (array of strings)
    call_tool_ok(
        &handler,
        75,
        "vision_reason_diagnose",
        json!({
            "symptoms": ["Layout is broken on mobile"]
        }),
    )
    .await;

    // Invention 15: Cross-Modal Binding
    // vision_bind_code: required ["capture_id", "code_node_id", "binding_type"]
    call_tool_ok(
        &handler,
        76,
        "vision_bind_code",
        json!({
            "capture_id": ids[0], "code_node_id": "node_login_tsx", "binding_type": "rendered_by"
        }),
    )
    .await;

    // vision_bind_memory: required ["capture_id", "memory_node_id", "binding_type"]
    call_tool_ok(
        &handler,
        77,
        "vision_bind_memory",
        json!({
            "capture_id": ids[0], "memory_node_id": "42", "binding_type": "fact_about"
        }),
    )
    .await;

    // vision_bind_identity: required ["capture_id", "receipt_id", "binding_type"]
    call_tool_ok(
        &handler,
        78,
        "vision_bind_identity",
        json!({
            "capture_id": ids[0], "receipt_id": "arec_123", "binding_type": "modified_by"
        }),
    )
    .await;

    // vision_bind_time: required ["capture_id", "entity_id", "binding_type"]
    call_tool_ok(
        &handler,
        79,
        "vision_bind_time",
        json!({
            "capture_id": ids[0], "entity_id": "deploy_v2.1", "binding_type": "has_deadline"
        }),
    )
    .await;

    // vision_traverse_binding: required ["capture_id"]
    call_tool_ok(
        &handler,
        80,
        "vision_traverse_binding",
        json!({
            "capture_id": ids[0]
        }),
    )
    .await;

    // Invention 16: Visual Gestalt
    // vision_gestalt_analyze: required ["capture_id"]
    call_tool_ok(
        &handler,
        81,
        "vision_gestalt_analyze",
        json!({
            "capture_id": ids[0]
        }),
    )
    .await;

    // vision_gestalt_harmony: required ["capture_id"]
    call_tool_ok(
        &handler,
        82,
        "vision_gestalt_harmony",
        json!({
            "capture_id": ids[0]
        }),
    )
    .await;

    // vision_gestalt_improve: required ["capture_id"]
    call_tool_ok(
        &handler,
        83,
        "vision_gestalt_improve",
        json!({
            "capture_id": ids[0]
        }),
    )
    .await;
}

// ============================================================================
// 6. Scale Tests
// ============================================================================

#[tokio::test]
async fn test_scale_100_invention_calls() {
    let (_dir, handler, ids) = setup_with_captures(5).await;

    let start = std::time::Instant::now();

    for i in 0..100 {
        let tool = match i % 5 {
            0 => "vision_ground_claim",
            1 => "vision_truth_check",
            2 => "vision_dejavu_check",
            3 => "vision_semantic_find",
            _ => "vision_regression_predict",
        };
        let args = match i % 5 {
            0 => json!({ "claim": format!("Claim number {i}") }),
            1 => json!({ "claim": format!("Truth check {i}") }),
            2 => json!({ "capture_id": ids[0] }),
            3 => json!({ "role": format!("button") }),
            _ => json!({ "change_description": format!("Change {i}") }),
        };
        call_tool_ok(&handler, 100 + i, tool, args).await;
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 15,
        "100 invention tool calls took {:?} — too slow",
        elapsed
    );
}

// ============================================================================
// 7. Edge Cases — Unicode, Long Strings
// ============================================================================

#[tokio::test]
async fn test_invention_unicode_args() {
    let (_dir, handler, _ids) = setup_with_captures(1).await;

    // vision_ground_claim: required ["claim"]
    call_tool_ok(
        &handler,
        200,
        "vision_ground_claim",
        json!({
            "claim": "ログインページにボタンがある"
        }),
    )
    .await;

    // vision_hallucination_check: required ["ai_description"]
    call_tool_ok(
        &handler,
        201,
        "vision_hallucination_check",
        json!({
            "ai_description": "Das Dashboard zeigt Grafiken mit 中文说明"
        }),
    )
    .await;

    // vision_reason: required ["observation"]
    call_tool_ok(
        &handler,
        202,
        "vision_reason",
        json!({
            "observation": "为什么布局在移动端出错？"
        }),
    )
    .await;
}

#[tokio::test]
async fn test_invention_long_strings() {
    let (_dir, handler, _ids) = setup_with_captures(1).await;

    let long = "X".repeat(10_000);

    // vision_ground_claim: required ["claim"]
    call_tool_ok(
        &handler,
        210,
        "vision_ground_claim",
        json!({
            "claim": long
        }),
    )
    .await;

    // vision_reason: required ["observation"]
    call_tool_ok(
        &handler,
        211,
        "vision_reason",
        json!({
            "observation": long
        }),
    )
    .await;
}

// ============================================================================
// 8. Unknown tool returns error
// ============================================================================

#[tokio::test]
async fn test_unknown_tool_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        mcp_request(
            300,
            "tools/call",
            json!({ "name": "vision_nonexistent_tool", "arguments": {} }),
        ),
    )
    .await;

    assert!(
        resp.get("error").is_some(),
        "Unknown tool should return error: {}",
        resp
    );
}
