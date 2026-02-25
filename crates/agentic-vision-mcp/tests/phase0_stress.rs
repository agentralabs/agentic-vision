//! Phase 0 Stress Tests: Context capture, temporal chaining, observation_log,
//! scale, edge cases, and regression for agentic-vision-mcp.

use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use agentic_vision_mcp::protocol::ProtocolHandler;
use agentic_vision_mcp::session::VisionSessionManager;
use agentic_vision_mcp::types::*;

// ─────────────────────── helpers ───────────────────────

fn temp_session(dir: &tempfile::TempDir) -> VisionSessionManager {
    let path = dir.path().join("stress.avis");
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
            "clientInfo": { "name": "stress-test", "version": "1.0" }
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

/// Capture an image via the protocol and return the capture_id.
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
                    "labels": ["test"],
                    "description": format!("Capture {id}")
                }
            }),
        ),
    )
    .await;

    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    parsed["capture_id"].as_u64().unwrap()
}

// ============================================================================
// 1. observation_log Tool — Context Capture
// ============================================================================

#[tokio::test]
async fn test_observation_log_basic() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());

    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        mcp_request(
            1,
            "tools/call",
            json!({
                "name": "observation_log",
                "arguments": {
                    "intent": "Checking deploy button color change",
                    "observation": "Button is now green",
                    "topic": "ui-testing"
                }
            }),
        ),
    )
    .await;

    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    assert!(parsed["note_id"].as_u64().is_some());
    assert!(parsed["message"].as_str().unwrap().contains("logged"));
}

#[tokio::test]
async fn test_observation_log_intent_only() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        mcp_request(
            1,
            "tools/call",
            json!({
                "name": "observation_log",
                "arguments": {
                    "intent": "Verifying layout after resize"
                }
            }),
        ),
    )
    .await;

    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    assert!(parsed["note_id"].as_u64().is_some());
}

#[tokio::test]
async fn test_observation_log_empty_intent_fails() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        mcp_request(
            1,
            "tools/call",
            json!({
                "name": "observation_log",
                "arguments": {
                    "intent": ""
                }
            }),
        ),
    )
    .await;

    // Should be an error
    let is_error = resp
        .get("result")
        .and_then(|r| r.get("isError"))
        .and_then(|e| e.as_bool())
        .unwrap_or(false);
    let has_error = resp.get("error").is_some();
    assert!(
        is_error || has_error,
        "Empty intent should be rejected: {resp}"
    );
}

#[tokio::test]
async fn test_observation_log_with_related_capture() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    // Capture an image first
    let capture_id = capture_via_protocol(&handler, 1).await;

    // Now log observation referencing that capture
    let resp = send_unwrap(
        &handler,
        mcp_request(
            2,
            "tools/call",
            json!({
                "name": "observation_log",
                "arguments": {
                    "intent": "Analyzing captured screenshot",
                    "related_capture_id": capture_id,
                    "observation": "Screenshot shows expected layout"
                }
            }),
        ),
    )
    .await;

    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    assert!(parsed["note_id"].as_u64().is_some());
}

// ============================================================================
// 2. Temporal Chain — captures linked consecutively
// ============================================================================

#[tokio::test]
async fn test_temporal_chain_consecutive_captures() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let mut capture_ids = Vec::new();

    for i in 0..5 {
        let id = capture_via_protocol(&handler, i + 1).await;
        capture_ids.push(id);
    }

    // Verify temporal chain in session
    let session_guard = session.lock().await;
    let chain = session_guard.temporal_chain();
    assert_eq!(chain.len(), 4, "5 captures should produce 4 temporal links");

    // Each link should be (prev, next)
    for i in 0..4 {
        assert_eq!(chain[i].0, capture_ids[i]);
        assert_eq!(chain[i].1, capture_ids[i + 1]);
    }
}

#[tokio::test]
async fn test_temporal_chain_resets_on_new_session() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    // Start session 1 via protocol
    send_unwrap(
        &handler,
        mcp_request(
            1,
            "tools/call",
            json!({"name": "session_start", "arguments": {}}),
        ),
    )
    .await;

    // Capture 3 images
    for i in 0..3 {
        capture_via_protocol(&handler, 10 + i).await;
    }

    {
        let sg = session.lock().await;
        assert_eq!(sg.temporal_chain().len(), 2, "3 captures → 2 links");
        assert!(sg.last_temporal_capture_id().is_some());
    }

    // End session
    send_unwrap(
        &handler,
        mcp_request(
            2,
            "tools/call",
            json!({"name": "session_end", "arguments": {}}),
        ),
    )
    .await;

    // Start a new session
    send_unwrap(
        &handler,
        mcp_request(
            3,
            "tools/call",
            json!({"name": "session_start", "arguments": {}}),
        ),
    )
    .await;

    let sg = session.lock().await;
    assert_eq!(sg.temporal_chain().len(), 0, "Chain should reset");
    assert!(
        sg.last_temporal_capture_id().is_none(),
        "Last ID should reset"
    );
}

// ============================================================================
// 3. Auto-Capture Tool Context
// ============================================================================

#[tokio::test]
async fn test_auto_capture_tool_context() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    // Call vision_query (should auto-log context)
    send_unwrap(
        &handler,
        mcp_request(
            1,
            "tools/call",
            json!({
                "name": "vision_query",
                "arguments": {}
            }),
        ),
    )
    .await;

    // Check tool call log
    let session_guard = session.lock().await;
    let log = session_guard.tool_call_log();
    assert!(
        log.iter().any(|r| r.tool_name == "vision_query"),
        "Tool call log should contain vision_query"
    );
}

#[tokio::test]
async fn test_observation_log_not_auto_logged() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    // Call observation_log (should NOT be auto-logged to avoid recursion)
    send_unwrap(
        &handler,
        mcp_request(
            1,
            "tools/call",
            json!({
                "name": "observation_log",
                "arguments": { "intent": "Test intent" }
            }),
        ),
    )
    .await;

    let session_guard = session.lock().await;
    let log = session_guard.tool_call_log();
    assert!(
        !log.iter().any(|r| r.tool_name == "observation_log"),
        "observation_log should not be auto-logged"
    );
}

// ============================================================================
// 4. Scale Tests
// ============================================================================

#[tokio::test]
async fn test_scale_20_captures_with_chain() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let start = std::time::Instant::now();

    for i in 0..20 {
        capture_via_protocol(&handler, i + 1).await;
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 60,
        "20 captures took {:?} — too slow",
        elapsed
    );

    let sg = session.lock().await;
    let chain = sg.temporal_chain();
    assert_eq!(chain.len(), 19, "20 captures should produce 19 links");
}

#[tokio::test]
async fn test_scale_500_observation_notes() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let start = std::time::Instant::now();

    for i in 0..500 {
        send_unwrap(
            &handler,
            mcp_request(
                i + 1,
                "tools/call",
                json!({
                    "name": "observation_log",
                    "arguments": {
                        "intent": format!("Observation intent {i}"),
                        "observation": format!("Observed result {i}"),
                        "topic": format!("topic-{}", i % 10)
                    }
                }),
            ),
        )
        .await;
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 30,
        "500 observation notes took {:?} — too slow",
        elapsed
    );

    let sg = session.lock().await;
    assert_eq!(sg.observation_notes().len(), 500);
}

// ============================================================================
// 5. Edge Cases — unicode, special chars
// ============================================================================

#[tokio::test]
async fn test_observation_log_unicode() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        mcp_request(
            1,
            "tools/call",
            json!({
                "name": "observation_log",
                "arguments": {
                    "intent": "检查部署按钮颜色变化",
                    "observation": "ボタンが緑色に変わりました",
                    "topic": "국제화-테스트"
                }
            }),
        ),
    )
    .await;

    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    assert!(parsed["note_id"].as_u64().is_some());
}

#[tokio::test]
async fn test_observation_log_long_intent() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let long_intent = "A".repeat(5000);
    let resp = send_unwrap(
        &handler,
        mcp_request(
            1,
            "tools/call",
            json!({
                "name": "observation_log",
                "arguments": { "intent": long_intent }
            }),
        ),
    )
    .await;

    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    assert!(parsed["note_id"].as_u64().is_some());
}

// ============================================================================
// 6. Regression — tool list includes observation_log
// ============================================================================

#[tokio::test]
async fn test_tool_list_includes_observation_log() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(&handler, mcp_request(10, "tools/list", json!({}))).await;

    let tools = resp["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(
        names.contains(&"observation_log"),
        "Tool list must include observation_log, found: {:?}",
        names
    );
}

#[tokio::test]
async fn test_tool_count_is_21() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(&handler, mcp_request(10, "tools/list", json!({}))).await;

    let tools = resp["result"]["tools"].as_array().unwrap();
    assert_eq!(
        tools.len(),
        21,
        "Should have 21 tools (11 original + observation_log + 3 grounding + 6 workspace)"
    );
}
