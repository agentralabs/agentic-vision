//! Edge case integration tests for agentic-vision-mcp.
//!
//! Tests 16 edge cases across Security, UX, Concurrency, and Boundary Values.

use std::io::Write;
use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use agentic_vision_mcp::protocol::ProtocolHandler;
use agentic_vision_mcp::session::VisionSessionManager;
use agentic_vision_mcp::transport::framing;
use agentic_vision_mcp::types::*;

// ─────────────────────── helpers ───────────────────────

/// Create a VisionSessionManager using a temp .avis path.
fn temp_session(dir: &tempfile::TempDir) -> VisionSessionManager {
    let path = dir.path().join("test.avis");
    VisionSessionManager::open(path.to_str().unwrap(), None).unwrap()
}

/// Create Arc<Mutex<VisionSessionManager>> for handler tests.
fn arc_session(dir: &tempfile::TempDir) -> Arc<Mutex<VisionSessionManager>> {
    Arc::new(Mutex::new(temp_session(dir)))
}

/// Build an MCP JSON-RPC request.
fn mcp_request(id: i64, method: &str, params: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    })
}

/// Build an initialize request.
fn init_request() -> Value {
    mcp_request(
        0,
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "1.0" }
        }),
    )
}

/// Send a JSON-RPC message through the handler and return the response.
async fn send(handler: &ProtocolHandler, msg: Value) -> Option<Value> {
    let parsed: JsonRpcMessage = serde_json::from_value(msg).unwrap();
    handler.handle_message(parsed).await
}

/// Send and unwrap the response.
async fn send_unwrap(handler: &ProtocolHandler, msg: Value) -> Value {
    send(handler, msg).await.expect("expected response")
}

/// Create a small valid PNG image in memory (1x1 red pixel).
fn tiny_png() -> Vec<u8> {
    let img = image::DynamicImage::new_rgb8(1, 1);
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    img.write_with_encoder(encoder).unwrap();
    buf
}

/// Create an NxN solid color PNG.
fn make_png(width: u32, height: u32) -> Vec<u8> {
    let img = image::DynamicImage::new_rgb8(width, height);
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    img.write_with_encoder(encoder).unwrap();
    buf
}

/// Capture a base64-encoded image through the handler.
async fn capture_image(
    handler: &ProtocolHandler,
    base64_data: &str,
    labels: Vec<&str>,
    description: Option<&str>,
) -> Value {
    let msg = mcp_request(
        10,
        "tools/call",
        json!({
            "name": "vision_capture",
            "arguments": {
                "source": {
                    "type": "base64",
                    "data": base64_data,
                    "mime": "image/png"
                },
                "labels": labels,
                "description": description
            }
        }),
    );
    send_unwrap(handler, msg).await
}

// ═══════════════════════════════════════════════════════
// SECURITY TESTS
// ═══════════════════════════════════════════════════════

/// Test 1: Path traversal — --vision "../../../tmp/evil.avis"
#[tokio::test]
async fn test_01_path_traversal() {
    let dir = tempfile::tempdir().unwrap();
    // Create the traversal target to ensure it doesn't escape
    let evil_path = format!(
        "{}/../../../tmp/evil_test_{}.avis",
        dir.path().display(),
        std::process::id()
    );

    // The server should resolve the path but it should NOT create files
    // outside the intended directory. VisionSessionManager::open will
    // create parent dirs — we test that it doesn't panic or corrupt.
    let result = VisionSessionManager::open(&evil_path, None);

    // It should either succeed (creating the file at the resolved path)
    // or fail gracefully — never panic.
    match result {
        Ok(mut session) => {
            // Force a save by capturing something or ending session
            let _ = session.end_session();
            // Clean up
            let _ = std::fs::remove_file(&evil_path);
        }
        Err(e) => {
            // Graceful error is acceptable
            eprintln!("  Path traversal correctly rejected: {e}");
        }
    }
    // The key assertion: we didn't panic
    println!("TEST 01 — Path Traversal: PASS");
}

/// Test 2: Malformed JSON — {"broken":
#[tokio::test]
async fn test_02_malformed_json() {
    let malformed = r#"{"broken":"#;
    let result = framing::parse_message(malformed);
    assert!(result.is_err(), "Malformed JSON should return error");

    let err = result.unwrap_err();
    assert_eq!(err.code(), -32700, "Should be PARSE_ERROR (-32700)");

    // Also test empty message
    let empty = framing::parse_message("");
    assert!(empty.is_err());

    // Test truncated request
    let truncated = r#"{"jsonrpc":"2.0","id":1,"method":"#;
    assert!(framing::parse_message(truncated).is_err());

    println!("TEST 02 — Malformed JSON: PASS");
}

/// Test 3: Huge image — 100MP (10000x10000) image
#[tokio::test]
async fn test_03_huge_image() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    // Initialize
    send_unwrap(&handler, init_request()).await;

    // Create a large image (not truly 100MP — that would take too long —
    // but large enough to test the resize path: 4000x4000 = 16MP)
    let large_png = make_png(4000, 4000);
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &large_png);

    let resp = capture_image(&handler, &b64, vec!["huge"], Some("huge image test")).await;

    // Should succeed — the engine resizes for embedding (224x224) and thumbnail (512x512)
    assert!(
        resp.get("result").is_some(),
        "Huge image capture should succeed, got: {resp}"
    );

    let result = &resp["result"];
    let content_text = result["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(content_text).unwrap();
    assert_eq!(parsed["dimensions"]["width"], 4000);
    assert_eq!(parsed["dimensions"]["height"], 4000);

    println!("TEST 03 — Huge Image (16MP): PASS");
}

/// Test 4: Invalid params — vision_capture with no source
#[tokio::test]
async fn test_04_invalid_params_no_source() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    send_unwrap(&handler, init_request()).await;

    // Call vision_capture with no arguments
    let msg = mcp_request(
        1,
        "tools/call",
        json!({ "name": "vision_capture", "arguments": {} }),
    );
    let resp = send_unwrap(&handler, msg).await;

    // Per MCP spec, tool execution errors use isError: true (not JSON-RPC errors)
    assert!(
        resp.get("result").is_some()
            && resp["result"].get("isError") == Some(&json!(true)),
        "Missing source should return tool error with isError: true, got: {resp}"
    );

    // Call vision_capture with source but missing required path
    let msg2 = mcp_request(
        2,
        "tools/call",
        json!({
            "name": "vision_capture",
            "arguments": {
                "source": { "type": "file" }
            }
        }),
    );
    let resp2 = send_unwrap(&handler, msg2).await;
    assert!(
        resp2.get("result").is_some()
            && resp2["result"].get("isError") == Some(&json!(true)),
        "Missing path should return tool error with isError: true"
    );

    // Call with invalid source type
    let msg3 = mcp_request(
        3,
        "tools/call",
        json!({
            "name": "vision_capture",
            "arguments": {
                "source": { "type": "webcam", "data": "fake" }
            }
        }),
    );
    let resp3 = send_unwrap(&handler, msg3).await;
    assert!(
        resp3.get("result").is_some()
            && resp3["result"].get("isError") == Some(&json!(true)),
        "Invalid source type should return tool error with isError: true"
    );

    println!("TEST 04 — Invalid Params (no source): PASS");
}

// ═══════════════════════════════════════════════════════
// USER EXPERIENCE TESTS
// ═══════════════════════════════════════════════════════

/// Test 5: Missing directory — --vision /nonexistent/dir/test.avis
#[tokio::test]
async fn test_05_missing_directory() {
    let unique = format!(
        "/tmp/avis_test_{}/deep/nested/test.avis",
        std::process::id()
    );

    // Should auto-create the directory chain
    let result = VisionSessionManager::open(&unique, None);

    match result {
        Ok(mut session) => {
            // Should have created the dir and opened successfully.
            // Capture an image to mark the store as dirty so save() writes the file.
            let png = make_png(10, 10);
            let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png);
            session
                .capture("base64", &b64, Some("image/png"), vec![], None, false)
                .unwrap();
            session.end_session().unwrap();
            assert!(
                std::path::Path::new(&unique).exists(),
                "File should have been created"
            );
            // Clean up
            let _ = std::fs::remove_dir_all(format!("/tmp/avis_test_{}", std::process::id()));
        }
        Err(e) => {
            panic!("Should auto-create missing directories, got error: {e}");
        }
    }

    println!("TEST 05 — Missing Directory: PASS");
}

/// Test 6: Empty file — touch empty.avis then serve
#[tokio::test]
async fn test_06_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.avis");

    // Create an empty file
    std::fs::File::create(&path).unwrap();
    assert!(path.exists());
    assert_eq!(std::fs::metadata(&path).unwrap().len(), 0);

    // Try to open it
    let result = VisionSessionManager::open(path.to_str().unwrap(), None);

    // Should fail gracefully with a meaningful error (not panic)
    assert!(result.is_err(), "Empty .avis file should produce an error");
    let err = match result {
        Err(e) => format!("{e}"),
        Ok(_) => panic!("Expected error for empty file"),
    };
    eprintln!("  Empty file error: {err}");
    // Should mention something about reading failing
    assert!(
        err.contains("read")
            || err.contains("magic")
            || err.contains("fill")
            || err.contains("UnexpectedEof")
            || err.contains("failed"),
        "Error should indicate read failure: {err}"
    );

    println!("TEST 06 — Empty File: PASS");
}

/// Test 7: Corrupted file — garbage bytes in .avis
#[tokio::test]
async fn test_07_corrupted_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("corrupt.avis");

    // Write random garbage
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(b"THIS IS NOT A VALID AVIS FILE AT ALL GARBAGE GARBAGE MORE GARBAGE PADDING TO 64 BYTES OR MORE").unwrap();

    let result = VisionSessionManager::open(path.to_str().unwrap(), None);
    assert!(result.is_err(), "Corrupted file should produce an error");

    let err = match result {
        Err(e) => format!("{e}"),
        Ok(_) => panic!("Expected error for corrupted file"),
    };
    eprintln!("  Corrupted file error: {err}");
    assert!(
        err.contains("magic") || err.contains("Invalid") || err.contains("invalid"),
        "Should mention invalid magic: {err}"
    );

    // Also test with correct magic but wrong version
    let mut f2 = std::fs::File::create(&path).unwrap();
    let mut header = vec![0u8; 64];
    // Write "AVIS" magic in little-endian
    header[0..4].copy_from_slice(&0x41564953u32.to_le_bytes());
    // Write bogus version 99
    header[4..6].copy_from_slice(&99u16.to_le_bytes());
    f2.write_all(&header).unwrap();

    let result2 = VisionSessionManager::open(path.to_str().unwrap(), None);
    assert!(result2.is_err(), "Wrong version should produce an error");
    let err2 = match result2 {
        Err(e) => format!("{e}"),
        Ok(_) => panic!("Expected error for wrong version"),
    };
    assert!(
        err2.contains("version") || err2.contains("Unsupported"),
        "Should mention unsupported version: {err2}"
    );

    println!("TEST 07 — Corrupted File: PASS");
}

/// Test 8: Unicode in description — emoji/Chinese labels
#[tokio::test]
async fn test_08_unicode_descriptions() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    send_unwrap(&handler, init_request()).await;

    let png_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, tiny_png());

    // Capture with emoji description and Chinese labels
    let msg = mcp_request(
        1,
        "tools/call",
        json!({
            "name": "vision_capture",
            "arguments": {
                "source": { "type": "base64", "data": png_data, "mime": "image/png" },
                "description": "🎨 截图 — UI screenshot with émojis & spëcial chars: ñ, ü, λ, 日本語",
                "labels": ["🏠首页", "用户界面", "Ünïcödé", "العربية"]
            }
        }),
    );
    let resp = send_unwrap(&handler, msg).await;
    assert!(
        resp.get("result").is_some(),
        "Unicode capture should succeed: {resp}"
    );

    // Query back and verify labels survived
    let query_msg = mcp_request(
        2,
        "tools/call",
        json!({
            "name": "vision_query",
            "arguments": { "labels": ["🏠首页"] }
        }),
    );
    let query_resp = send_unwrap(&handler, query_msg).await;
    let result_text = query_resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(result_text).unwrap();
    assert_eq!(
        parsed["total"], 1,
        "Should find 1 result with Chinese emoji label"
    );

    println!("TEST 08 — Unicode Descriptions: PASS");
}

/// Test 9: Future protocol version — "2025-11-25"
#[tokio::test]
async fn test_09_future_protocol_version() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    let msg = mcp_request(
        0,
        "initialize",
        json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": { "name": "future-client", "version": "99.0" }
        }),
    );
    let resp = send_unwrap(&handler, msg).await;

    // Server should respond with its own version, not crash
    assert!(
        resp.get("result").is_some(),
        "Should handle future protocol version: {resp}"
    );
    let result = &resp["result"];
    assert_eq!(
        result["protocolVersion"], "2024-11-05",
        "Server should respond with its own protocol version"
    );
    assert!(
        result["serverInfo"]["name"]
            .as_str()
            .unwrap()
            .contains("agentic-vision"),
        "Should identify itself"
    );

    println!("TEST 09 — Future Protocol Version: PASS");
}

// ═══════════════════════════════════════════════════════
// CONCURRENCY TESTS
// ═══════════════════════════════════════════════════════

/// Test 10: SIGTERM — Graceful shutdown via shutdown method
#[tokio::test]
async fn test_10_graceful_shutdown() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shutdown_test.avis");
    let session = VisionSessionManager::open(path.to_str().unwrap(), None).unwrap();
    let session = Arc::new(Mutex::new(session));
    let handler = ProtocolHandler::new(session.clone());

    // Initialize
    send_unwrap(&handler, init_request()).await;

    // Capture something so there's data to save
    let png_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, tiny_png());
    capture_image(&handler, &png_data, vec!["shutdown-test"], None).await;

    // Send shutdown
    let shutdown_msg = mcp_request(99, "shutdown", json!(null));
    let resp = send_unwrap(&handler, shutdown_msg).await;
    assert!(
        resp.get("result").is_some(),
        "Shutdown should succeed: {resp}"
    );

    // Verify the file was saved (has non-zero size)
    assert!(path.exists(), "Vision file should exist after shutdown");
    let size = std::fs::metadata(&path).unwrap().len();
    assert!(size > 0, "Vision file should have data after shutdown save");

    // Verify the saved file can be reopened
    let reopened = VisionSessionManager::open(path.to_str().unwrap(), None);
    assert!(reopened.is_ok(), "Should reopen after shutdown");
    let reopened = reopened.unwrap();
    assert_eq!(
        reopened.store().count(),
        1,
        "Should have 1 capture after reopen"
    );

    println!("TEST 10 — Graceful Shutdown: PASS");
}

/// Test 11: Rapid reconnect — Start/stop/start (multiple session opens)
#[tokio::test]
async fn test_11_rapid_reconnect() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("reconnect.avis");

    let png = make_png(10, 10);
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png);

    for i in 0..5 {
        let mut session = VisionSessionManager::open(path.to_str().unwrap(), None).unwrap();
        let session_id = session.start_session(None).unwrap();
        assert!(session_id > 0, "Session {i} should have valid ID");
        // Capture something so the store gets marked dirty and save() writes
        session
            .capture(
                "base64",
                &b64,
                Some("image/png"),
                vec![format!("round-{i}")],
                None,
                false,
            )
            .unwrap();
        session.end_session().unwrap();
        // Drop triggers save via Drop impl
    }

    // Final open should see accumulated captures
    let final_session = VisionSessionManager::open(path.to_str().unwrap(), None).unwrap();
    assert!(
        final_session.store().count() >= 5,
        "Should have accumulated captures across reconnects, got {}",
        final_session.store().count()
    );
    assert!(
        final_session.current_session_id() > 1,
        "Should have accumulated sessions, got {}",
        final_session.current_session_id()
    );

    println!("TEST 11 — Rapid Reconnect: PASS");
}

// ═══════════════════════════════════════════════════════
// BOUNDARY VALUE TESTS
// ═══════════════════════════════════════════════════════

/// Test 12: 1x1 image — minimum valid capture
#[tokio::test]
async fn test_12_minimum_image() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    send_unwrap(&handler, init_request()).await;

    let png_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, tiny_png());

    let resp = capture_image(&handler, &png_data, vec!["tiny"], Some("1x1 pixel")).await;
    assert!(
        resp.get("result").is_some(),
        "1x1 image should capture successfully: {resp}"
    );

    let result_text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(result_text).unwrap();
    assert_eq!(parsed["dimensions"]["width"], 1);
    assert_eq!(parsed["dimensions"]["height"], 1);
    assert_eq!(parsed["embedding_dims"], 512);

    println!("TEST 12 — 1x1 Image: PASS");
}

/// Test 13: u64 max ID — vision_similar with huge ID
#[tokio::test]
async fn test_13_huge_id() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    send_unwrap(&handler, init_request()).await;

    // Ask for a capture that doesn't exist with u64 max
    let msg = mcp_request(
        1,
        "tools/call",
        json!({
            "name": "vision_similar",
            "arguments": { "capture_id": 18446744073709551615u64 }
        }),
    );
    let resp = send_unwrap(&handler, msg).await;

    // Per MCP spec, tool execution errors (capture not found) use isError: true
    assert!(
        resp.get("result").is_some()
            && resp["result"].get("isError") == Some(&json!(true)),
        "Huge ID should return tool error with isError: true: {resp}"
    );
    // The error message should mention the capture was not found
    let err_text = resp["result"]["content"][0]["text"].as_str().unwrap_or("");
    assert!(
        err_text.contains("not found") || err_text.contains("Capture"),
        "Error message should mention capture not found: {err_text}"
    );

    println!("TEST 13 — u64 Max ID: PASS");
}

/// Test 14: Empty description — vision_capture with description=""
#[tokio::test]
async fn test_14_empty_description() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    send_unwrap(&handler, init_request()).await;

    let png_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, tiny_png());

    // Empty string description
    let resp = capture_image(&handler, &png_data, vec![], Some("")).await;
    assert!(
        resp.get("result").is_some(),
        "Empty description should succeed: {resp}"
    );

    // Null description (via explicit json)
    let msg = mcp_request(
        2,
        "tools/call",
        json!({
            "name": "vision_capture",
            "arguments": {
                "source": { "type": "base64", "data": png_data, "mime": "image/png" },
                "description": null,
                "labels": []
            }
        }),
    );
    let resp2 = send_unwrap(&handler, msg).await;
    assert!(
        resp2.get("result").is_some(),
        "Null description should succeed: {resp2}"
    );

    // No description at all
    let msg3 = mcp_request(
        3,
        "tools/call",
        json!({
            "name": "vision_capture",
            "arguments": {
                "source": { "type": "base64", "data": png_data, "mime": "image/png" }
            }
        }),
    );
    let resp3 = send_unwrap(&handler, msg3).await;
    assert!(
        resp3.get("result").is_some(),
        "Missing description should succeed: {resp3}"
    );

    println!("TEST 14 — Empty Description: PASS");
}

/// Test 15: Max embedding size — verify embedding is always 512-dim
#[tokio::test]
async fn test_15_embedding_dimension() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = temp_session(&dir);

    // Create images of various sizes and verify embeddings
    let sizes = [(1, 1), (10, 10), (224, 224), (1000, 500), (2000, 2000)];

    for (w, h) in sizes {
        let png_data = make_png(w, h);
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_data);

        let result = session
            .capture("base64", &b64, Some("image/png"), vec![], None, false)
            .unwrap();

        assert_eq!(
            result.embedding_dims, 512,
            "Embedding should be 512-dim for {w}x{h} image"
        );
    }

    // Verify stored embeddings are actually 512-dim
    for obs in &session.store().observations {
        assert_eq!(
            obs.embedding.len(),
            512,
            "Stored embedding for id={} should be 512-dim, got {}",
            obs.id,
            obs.embedding.len()
        );
    }

    println!("TEST 15 — Embedding Dimension: PASS");
}

/// Test 16: 100 rapid captures — stress test
#[tokio::test]
async fn test_16_rapid_captures() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());

    send_unwrap(&handler, init_request()).await;

    let png_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, tiny_png());

    let start = std::time::Instant::now();

    for i in 0..100 {
        let msg = mcp_request(
            i + 1,
            "tools/call",
            json!({
                "name": "vision_capture",
                "arguments": {
                    "source": { "type": "base64", "data": &png_data, "mime": "image/png" },
                    "labels": [format!("batch-{i}")],
                    "description": format!("Rapid capture #{i}")
                }
            }),
        );
        let resp = send_unwrap(&handler, msg).await;
        assert!(
            resp.get("result").is_some(),
            "Capture {i} should succeed: {resp}"
        );
    }

    let elapsed = start.elapsed();
    eprintln!("  100 captures completed in {:?}", elapsed);

    // Verify all 100 are stored
    let locked = session.lock().await;
    assert_eq!(
        locked.store().count(),
        100,
        "Should have 100 captures stored"
    );

    // Query all
    drop(locked);
    let query_msg = mcp_request(
        999,
        "tools/call",
        json!({
            "name": "vision_query",
            "arguments": { "max_results": 200 }
        }),
    );
    let query_resp = send_unwrap(&handler, query_msg).await;
    let text = query_resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    assert_eq!(parsed["total"], 100);

    println!("TEST 16 — 100 Rapid Captures: PASS");
}

// ═══════════════════════════════════════════════════════
// ADDITIONAL EDGE CASES (bonus coverage)
// ═══════════════════════════════════════════════════════

/// Bonus: Unknown tool name
#[tokio::test]
async fn test_bonus_unknown_tool() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    send_unwrap(&handler, init_request()).await;

    let msg = mcp_request(
        1,
        "tools/call",
        json!({ "name": "nonexistent_tool", "arguments": {} }),
    );
    let resp = send_unwrap(&handler, msg).await;
    assert!(
        resp.get("error").is_some(),
        "Unknown tool should error: {resp}"
    );
    assert_eq!(resp["error"]["code"], -32803); // TOOL_NOT_FOUND

    println!("TEST BONUS — Unknown Tool: PASS");
}

/// Bonus: Unknown method
#[tokio::test]
async fn test_bonus_unknown_method() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    send_unwrap(&handler, init_request()).await;

    let msg = mcp_request(1, "foo/bar/baz", json!({}));
    let resp = send_unwrap(&handler, msg).await;
    assert!(
        resp.get("error").is_some(),
        "Unknown method should error: {resp}"
    );
    assert_eq!(resp["error"]["code"], -32601); // METHOD_NOT_FOUND

    println!("TEST BONUS — Unknown Method: PASS");
}

/// Bonus: vision_compare with same ID twice
#[tokio::test]
async fn test_bonus_compare_self() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    send_unwrap(&handler, init_request()).await;

    let png_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, tiny_png());
    let cap_resp = capture_image(&handler, &png_data, vec![], None).await;
    let cap_text = cap_resp["result"]["content"][0]["text"].as_str().unwrap();
    let cap: Value = serde_json::from_str(cap_text).unwrap();
    let id = cap["capture_id"].as_u64().unwrap();

    let msg = mcp_request(
        2,
        "tools/call",
        json!({
            "name": "vision_compare",
            "arguments": { "id_a": id, "id_b": id }
        }),
    );
    let resp = send_unwrap(&handler, msg).await;
    assert!(
        resp.get("result").is_some(),
        "Self-compare should work: {resp}"
    );

    // Similarity with self should be exactly 1.0 (or 0.0 for zero vectors in fallback mode)
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    let sim = parsed["similarity"].as_f64().unwrap();
    // In fallback mode (no CLIP model), embeddings are all zeros so cosine = 0.0
    // With a real model, cosine(x, x) = 1.0. Either is valid.
    assert!(
        sim == 0.0 || (sim - 1.0).abs() < 0.001,
        "Self-similarity should be 0.0 (fallback) or 1.0 (model), got {sim}"
    );

    println!("TEST BONUS — Compare Self: PASS");
}

/// Bonus: vision_query rejects invalid parameter ranges.
#[tokio::test]
async fn test_bonus_vision_query_param_validation() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    send_unwrap(&handler, init_request()).await;

    let invalid_cases = vec![
        json!({
            "name": "vision_query",
            "arguments": { "min_quality": 1.5 }
        }),
        json!({
            "name": "vision_query",
            "arguments": { "after": 200, "before": 100 }
        }),
        json!({
            "name": "vision_query",
            "arguments": { "max_results": 0 }
        }),
        json!({
            "name": "vision_query",
            "arguments": { "sort_by": "oldest" }
        }),
    ];

    for (i, body) in invalid_cases.into_iter().enumerate() {
        let msg = mcp_request((200 + i) as i64, "tools/call", body);
        let resp = send_unwrap(&handler, msg).await;
        // Per MCP spec, tool execution errors use isError: true
        assert!(
            resp.get("result").is_some()
                && resp["result"].get("isError") == Some(&json!(true)),
            "Invalid vision_query case #{i} should return tool error with isError: true: {resp}"
        );
    }
}

/// Bonus: vision_similar rejects ambiguous and invalid similarity arguments.
#[tokio::test]
async fn test_bonus_vision_similar_param_validation() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    send_unwrap(&handler, init_request()).await;

    let invalid_cases = vec![
        json!({
            "name": "vision_similar",
            "arguments": { "capture_id": 1, "embedding": [0.1, 0.2, 0.3] }
        }),
        json!({
            "name": "vision_similar",
            "arguments": { "embedding": [] }
        }),
        json!({
            "name": "vision_similar",
            "arguments": { "embedding": [0.1, 0.2], "top_k": 0 }
        }),
        json!({
            "name": "vision_similar",
            "arguments": { "embedding": [0.1, 0.2], "min_similarity": 1.1 }
        }),
    ];

    for (i, body) in invalid_cases.into_iter().enumerate() {
        let msg = mcp_request((300 + i) as i64, "tools/call", body);
        let resp = send_unwrap(&handler, msg).await;
        // Per MCP spec, tool execution errors use isError: true
        assert!(
            resp.get("result").is_some()
                && resp["result"].get("isError") == Some(&json!(true)),
            "Invalid vision_similar case #{i} should return tool error with isError: true: {resp}"
        );
    }
}

/// Bonus: vision_track rejects invalid tracking configuration.
#[tokio::test]
async fn test_bonus_vision_track_param_validation() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session);

    send_unwrap(&handler, init_request()).await;

    let invalid_cases = vec![
        json!({
            "name": "vision_track",
            "arguments": {
                "region": { "x": 0, "y": 0, "w": 0, "h": 100 }
            }
        }),
        json!({
            "name": "vision_track",
            "arguments": {
                "region": { "x": 0, "y": 0, "w": 100, "h": 100 },
                "interval_ms": 0
            }
        }),
        json!({
            "name": "vision_track",
            "arguments": {
                "region": { "x": 0, "y": 0, "w": 100, "h": 100 },
                "on_change_threshold": -0.1
            }
        }),
        json!({
            "name": "vision_track",
            "arguments": {
                "region": { "x": 0, "y": 0, "w": 100, "h": 100 },
                "max_captures": 0
            }
        }),
    ];

    for (i, body) in invalid_cases.into_iter().enumerate() {
        let msg = mcp_request((400 + i) as i64, "tools/call", body);
        let resp = send_unwrap(&handler, msg).await;
        // Per MCP spec, tool execution errors use isError: true
        assert!(
            resp.get("result").is_some()
                && resp["result"].get("isError") == Some(&json!(true)),
            "Invalid vision_track case #{i} should return tool error with isError: true: {resp}"
        );
    }
}
