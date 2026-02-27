//! Edge-case integration tests for all 51 agentic-vision invention tools.
//!
//! Covers: tool count, smoke tests (empty args), empty store, invalid args,
//! boundary values, rapid-fire concurrency, and with-captures scenarios.
//! Target: ~90 tests total.

use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use agentic_vision_mcp::protocol::ProtocolHandler;
use agentic_vision_mcp::session::VisionSessionManager;
use agentic_vision_mcp::types::*;

// ─────────────────────── helpers ───────────────────────

fn temp_session(dir: &tempfile::TempDir) -> VisionSessionManager {
    let path = dir.path().join("edge_inv.avis");
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
            "clientInfo": { "name": "edge-inv-test", "version": "1.0" }
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

/// Call a tool and assert it returns a result (not a top-level JSON-RPC error).
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

/// Call a tool and assert it returns a top-level JSON-RPC error.
async fn call_tool_err(handler: &ProtocolHandler, id: i64, name: &str, args: Value) -> Value {
    let resp = send_unwrap(
        handler,
        mcp_request(id, "tools/call", json!({ "name": name, "arguments": args })),
    )
    .await;
    // Some tools return error at top level, some return isError inside result.
    // Accept either as "error".
    let has_error = resp.get("error").is_some()
        || resp
            .pointer("/result/isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
    assert!(
        has_error,
        "Tool '{}' should fail with error, got: {}",
        name,
        resp
    );
    resp
}

/// Extract the text content from a successful tool call response.
fn extract_tool_text(resp: &Value) -> String {
    resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

/// Capture a 1x1 PNG via the protocol handler. Returns the capture_id.
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
                    "labels": ["edge-test", "tiny"],
                    "description": format!("Edge capture {id}")
                }
            }),
        ),
    )
    .await;
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    parsed["capture_id"].as_u64().unwrap()
}

/// Setup: init + capture N images. Returns (TempDir, ProtocolHandler, Vec<capture_id>).
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

/// Setup with no captures (empty store).
async fn setup_empty() -> (tempfile::TempDir, ProtocolHandler) {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;
    (dir, handler)
}

/// All 51 invention tool names.
const ALL_51: &[&str] = &[
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

// ═══════════════════════════════════════════════════════
// 1. TOOL COUNT (1 test)
// ═══════════════════════════════════════════════════════

/// Test 1: tools/list returns at least 72 tools (21 base + 51 inventions).
#[tokio::test]
async fn test_01_tool_count_at_least_72() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(&handler, mcp_request(1, "tools/list", json!({}))).await;
    let tools = resp["result"]["tools"].as_array().unwrap();
    assert!(
        tools.len() >= 72,
        "Expected >= 72 tools, found {}",
        tools.len()
    );
}

// ═══════════════════════════════════════════════════════
// 2. SMOKE TEST ALL 51 WITH EMPTY ARGS (16 tests)
//    — one loop test + individual key tool tests
// ═══════════════════════════════════════════════════════

/// Test 2: Smoke-call every invention with empty args — none should panic.
/// Tools may return errors (missing required params) but must not crash.
#[tokio::test]
async fn test_02_smoke_all_51_empty_args_no_panic() {
    let (_dir, handler) = setup_empty().await;

    for (i, tool_name) in ALL_51.iter().enumerate() {
        let resp = send_unwrap(
            &handler,
            mcp_request(
                (100 + i) as i64,
                "tools/call",
                json!({ "name": tool_name, "arguments": {} }),
            ),
        )
        .await;
        // Must have either "result" or "error" — never panic/crash
        assert!(
            resp.get("result").is_some() || resp.get("error").is_some(),
            "Tool '{}' returned neither result nor error: {}",
            tool_name,
            resp
        );
    }
}

/// Test 3: vision_ground_claim with empty args returns a response.
#[tokio::test]
async fn test_03_ground_claim_empty_args() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({ "name": "vision_ground_claim", "arguments": {} })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 4: vision_hallucination_check with empty args.
#[tokio::test]
async fn test_04_hallucination_check_empty_args() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({ "name": "vision_hallucination_check", "arguments": {} })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 5: vision_truth_refresh with empty args (no required fields).
#[tokio::test]
async fn test_05_truth_refresh_empty_args() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_truth_refresh", json!({})).await;
}

/// Test 6: vision_consolidate with empty args (no required fields).
#[tokio::test]
async fn test_06_consolidate_empty_args() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_consolidate", json!({})).await;
}

/// Test 7: vision_consolidate_preview with empty args.
#[tokio::test]
async fn test_07_consolidate_preview_empty_args() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_consolidate_preview", json!({})).await;
}

/// Test 8: vision_consolidate_policy with empty args.
#[tokio::test]
async fn test_08_consolidate_policy_empty_args() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_consolidate_policy", json!({})).await;
}

/// Test 9: vision_dejavu_patterns with empty args (no required fields).
#[tokio::test]
async fn test_09_dejavu_patterns_empty_args() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_dejavu_patterns", json!({})).await;
}

/// Test 10: vision_timeline with empty args (optional fields).
#[tokio::test]
async fn test_10_timeline_empty_args() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_timeline", json!({})).await;
}

/// Test 11: vision_reason with empty args.
#[tokio::test]
async fn test_11_reason_empty_args() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({ "name": "vision_reason", "arguments": {} })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 12: vision_semantic_find with empty args.
#[tokio::test]
async fn test_12_semantic_find_empty_args() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({ "name": "vision_semantic_find", "arguments": {} })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 13: vision_regression_predict with empty args.
#[tokio::test]
async fn test_13_regression_predict_empty_args() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({ "name": "vision_regression_predict", "arguments": {} })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 14: vision_phantom_create with empty args.
#[tokio::test]
async fn test_14_phantom_create_empty_args() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({ "name": "vision_phantom_create", "arguments": {} })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 15: vision_gestalt_analyze with empty args.
#[tokio::test]
async fn test_15_gestalt_analyze_empty_args() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({ "name": "vision_gestalt_analyze", "arguments": {} })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 16: vision_bind_code with empty args.
#[tokio::test]
async fn test_16_bind_code_empty_args() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({ "name": "vision_bind_code", "arguments": {} })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 17: vision_prophecy with empty args.
#[tokio::test]
async fn test_17_prophecy_empty_args() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({ "name": "vision_prophecy", "arguments": {} })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

// ═══════════════════════════════════════════════════════
// 3. EMPTY STORE CALLS (10 tests)
//    — tools that reference captures on an empty store
// ═══════════════════════════════════════════════════════

/// Test 18: vision_compare_contexts with empty capture list.
#[tokio::test]
async fn test_18_compare_contexts_empty_store() {
    let (_dir, handler) = setup_empty().await;
    call_tool_err(&handler, 1, "vision_compare_contexts", json!({
        "capture_ids": []
    })).await;
}

/// Test 19: vision_compare_sites referencing non-existent captures.
#[tokio::test]
async fn test_19_compare_sites_nonexistent_captures() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_compare_sites", json!({
        "capture_a": 99999, "capture_b": 99998
    })).await;
}

/// Test 20: vision_dejavu_check with non-existent capture_id.
#[tokio::test]
async fn test_20_dejavu_check_nonexistent() {
    let (_dir, handler) = setup_empty().await;
    call_tool_err(&handler, 1, "vision_dejavu_check", json!({
        "capture_id": 12345
    })).await;
}

/// Test 21: vision_attention_predict with non-existent capture.
#[tokio::test]
async fn test_21_attention_predict_nonexistent() {
    let (_dir, handler) = setup_empty().await;
    call_tool_err(&handler, 1, "vision_attention_predict", json!({
        "capture_id": 77777
    })).await;
}

/// Test 22: vision_semantic_analyze with non-existent capture.
#[tokio::test]
async fn test_22_semantic_analyze_nonexistent() {
    let (_dir, handler) = setup_empty().await;
    call_tool_err(&handler, 1, "vision_semantic_analyze", json!({
        "capture_id": 88888
    })).await;
}

/// Test 23: vision_gestalt_harmony with non-existent capture.
#[tokio::test]
async fn test_23_gestalt_harmony_nonexistent() {
    let (_dir, handler) = setup_empty().await;
    call_tool_err(&handler, 1, "vision_gestalt_harmony", json!({
        "capture_id": 33333
    })).await;
}

/// Test 24: vision_traverse_binding with non-existent capture.
#[tokio::test]
async fn test_24_traverse_binding_nonexistent() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_traverse_binding", json!({
        "capture_id": 44444
    })).await;
}

/// Test 25: vision_archaeology_dig on empty store.
#[tokio::test]
async fn test_25_archaeology_dig_empty_store() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_archaeology_dig", json!({
        "target": "nonexistent page"
    })).await;
}

/// Test 26: vision_prophecy_diff with non-existent capture.
#[tokio::test]
async fn test_26_prophecy_diff_nonexistent() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_prophecy_diff", json!({
        "capture_id": 55555, "change_description": "add footer"
    })).await;
}

/// Test 27: vision_regression_test on empty store.
#[tokio::test]
async fn test_27_regression_test_empty_store() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_regression_test", json!({
        "target": "dashboard"
    })).await;
}

// ═══════════════════════════════════════════════════════
// 4. INVALID ARGS (8 tests)
// ═══════════════════════════════════════════════════════

/// Test 28: vision_compare_contexts with string instead of array.
#[tokio::test]
async fn test_28_compare_contexts_wrong_type() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({
            "name": "vision_compare_contexts",
            "arguments": { "capture_ids": "not_an_array" }
        })),
    )
    .await;
    // Should not panic; any response is acceptable
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 29: vision_compare_sites with negative capture ids.
#[tokio::test]
async fn test_29_compare_sites_negative_ids() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({
            "name": "vision_compare_sites",
            "arguments": { "capture_a": -1, "capture_b": -2 }
        })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 30: vision_phantom_create with empty modifications array.
#[tokio::test]
async fn test_30_phantom_create_empty_modifications() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({
            "name": "vision_phantom_create",
            "arguments": {
                "base_capture": 1,
                "modifications": []
            }
        })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 31: vision_dejavu_alert with empty pattern_labels array.
#[tokio::test]
async fn test_31_dejavu_alert_empty_labels() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_dejavu_alert", json!({
        "pattern_labels": []
    })).await;
}

/// Test 32: vision_bind_code with null fields.
#[tokio::test]
async fn test_32_bind_code_null_fields() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({
            "name": "vision_bind_code",
            "arguments": {
                "capture_id": null,
                "code_node_id": null,
                "binding_type": null
            }
        })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 33: vision_reason_diagnose with wrong type for symptoms.
#[tokio::test]
async fn test_33_reason_diagnose_wrong_type() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({
            "name": "vision_reason_diagnose",
            "arguments": { "symptoms": "not_an_array" }
        })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 34: vision_compare_devices with malformed captures array.
#[tokio::test]
async fn test_34_compare_devices_malformed() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({
            "name": "vision_compare_devices",
            "arguments": {
                "captures": [{ "bad_key": 1 }]
            }
        })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

/// Test 35: vision_at_time with string instead of number.
#[tokio::test]
async fn test_35_at_time_string_target() {
    let (_dir, handler) = setup_empty().await;
    let resp = send_unwrap(
        &handler,
        mcp_request(1, "tools/call", json!({
            "name": "vision_at_time",
            "arguments": { "target_time": "not-a-number" }
        })),
    )
    .await;
    assert!(resp.get("result").is_some() || resp.get("error").is_some());
}

// ═══════════════════════════════════════════════════════
// 5. BOUNDARY VALUES (5 tests)
// ═══════════════════════════════════════════════════════

/// Test 36: vision_ground_claim with 100KB claim string.
#[tokio::test]
async fn test_36_ground_claim_100kb_string() {
    let (_dir, handler) = setup_empty().await;
    let huge = "A".repeat(100_000);
    call_tool_ok(&handler, 1, "vision_ground_claim", json!({ "claim": huge })).await;
}

/// Test 37: vision_at_time with timestamp 0 (epoch).
#[tokio::test]
async fn test_37_at_time_epoch_zero() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_at_time", json!({ "target_time": 0 })).await;
}

/// Test 38: vision_at_time with far-future timestamp.
#[tokio::test]
async fn test_38_at_time_far_future() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_at_time", json!({ "target_time": 9999999999i64 })).await;
}

/// Test 39: vision_consolidate_policy with extreme max_age_hours.
#[tokio::test]
async fn test_39_consolidate_policy_extreme_age() {
    let (_dir, handler) = setup_empty().await;
    call_tool_ok(&handler, 1, "vision_consolidate_policy", json!({
        "max_age_hours": 999_999_999
    })).await;
}

/// Test 40: vision_compare_contexts with many IDs (large array).
#[tokio::test]
async fn test_40_compare_contexts_large_id_array() {
    let (_dir, handler) = setup_empty().await;
    let ids: Vec<u64> = (1..=500).collect();
    call_tool_ok(&handler, 1, "vision_compare_contexts", json!({
        "capture_ids": ids
    })).await;
}

// ═══════════════════════════════════════════════════════
// 6. RAPID-FIRE (5 tests)
// ═══════════════════════════════════════════════════════

/// Test 41: 50 rapid vision_ground_claim calls.
#[tokio::test]
async fn test_41_rapid_ground_claim_50() {
    let (_dir, handler) = setup_empty().await;
    let start = std::time::Instant::now();
    for i in 0..50 {
        call_tool_ok(
            &handler,
            i,
            "vision_ground_claim",
            json!({ "claim": format!("Rapid claim {i}") }),
        )
        .await;
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 10,
        "50 rapid ground_claim calls took {:?} — too slow",
        elapsed
    );
}

/// Test 42: 50 rapid vision_truth_check calls.
#[tokio::test]
async fn test_42_rapid_truth_check_50() {
    let (_dir, handler) = setup_empty().await;
    let start = std::time::Instant::now();
    for i in 0..50 {
        call_tool_ok(
            &handler,
            i,
            "vision_truth_check",
            json!({ "claim": format!("Truth #{i}") }),
        )
        .await;
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 10,
        "50 rapid truth_check calls took {:?} — too slow",
        elapsed
    );
}

/// Test 43: 50 rapid vision_reason calls.
#[tokio::test]
async fn test_43_rapid_reason_50() {
    let (_dir, handler) = setup_empty().await;
    let start = std::time::Instant::now();
    for i in 0..50 {
        call_tool_ok(
            &handler,
            i,
            "vision_reason",
            json!({ "observation": format!("Observation {i}") }),
        )
        .await;
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 10,
        "50 rapid reason calls took {:?} — too slow",
        elapsed
    );
}

/// Test 44: Rapid-fire mixed tools — 60 calls across 6 tools.
#[tokio::test]
async fn test_44_rapid_mixed_tools_60() {
    let (_dir, handler) = setup_empty().await;
    let start = std::time::Instant::now();

    let tools_and_args: Vec<(&str, Value)> = vec![
        ("vision_ground_claim", json!({ "claim": "test" })),
        ("vision_truth_check", json!({ "claim": "test" })),
        ("vision_reason", json!({ "observation": "test" })),
        ("vision_semantic_find", json!({ "role": "nav" })),
        ("vision_regression_predict", json!({ "change_description": "test" })),
        ("vision_dejavu_patterns", json!({})),
    ];

    for i in 0..60 {
        let (tool, args) = &tools_and_args[i % tools_and_args.len()];
        call_tool_ok(&handler, i as i64, tool, args.clone()).await;
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 15,
        "60 mixed rapid-fire calls took {:?} — too slow",
        elapsed
    );
}

/// Test 45: Rapid-fire concurrent — spawn 10 tasks in parallel.
#[tokio::test]
async fn test_45_rapid_concurrent_10_tasks() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = Arc::new(ProtocolHandler::new(session.clone()));
    send_unwrap(&handler, init_request()).await;

    let start = std::time::Instant::now();
    let mut handles = Vec::new();

    for i in 0..10 {
        let h = handler.clone();
        handles.push(tokio::spawn(async move {
            for j in 0..5 {
                let id = (i * 10 + j) as i64;
                let resp = send_unwrap(
                    &h,
                    mcp_request(
                        id,
                        "tools/call",
                        json!({
                            "name": "vision_ground_claim",
                            "arguments": { "claim": format!("Concurrent claim {i}-{j}") }
                        }),
                    ),
                )
                .await;
                assert!(resp.get("result").is_some() || resp.get("error").is_some());
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 15,
        "10 concurrent tasks (50 total calls) took {:?} — too slow",
        elapsed
    );
}

// ═══════════════════════════════════════════════════════
// 7. WITH CAPTURES PRESENT (10 tests)
//    — capture a 1x1 PNG, then exercise tools that use capture IDs
// ═══════════════════════════════════════════════════════

/// Test 46: vision_compare_contexts with real captures.
#[tokio::test]
async fn test_46_compare_contexts_real_captures() {
    let (_dir, handler, ids) = setup_with_captures(3).await;
    let resp = call_tool_ok(&handler, 1, "vision_compare_contexts", json!({
        "capture_ids": [ids[0], ids[1], ids[2]]
    })).await;
    let text = extract_tool_text(&resp);
    assert!(!text.is_empty(), "Expected non-empty response text");
}

/// Test 47: vision_dejavu_check with real capture.
#[tokio::test]
async fn test_47_dejavu_check_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(2).await;
    let resp = call_tool_ok(&handler, 1, "vision_dejavu_check", json!({
        "capture_id": ids[0]
    })).await;
    let text = extract_tool_text(&resp);
    assert!(!text.is_empty());
}

/// Test 48: vision_attention_predict with real capture.
#[tokio::test]
async fn test_48_attention_predict_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    let resp = call_tool_ok(&handler, 1, "vision_attention_predict", json!({
        "capture_id": ids[0]
    })).await;
    let text = extract_tool_text(&resp);
    assert!(!text.is_empty());
}

/// Test 49: vision_semantic_analyze with real capture.
#[tokio::test]
async fn test_49_semantic_analyze_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    let resp = call_tool_ok(&handler, 1, "vision_semantic_analyze", json!({
        "capture_id": ids[0]
    })).await;
    let text = extract_tool_text(&resp);
    assert!(!text.is_empty());
}

/// Test 50: vision_semantic_intent with real capture.
#[tokio::test]
async fn test_50_semantic_intent_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    let resp = call_tool_ok(&handler, 1, "vision_semantic_intent", json!({
        "capture_id": ids[0]
    })).await;
    let text = extract_tool_text(&resp);
    assert!(!text.is_empty());
}

/// Test 51: vision_gestalt_analyze with real capture.
#[tokio::test]
async fn test_51_gestalt_analyze_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    let resp = call_tool_ok(&handler, 1, "vision_gestalt_analyze", json!({
        "capture_id": ids[0]
    })).await;
    let text = extract_tool_text(&resp);
    assert!(!text.is_empty());
}

/// Test 52: vision_gestalt_harmony with real capture.
#[tokio::test]
async fn test_52_gestalt_harmony_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    let resp = call_tool_ok(&handler, 1, "vision_gestalt_harmony", json!({
        "capture_id": ids[0]
    })).await;
    let text = extract_tool_text(&resp);
    assert!(!text.is_empty());
}

/// Test 53: vision_gestalt_improve with real capture.
#[tokio::test]
async fn test_53_gestalt_improve_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    let resp = call_tool_ok(&handler, 1, "vision_gestalt_improve", json!({
        "capture_id": ids[0]
    })).await;
    let text = extract_tool_text(&resp);
    assert!(!text.is_empty());
}

/// Test 54: vision_traverse_binding with real capture.
#[tokio::test]
async fn test_54_traverse_binding_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    let resp = call_tool_ok(&handler, 1, "vision_traverse_binding", json!({
        "capture_id": ids[0]
    })).await;
    let text = extract_tool_text(&resp);
    assert!(!text.is_empty());
}

/// Test 55: vision_prophecy_diff with real capture.
#[tokio::test]
async fn test_55_prophecy_diff_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    let resp = call_tool_ok(&handler, 1, "vision_prophecy_diff", json!({
        "capture_id": ids[0], "change_description": "Move button to top"
    })).await;
    let text = extract_tool_text(&resp);
    assert!(!text.is_empty());
}

// ═══════════════════════════════════════════════════════
// 8. ADDITIONAL EDGE CASES (35 more tests to reach ~90)
// ═══════════════════════════════════════════════════════

/// Test 56: vision_bind_memory with real capture.
#[tokio::test]
async fn test_56_bind_memory_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_bind_memory", json!({
        "capture_id": ids[0], "memory_node_id": "mem_999", "binding_type": "fact_about"
    })).await;
}

/// Test 57: vision_bind_identity with real capture.
#[tokio::test]
async fn test_57_bind_identity_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_bind_identity", json!({
        "capture_id": ids[0], "receipt_id": "arec_edge", "binding_type": "modified_by"
    })).await;
}

/// Test 58: vision_bind_time with real capture.
#[tokio::test]
async fn test_58_bind_time_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_bind_time", json!({
        "capture_id": ids[0], "entity_id": "release_v3", "binding_type": "has_deadline"
    })).await;
}

/// Test 59: vision_bind_code with real capture.
#[tokio::test]
async fn test_59_bind_code_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_bind_code", json!({
        "capture_id": ids[0], "code_node_id": "node_app_tsx", "binding_type": "rendered_by"
    })).await;
}

/// Test 60: vision_phantom_create with real capture.
#[tokio::test]
async fn test_60_phantom_create_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_phantom_create", json!({
        "base_capture": ids[0],
        "modifications": [
            { "mod_type": "layout", "target": "header", "modification": "move to bottom" }
        ]
    })).await;
}

/// Test 61: vision_phantom_compare with real capture.
#[tokio::test]
async fn test_61_phantom_compare_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_phantom_compare", json!({
        "real_capture": ids[0], "phantom_id": "phantom_edge"
    })).await;
}

/// Test 62: vision_phantom_ab_test with real capture.
#[tokio::test]
async fn test_62_phantom_ab_test_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_phantom_ab_test", json!({
        "base_capture": ids[0], "variant_description": "Dark mode variant"
    })).await;
}

/// Test 63: vision_attention_optimize with real capture.
#[tokio::test]
async fn test_63_attention_optimize_real_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_attention_optimize", json!({
        "capture_id": ids[0], "target_element": "sign-up button"
    })).await;
}

/// Test 64: vision_attention_compare with two real captures.
#[tokio::test]
async fn test_64_attention_compare_real_captures() {
    let (_dir, handler, ids) = setup_with_captures(2).await;
    call_tool_ok(&handler, 1, "vision_attention_compare", json!({
        "capture_a": ids[0], "capture_b": ids[1]
    })).await;
}

/// Test 65: vision_prophecy with real captures present.
#[tokio::test]
async fn test_65_prophecy_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(2).await;
    call_tool_ok(&handler, 1, "vision_prophecy", json!({
        "change_type": "layout", "target": "sidebar", "details": "collapse to icons"
    })).await;
}

/// Test 66: vision_prophecy_compare with real captures.
#[tokio::test]
async fn test_66_prophecy_compare_real_captures() {
    let (_dir, handler, ids) = setup_with_captures(2).await;
    call_tool_ok(&handler, 1, "vision_prophecy_compare", json!({
        "capture_before": ids[0], "capture_after": ids[1]
    })).await;
}

/// Test 67: vision_regression_history with captures.
#[tokio::test]
async fn test_67_regression_history_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(3).await;
    call_tool_ok(&handler, 1, "vision_regression_history", json!({
        "element": "login button"
    })).await;
}

/// Test 68: vision_compare_versions with real captures.
#[tokio::test]
async fn test_68_compare_versions_real_captures() {
    let (_dir, handler, ids) = setup_with_captures(2).await;
    call_tool_ok(&handler, 1, "vision_compare_versions", json!({
        "capture_old": ids[0], "capture_new": ids[1]
    })).await;
}

/// Test 69: vision_compare_devices with real captures.
#[tokio::test]
async fn test_69_compare_devices_real_captures() {
    let (_dir, handler, ids) = setup_with_captures(2).await;
    call_tool_ok(&handler, 1, "vision_compare_devices", json!({
        "captures": [
            { "capture_id": ids[0], "device": "Desktop Chrome" },
            { "capture_id": ids[1], "device": "iPhone Safari" }
        ]
    })).await;
}

/// Test 70: vision_hallucination_check with unicode and captures.
#[tokio::test]
async fn test_70_hallucination_check_unicode_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_hallucination_check", json!({
        "ai_description": "该页面显示带有日本语ボタン的登录表单"
    })).await;
}

/// Test 71: vision_hallucination_fix with captures.
#[tokio::test]
async fn test_71_hallucination_fix_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_hallucination_fix", json!({
        "claim": "I see a complex dashboard with 10 graphs"
    })).await;
}

/// Test 72: vision_truth_history with captures.
#[tokio::test]
async fn test_72_truth_history_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(2).await;
    call_tool_ok(&handler, 1, "vision_truth_history", json!({
        "subject": "page header color"
    })).await;
}

/// Test 73: vision_archaeology_reconstruct with captures.
#[tokio::test]
async fn test_73_archaeology_reconstruct_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(2).await;
    call_tool_ok(&handler, 1, "vision_archaeology_reconstruct", json!({
        "target": "navigation bar"
    })).await;
}

/// Test 74: vision_archaeology_report with captures.
#[tokio::test]
async fn test_74_archaeology_report_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(2).await;
    call_tool_ok(&handler, 1, "vision_archaeology_report", json!({
        "target": "footer"
    })).await;
}

/// Test 75: vision_consolidate with captures present.
#[tokio::test]
async fn test_75_consolidate_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(5).await;
    call_tool_ok(&handler, 1, "vision_consolidate", json!({})).await;
}

/// Test 76: vision_consolidate_preview with captures present.
#[tokio::test]
async fn test_76_consolidate_preview_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(5).await;
    call_tool_ok(&handler, 1, "vision_consolidate_preview", json!({})).await;
}

/// Test 77: vision_dejavu_alert with captures and labels.
#[tokio::test]
async fn test_77_dejavu_alert_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(3).await;
    call_tool_ok(&handler, 1, "vision_dejavu_alert", json!({
        "pattern_labels": ["edge-test", "tiny"]
    })).await;
}

/// Test 78: vision_dejavu_patterns with captures.
#[tokio::test]
async fn test_78_dejavu_patterns_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(3).await;
    call_tool_ok(&handler, 1, "vision_dejavu_patterns", json!({
        "min_occurrences": 1
    })).await;
}

/// Test 79: vision_reconstruct at now-ish time with captures.
#[tokio::test]
async fn test_79_reconstruct_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(2).await;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    call_tool_ok(&handler, 1, "vision_reconstruct", json!({
        "target_time": now
    })).await;
}

/// Test 80: vision_timeline with subject and captures.
#[tokio::test]
async fn test_80_timeline_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(3).await;
    call_tool_ok(&handler, 1, "vision_timeline", json!({
        "subject": "edge-test"
    })).await;
}

/// Test 81: vision_reason_about with captures.
#[tokio::test]
async fn test_81_reason_about_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_reason_about", json!({
        "question": "What are the main UI elements visible?"
    })).await;
}

/// Test 82: vision_reason_diagnose with captures.
#[tokio::test]
async fn test_82_reason_diagnose_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_reason_diagnose", json!({
        "symptoms": ["Button is too small", "Text overlaps image"]
    })).await;
}

/// Test 83: vision_verify_claim with captures.
#[tokio::test]
async fn test_83_verify_claim_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_verify_claim", json!({
        "claim": "The captured image is a 1x1 pixel PNG"
    })).await;
}

/// Test 84: vision_cite with captures.
#[tokio::test]
async fn test_84_cite_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_cite", json!({
        "element": "the pixel at position 0,0"
    })).await;
}

/// Test 85: vision_contradict with captures.
#[tokio::test]
async fn test_85_contradict_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(1).await;
    call_tool_ok(&handler, 1, "vision_contradict", json!({
        "claim": "The captured image contains multiple colors"
    })).await;
}

/// Test 86: vision_regression_predict with captures.
#[tokio::test]
async fn test_86_regression_predict_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(2).await;
    call_tool_ok(&handler, 1, "vision_regression_predict", json!({
        "change_description": "Increase all font sizes by 2px"
    })).await;
}

/// Test 87: vision_regression_test with captures.
#[tokio::test]
async fn test_87_regression_test_with_captures() {
    let (_dir, handler, _ids) = setup_with_captures(2).await;
    call_tool_ok(&handler, 1, "vision_regression_test", json!({
        "target": "form layout"
    })).await;
}

/// Test 88: Rapid-fire with captures — 30 calls across grounding tools.
#[tokio::test]
async fn test_88_rapid_fire_grounding_with_captures() {
    let (_dir, handler, ids) = setup_with_captures(2).await;
    let start = std::time::Instant::now();

    for i in 0..30 {
        match i % 6 {
            0 => {
                call_tool_ok(&handler, i, "vision_ground_claim", json!({
                    "claim": format!("Claim with captures {i}")
                })).await;
            }
            1 => {
                call_tool_ok(&handler, i, "vision_verify_claim", json!({
                    "claim": format!("Verify {i}")
                })).await;
            }
            2 => {
                call_tool_ok(&handler, i, "vision_hallucination_check", json!({
                    "ai_description": format!("AI says {i}")
                })).await;
            }
            3 => {
                call_tool_ok(&handler, i, "vision_truth_check", json!({
                    "claim": format!("Truth {i}")
                })).await;
            }
            4 => {
                call_tool_ok(&handler, i, "vision_compare_contexts", json!({
                    "capture_ids": [ids[0], ids[1]]
                })).await;
            }
            _ => {
                call_tool_ok(&handler, i, "vision_contradict", json!({
                    "claim": format!("Contradiction {i}")
                })).await;
            }
        }
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 10,
        "30 rapid-fire grounding calls took {:?} — too slow",
        elapsed
    );
}

/// Test 89: All binding tools in sequence on same capture.
#[tokio::test]
async fn test_89_all_bindings_on_single_capture() {
    let (_dir, handler, ids) = setup_with_captures(1).await;

    call_tool_ok(&handler, 1, "vision_bind_code", json!({
        "capture_id": ids[0], "code_node_id": "cmp_header", "binding_type": "rendered_by"
    })).await;

    call_tool_ok(&handler, 2, "vision_bind_memory", json!({
        "capture_id": ids[0], "memory_node_id": "mem_100", "binding_type": "fact_about"
    })).await;

    call_tool_ok(&handler, 3, "vision_bind_identity", json!({
        "capture_id": ids[0], "receipt_id": "arec_500", "binding_type": "modified_by"
    })).await;

    call_tool_ok(&handler, 4, "vision_bind_time", json!({
        "capture_id": ids[0], "entity_id": "sprint_42", "binding_type": "has_deadline"
    })).await;

    // Now traverse and verify we get something
    let resp = call_tool_ok(&handler, 5, "vision_traverse_binding", json!({
        "capture_id": ids[0]
    })).await;
    let text = extract_tool_text(&resp);
    assert!(!text.is_empty(), "Traverse should return bindings");
}

/// Test 90: Full gestalt pipeline on a single capture.
#[tokio::test]
async fn test_90_full_gestalt_pipeline() {
    let (_dir, handler, ids) = setup_with_captures(1).await;

    let analyze_resp = call_tool_ok(&handler, 1, "vision_gestalt_analyze", json!({
        "capture_id": ids[0]
    })).await;
    assert!(!extract_tool_text(&analyze_resp).is_empty());

    let harmony_resp = call_tool_ok(&handler, 2, "vision_gestalt_harmony", json!({
        "capture_id": ids[0]
    })).await;
    assert!(!extract_tool_text(&harmony_resp).is_empty());

    let improve_resp = call_tool_ok(&handler, 3, "vision_gestalt_improve", json!({
        "capture_id": ids[0]
    })).await;
    assert!(!extract_tool_text(&improve_resp).is_empty());
}
