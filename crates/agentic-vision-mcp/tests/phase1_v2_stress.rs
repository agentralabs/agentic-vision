//! Phase 1 V2 Stress Tests: Grounding (anti-hallucination), multi-context
//! workspaces, and integration scenarios for agentic-vision-mcp.

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
            "clientInfo": { "name": "v2-stress-test", "version": "1.0" }
        }),
    )
}

fn tool_call(id: i64, name: &str, args: Value) -> Value {
    mcp_request(
        id,
        "tools/call",
        json!({
            "name": name,
            "arguments": args
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

fn extract_tool_text(resp: &Value) -> String {
    resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

fn extract_tool_json(resp: &Value) -> Value {
    let text = extract_tool_text(resp);
    serde_json::from_str(&text).unwrap_or(json!({}))
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

/// Capture an image via the protocol with a description and labels. Returns capture_id.
async fn capture_with_desc(
    handler: &ProtocolHandler,
    id: i64,
    description: &str,
    labels: Vec<&str>,
) -> u64 {
    let b64 = tiny_png_base64();
    let label_values: Vec<Value> = labels.iter().map(|l| json!(l)).collect();
    let resp = send_unwrap(
        handler,
        mcp_request(
            id,
            "tools/call",
            json!({
                "name": "vision_capture",
                "arguments": {
                    "source": { "type": "base64", "data": b64, "mime": "image/png" },
                    "labels": label_values,
                    "description": description
                }
            }),
        ),
    )
    .await;

    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(text).unwrap();
    parsed["capture_id"].as_u64().unwrap()
}

/// Create an .avis file with observations, returning its path as a String.
/// Each entry is (description, labels).
fn create_avis_file(
    dir: &tempfile::TempDir,
    filename: &str,
    entries: &[(&str, &[&str])],
) -> String {
    let path = dir.path().join(filename);
    let mut store = agentic_vision::VisualMemoryStore::new(agentic_vision::EMBEDDING_DIM);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    for (i, (desc, labels)) in entries.iter().enumerate() {
        let obs = agentic_vision::VisualObservation {
            id: 0,
            timestamp: now + i as u64,
            session_id: 1,
            source: agentic_vision::CaptureSource::Clipboard,
            embedding: vec![0.0; agentic_vision::EMBEDDING_DIM as usize],
            thumbnail: tiny_png(),
            metadata: agentic_vision::ObservationMeta {
                width: 64,
                height: 64,
                original_width: 512,
                original_height: 512,
                labels: labels.iter().map(|s| s.to_string()).collect(),
                description: Some(desc.to_string()),
                quality_score: 0.5,
            },
            memory_link: None,
        };
        store.add(obs);
    }

    agentic_vision::AvisWriter::write_to_file(&store, &path).unwrap();
    path.to_str().unwrap().to_string()
}

// ============================================================================
// GROUNDING TESTS (1-12)
// ============================================================================

/// 1. Capture an observation with a description, then ground a claim about it.
#[tokio::test]
async fn test_grounding_verified_capture() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    capture_with_desc(
        &handler,
        1,
        "Login page with blue submit button",
        vec!["login", "button"],
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_ground",
            json!({ "claim": "login page has a blue button" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "verified");
    assert!(parsed["confidence"].as_f64().unwrap() > 0.0);
    assert!(parsed["evidence"].as_array().unwrap().len() >= 1);
}

/// 2. Capture with elements metadata, ground a claim about an element.
#[tokio::test]
async fn test_grounding_verified_element() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    capture_with_desc(
        &handler,
        1,
        "Dashboard showing admin panel with red alert banner",
        vec!["dashboard", "admin", "alert"],
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_ground",
            json!({ "claim": "red alert banner on the admin dashboard" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "verified");
    assert!(parsed["evidence"].as_array().unwrap().len() >= 1);
}

/// 3. Capture with a URL in description, ground claim mentioning URL.
#[tokio::test]
async fn test_grounding_verified_url() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    capture_with_desc(
        &handler,
        1,
        "Screenshot of example.com showing hero section",
        vec!["homepage"],
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_ground",
            json!({ "claim": "example.com hero section" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "verified");
}

/// 4. Ground a claim with no captures at all.
#[tokio::test]
async fn test_grounding_ungrounded_no_captures() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_ground",
            json!({ "claim": "there is a blue button on the page" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "ungrounded");
}

/// 5. Capture page X, make claim about page Y.
#[tokio::test]
async fn test_grounding_ungrounded_wrong_page() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    capture_with_desc(
        &handler,
        1,
        "Checkout page with payment form",
        vec!["checkout"],
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_ground",
            json!({ "claim": "settings profile avatar upload" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "ungrounded");
}

/// 6. Empty claim string.
#[tokio::test]
async fn test_grounding_empty_claim() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_ground", json!({ "claim": "" })),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "ungrounded");
    assert!(parsed["reason"].as_str().unwrap().contains("Empty"));
}

/// 7. Very long claim (1000+ chars).
#[tokio::test]
async fn test_grounding_long_claim() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    capture_with_desc(&handler, 1, "Homepage hero banner with logo", vec!["hero"]).await;

    let long_claim = format!(
        "homepage hero banner {}",
        "with additional context ".repeat(60)
    );
    assert!(long_claim.len() > 1000);

    let resp = send_unwrap(
        &handler,
        tool_call(2, "vision_ground", json!({ "claim": long_claim })),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    // Should still work (either verified or ungrounded, but no crash)
    let status = parsed["status"].as_str().unwrap();
    assert!(status == "verified" || status == "ungrounded");
}

/// 8. Captures with unicode descriptions.
#[tokio::test]
async fn test_grounding_unicode_content() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    capture_with_desc(
        &handler,
        1,
        "ページに日本語テキストが表示されている",
        vec!["日本語", "テキスト"],
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_ground",
            json!({ "claim": "日本語テキストが表示" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "verified");
}

/// 9. Claims with special characters.
#[tokio::test]
async fn test_grounding_special_chars() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    capture_with_desc(
        &handler,
        1,
        "Error message: 404 (Not Found) <div class=\"error\">",
        vec!["error", "404"],
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(2, "vision_ground", json!({ "claim": "error 404 message" })),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "verified");
}

/// 10. Claim in different case than capture description.
#[tokio::test]
async fn test_grounding_case_insensitive() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    capture_with_desc(
        &handler,
        1,
        "Navigation Bar with DARK MODE toggle",
        vec!["navbar", "darkmode"],
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_ground",
            json!({ "claim": "navigation bar with dark mode toggle" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "verified");
}

/// 11. Multiple captures that match the claim.
#[tokio::test]
async fn test_grounding_multiple_captures() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    capture_with_desc(&handler, 1, "Login form with username field", vec!["login"]).await;
    capture_with_desc(&handler, 2, "Login form with password field", vec!["login"]).await;
    capture_with_desc(
        &handler,
        3,
        "Login page overview with both fields",
        vec!["login", "form"],
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(4, "vision_ground", json!({ "claim": "login form" })),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "verified");
    assert!(
        parsed["evidence_count"].as_u64().unwrap() >= 3,
        "Should match all 3 captures"
    );
}

/// 12. Claim partially matches captures.
#[tokio::test]
async fn test_grounding_partial_match() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    capture_with_desc(
        &handler,
        1,
        "User profile page with avatar and bio section",
        vec!["profile"],
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_ground",
            json!({ "claim": "profile page avatar" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "verified");
    let confidence = parsed["confidence"].as_f64().unwrap();
    assert!(
        confidence > 0.0 && confidence <= 1.0,
        "Partial match should yield partial confidence: {confidence}"
    );
}

// ============================================================================
// WORKSPACE TESTS (13-25)
// ============================================================================

/// 13. Create a workspace and verify the id format.
#[tokio::test]
async fn test_workspace_create() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_workspace_create",
            json!({ "name": "test-workspace" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "created");
    assert!(
        parsed["workspace_id"].as_str().unwrap().starts_with("vws_"),
        "Workspace ID should start with vws_"
    );
    assert_eq!(parsed["name"].as_str().unwrap(), "test-workspace");
}

/// 14. Create 3 workspaces, verify unique IDs.
#[tokio::test]
async fn test_workspace_create_multiple() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let mut ids = Vec::new();
    for i in 0..3 {
        let resp = send_unwrap(
            &handler,
            tool_call(
                i + 1,
                "vision_workspace_create",
                json!({ "name": format!("ws-{i}") }),
            ),
        )
        .await;
        let parsed = extract_tool_json(&resp);
        ids.push(parsed["workspace_id"].as_str().unwrap().to_string());
    }

    // All IDs should be unique
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 3, "Each workspace should have a unique ID");
}

/// 15. Add an .avis context file to a workspace.
#[tokio::test]
async fn test_workspace_add_context() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let avis_dir = tempfile::tempdir().unwrap();
    let avis_path = create_avis_file(
        &avis_dir,
        "site_a.avis",
        &[("Homepage with hero section", &["homepage", "hero"])],
    );

    // Create workspace
    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_workspace_create", json!({ "name": "ctx-test" })),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Add context
    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": avis_path
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "added");
    assert!(parsed["context_id"].as_str().is_some());
}

/// 16. Add 3 contexts with different roles.
#[tokio::test]
async fn test_workspace_add_multiple_contexts() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let avis_dir = tempfile::tempdir().unwrap();
    let path_a = create_avis_file(&avis_dir, "a.avis", &[("Site A homepage", &["homepage"])]);
    let path_b = create_avis_file(
        &avis_dir,
        "b.avis",
        &[("Site B settings page", &["settings"])],
    );
    let path_c = create_avis_file(
        &avis_dir,
        "c.avis",
        &[("Site C archive page", &["archive"])],
    );

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_workspace_create", json!({ "name": "multi-ctx" })),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    let roles = ["primary", "secondary", "reference"];
    let paths = [&path_a, &path_b, &path_c];

    for (i, (path, role)) in paths.iter().zip(roles.iter()).enumerate() {
        let resp = send_unwrap(
            &handler,
            tool_call(
                (i + 2) as i64,
                "vision_workspace_add",
                json!({
                    "workspace_id": ws_id,
                    "path": path,
                    "role": role,
                    "label": format!("context-{i}")
                }),
            ),
        )
        .await;
        let parsed = extract_tool_json(&resp);
        assert_eq!(parsed["status"].as_str().unwrap(), "added");
        assert_eq!(parsed["role"].as_str().unwrap(), *role);
    }
}

/// 17. List contexts in a workspace.
#[tokio::test]
async fn test_workspace_list() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let avis_dir = tempfile::tempdir().unwrap();
    let path_a = create_avis_file(&avis_dir, "a.avis", &[("Site A", &["a"])]);
    let path_b = create_avis_file(&avis_dir, "b.avis", &[("Site B", &["b"])]);

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_workspace_create", json!({ "name": "list-test" })),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_workspace_add",
            json!({ "workspace_id": ws_id, "path": path_a }),
        ),
    )
    .await;
    send_unwrap(
        &handler,
        tool_call(
            3,
            "vision_workspace_add",
            json!({ "workspace_id": ws_id, "path": path_b }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(4, "vision_workspace_list", json!({ "workspace_id": ws_id })),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["count"].as_u64().unwrap(), 2);
    assert_eq!(parsed["contexts"].as_array().unwrap().len(), 2);
}

/// 18. Query workspace with a single context.
#[tokio::test]
async fn test_workspace_query_single() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let avis_dir = tempfile::tempdir().unwrap();
    let path = create_avis_file(
        &avis_dir,
        "query.avis",
        &[
            (
                "Login form with email and password fields",
                &["login", "form"],
            ),
            ("Dashboard with charts and graphs", &["dashboard"]),
        ],
    );

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_workspace_create",
            json!({ "name": "query-single" }),
        ),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_workspace_add",
            json!({ "workspace_id": ws_id, "path": path }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            3,
            "vision_workspace_query",
            json!({
                "workspace_id": ws_id,
                "query": "login form"
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert!(parsed["total_matches"].as_u64().unwrap() >= 1);
    let results = parsed["results"].as_array().unwrap();
    assert!(!results.is_empty());
}

/// 19. Query across 2+ contexts.
#[tokio::test]
async fn test_workspace_query_across() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let avis_dir = tempfile::tempdir().unwrap();
    let path_a = create_avis_file(
        &avis_dir,
        "site_a.avis",
        &[("Navigation bar with search button", &["nav", "search"])],
    );
    let path_b = create_avis_file(
        &avis_dir,
        "site_b.avis",
        &[("Header navigation with search input", &["nav", "search"])],
    );

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_workspace_create",
            json!({ "name": "cross-query" }),
        ),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_a,
                "label": "site-a"
            }),
        ),
    )
    .await;
    send_unwrap(
        &handler,
        tool_call(
            3,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_b,
                "label": "site-b"
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            4,
            "vision_workspace_query",
            json!({
                "workspace_id": ws_id,
                "query": "navigation search"
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert!(parsed["total_matches"].as_u64().unwrap() >= 2);
    let results = parsed["results"].as_array().unwrap();
    assert_eq!(results.len(), 2, "Should have results from both contexts");
}

/// 20. Compare: item found in multiple contexts.
#[tokio::test]
async fn test_workspace_compare_found_both() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let avis_dir = tempfile::tempdir().unwrap();
    let path_a = create_avis_file(
        &avis_dir,
        "a.avis",
        &[("Footer with copyright notice", &["footer"])],
    );
    let path_b = create_avis_file(
        &avis_dir,
        "b.avis",
        &[("Footer section with legal links", &["footer"])],
    );

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_workspace_create", json!({ "name": "cmp-both" })),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_a,
                "label": "site-a"
            }),
        ),
    )
    .await;
    send_unwrap(
        &handler,
        tool_call(
            3,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_b,
                "label": "site-b"
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            4,
            "vision_workspace_compare",
            json!({
                "workspace_id": ws_id,
                "item": "footer"
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    let found_in = parsed["found_in"].as_array().unwrap();
    assert_eq!(found_in.len(), 2, "Footer should be found in both contexts");
    assert!(parsed["missing_from"].as_array().unwrap().is_empty());
}

/// 21. Compare: item found in only one context.
#[tokio::test]
async fn test_workspace_compare_found_one() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let avis_dir = tempfile::tempdir().unwrap();
    let path_a = create_avis_file(
        &avis_dir,
        "a.avis",
        &[("Checkout page with payment form", &["checkout", "payment"])],
    );
    let path_b = create_avis_file(
        &avis_dir,
        "b.avis",
        &[("Product listing with grid layout", &["products", "grid"])],
    );

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_workspace_create", json!({ "name": "cmp-one" })),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_a,
                "label": "site-a"
            }),
        ),
    )
    .await;
    send_unwrap(
        &handler,
        tool_call(
            3,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_b,
                "label": "site-b"
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            4,
            "vision_workspace_compare",
            json!({
                "workspace_id": ws_id,
                "item": "checkout payment"
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    let found_in = parsed["found_in"].as_array().unwrap();
    let missing_from = parsed["missing_from"].as_array().unwrap();
    assert_eq!(found_in.len(), 1, "Payment should be in site-a only");
    assert_eq!(
        missing_from.len(),
        1,
        "Payment should be missing from site-b"
    );
}

/// 22. Cross-reference an item across contexts.
#[tokio::test]
async fn test_workspace_xref() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let avis_dir = tempfile::tempdir().unwrap();
    let path_a = create_avis_file(
        &avis_dir,
        "a.avis",
        &[("Sidebar with navigation links", &["sidebar", "nav"])],
    );
    let path_b = create_avis_file(
        &avis_dir,
        "b.avis",
        &[("Main content area without sidebar", &["content"])],
    );
    let path_c = create_avis_file(
        &avis_dir,
        "c.avis",
        &[(
            "Sidebar collapsed state with hamburger menu",
            &["sidebar", "menu"],
        )],
    );

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_workspace_create", json!({ "name": "xref-test" })),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_a,
                "label": "page-a"
            }),
        ),
    )
    .await;
    send_unwrap(
        &handler,
        tool_call(
            3,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_b,
                "label": "page-b"
            }),
        ),
    )
    .await;
    send_unwrap(
        &handler,
        tool_call(
            4,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_c,
                "label": "page-c"
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            5,
            "vision_workspace_xref",
            json!({
                "workspace_id": ws_id,
                "item": "sidebar"
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    let present_in = parsed["present_in"].as_array().unwrap();
    let absent_from = parsed["absent_from"].as_array().unwrap();
    assert!(
        present_in.len() >= 2,
        "Sidebar should be present in at least 2 contexts, got {}",
        present_in.len()
    );
    assert_eq!(
        present_in.len() + absent_from.len(),
        3,
        "Total should be 3 contexts"
    );
}

/// 23. Query an empty workspace (no contexts added).
#[tokio::test]
async fn test_workspace_empty_query() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_workspace_create", json!({ "name": "empty-ws" })),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_workspace_query",
            json!({
                "workspace_id": ws_id,
                "query": "anything"
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["total_matches"].as_u64().unwrap(), 0);
    assert!(parsed["results"].as_array().unwrap().is_empty());
}

/// 24. Non-existent workspace ID returns error.
#[tokio::test]
async fn test_workspace_missing_id() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_workspace_list",
            json!({ "workspace_id": "vws_nonexistent" }),
        ),
    )
    .await;

    // Should be an error response
    let is_tool_error = resp
        .get("result")
        .and_then(|r| r.get("isError"))
        .and_then(|e| e.as_bool())
        .unwrap_or(false);
    let has_error = resp.get("error").is_some();
    assert!(
        is_tool_error || has_error,
        "Non-existent workspace should return error: {resp}"
    );
}

/// 25. Add invalid (non-existent) path to workspace.
#[tokio::test]
async fn test_workspace_add_invalid_path() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_workspace_create",
            json!({ "name": "bad-path-ws" }),
        ),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": "/tmp/nonexistent_file_abc123.avis"
            }),
        ),
    )
    .await;

    let is_tool_error = resp
        .get("result")
        .and_then(|r| r.get("isError"))
        .and_then(|e| e.as_bool())
        .unwrap_or(false);
    let has_error = resp.get("error").is_some();
    assert!(
        is_tool_error || has_error,
        "Invalid path should return error: {resp}"
    );
}

// ============================================================================
// INTEGRATION TESTS (26-30)
// ============================================================================

/// 26. Grounding + workspace combined in one session.
#[tokio::test]
async fn test_ground_then_workspace() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    // Ground a claim (should be ungrounded since no captures)
    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_ground",
            json!({ "claim": "green button on page" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "ungrounded");

    // Capture and ground again
    capture_with_desc(
        &handler,
        2,
        "Page with green submit button",
        vec!["button", "green"],
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            3,
            "vision_ground",
            json!({ "claim": "green button on page" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "verified");

    // Now create a workspace and query it (independent of grounding)
    let avis_dir = tempfile::tempdir().unwrap();
    let avis_path = create_avis_file(
        &avis_dir,
        "external.avis",
        &[("External site with green button too", &["button", "green"])],
    );

    let resp = send_unwrap(
        &handler,
        tool_call(4, "vision_workspace_create", json!({ "name": "combined" })),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    send_unwrap(
        &handler,
        tool_call(
            5,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": avis_path
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            6,
            "vision_workspace_query",
            json!({
                "workspace_id": ws_id,
                "query": "green button"
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert!(parsed["total_matches"].as_u64().unwrap() >= 1);
}

/// 27. Compare captures across different sites via workspace.
#[tokio::test]
async fn test_workspace_site_comparison() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let avis_dir = tempfile::tempdir().unwrap();
    let path_prod = create_avis_file(
        &avis_dir,
        "production.avis",
        &[
            ("Header with company logo", &["header", "logo"]),
            ("Footer with privacy policy link", &["footer", "privacy"]),
            ("Login page with SSO buttons", &["login", "sso"]),
        ],
    );
    let path_staging = create_avis_file(
        &avis_dir,
        "staging.avis",
        &[
            (
                "Header with company logo and beta badge",
                &["header", "logo", "beta"],
            ),
            ("Footer with privacy policy link", &["footer", "privacy"]),
            ("Settings page with feature flags", &["settings", "flags"]),
        ],
    );

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_workspace_create",
            json!({ "name": "site-compare" }),
        ),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_prod,
                "role": "primary",
                "label": "production"
            }),
        ),
    )
    .await;
    send_unwrap(
        &handler,
        tool_call(
            3,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_staging,
                "role": "secondary",
                "label": "staging"
            }),
        ),
    )
    .await;

    // Compare something both have
    let resp = send_unwrap(
        &handler,
        tool_call(
            4,
            "vision_workspace_compare",
            json!({
                "workspace_id": ws_id,
                "item": "footer privacy"
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["found_in"].as_array().unwrap().len(), 2);

    // Compare something only prod has
    let resp = send_unwrap(
        &handler,
        tool_call(
            5,
            "vision_workspace_compare",
            json!({
                "workspace_id": ws_id,
                "item": "login sso"
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["found_in"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["missing_from"].as_array().unwrap().len(), 1);
}

/// 28. Ground various claims against 20+ captures.
#[tokio::test]
async fn test_grounding_with_many_captures() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let pages = [
        (
            "Homepage hero section with call to action",
            vec!["homepage", "hero", "cta"],
        ),
        ("About us page with team photos", vec!["about", "team"]),
        ("Contact form with email field", vec!["contact", "form"]),
        ("Blog listing with article cards", vec!["blog", "articles"]),
        (
            "Product page with price and description",
            vec!["product", "price"],
        ),
        ("Cart page with item list", vec!["cart", "items"]),
        (
            "Checkout flow step one shipping",
            vec!["checkout", "shipping"],
        ),
        (
            "Checkout flow step two payment",
            vec!["checkout", "payment"],
        ),
        (
            "Order confirmation with receipt",
            vec!["order", "confirmation"],
        ),
        (
            "User settings with notification preferences",
            vec!["settings", "notifications"],
        ),
        (
            "Dashboard analytics overview charts",
            vec!["dashboard", "analytics"],
        ),
        (
            "Search results page with filters",
            vec!["search", "filters"],
        ),
        ("Mobile nav drawer open state", vec!["mobile", "nav"]),
        ("Dark mode toggle in header", vec!["darkmode", "header"]),
        ("Error 500 internal server error page", vec!["error", "500"]),
        (
            "Loading spinner full page overlay",
            vec!["loading", "spinner"],
        ),
        (
            "Modal popup for confirmation dialog",
            vec!["modal", "dialog"],
        ),
        (
            "Toast notification success message",
            vec!["toast", "success"],
        ),
        ("Breadcrumb navigation trail", vec!["breadcrumb", "nav"]),
        ("Pagination controls at page bottom", vec!["pagination"]),
        ("File upload drag and drop zone", vec!["upload", "dragdrop"]),
    ];

    for (i, (desc, labels)) in pages.iter().enumerate() {
        capture_with_desc(&handler, (i + 1) as i64, desc, labels.clone()).await;
    }

    // Ground a claim that exists
    let resp = send_unwrap(
        &handler,
        tool_call(
            100,
            "vision_ground",
            json!({ "claim": "checkout payment flow" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "verified");

    // Ground a claim that does not exist
    let resp = send_unwrap(
        &handler,
        tool_call(
            101,
            "vision_ground",
            json!({ "claim": "calendar appointment scheduling widget" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "ungrounded");

    // Ground a claim that partially matches
    let resp = send_unwrap(
        &handler,
        tool_call(
            102,
            "vision_ground",
            json!({ "claim": "dashboard overview" }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "verified");
}

/// 29. Workspace roles and labels are properly returned in list.
#[tokio::test]
async fn test_workspace_role_labels() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let avis_dir = tempfile::tempdir().unwrap();
    let path_a = create_avis_file(
        &avis_dir,
        "primary.avis",
        &[("Primary site data", &["primary"])],
    );
    let path_b = create_avis_file(
        &avis_dir,
        "archive.avis",
        &[("Old archived data", &["archive"])],
    );

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_workspace_create", json!({ "name": "role-test" })),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_a,
                "role": "primary",
                "label": "Production Site"
            }),
        ),
    )
    .await;
    send_unwrap(
        &handler,
        tool_call(
            3,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_b,
                "role": "archive",
                "label": "Legacy Archive"
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(4, "vision_workspace_list", json!({ "workspace_id": ws_id })),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    let contexts = parsed["contexts"].as_array().unwrap();
    assert_eq!(contexts.len(), 2);

    // Verify roles and labels
    let ctx_0 = &contexts[0];
    assert_eq!(ctx_0["role"].as_str().unwrap(), "primary");
    assert_eq!(ctx_0["label"].as_str().unwrap(), "Production Site");
    assert!(ctx_0["observation_count"].as_u64().unwrap() >= 1);

    let ctx_1 = &contexts[1];
    assert_eq!(ctx_1["role"].as_str().unwrap(), "archive");
    assert_eq!(ctx_1["label"].as_str().unwrap(), "Legacy Archive");
    assert!(ctx_1["observation_count"].as_u64().unwrap() >= 1);
}

/// 30. Full workflow: create workspace, add contexts, query, compare, xref.
#[tokio::test]
async fn test_full_workflow() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    // --- Step 1: Create workspace ---
    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_workspace_create",
            json!({ "name": "full-workflow" }),
        ),
    )
    .await;
    let ws_id = extract_tool_json(&resp)["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    // --- Step 2: Create .avis files and add them ---
    let avis_dir = tempfile::tempdir().unwrap();
    let path_app = create_avis_file(
        &avis_dir,
        "app.avis",
        &[
            (
                "Main app dashboard with sidebar navigation",
                &["dashboard", "sidebar", "nav"],
            ),
            (
                "User profile page with avatar upload",
                &["profile", "avatar"],
            ),
            ("Settings page with theme selector", &["settings", "theme"]),
        ],
    );
    let path_marketing = create_avis_file(
        &avis_dir,
        "marketing.avis",
        &[
            (
                "Landing page with hero and CTA button",
                &["landing", "hero", "cta"],
            ),
            ("Pricing page with three tiers", &["pricing", "tiers"]),
        ],
    );
    let path_docs = create_avis_file(
        &avis_dir,
        "docs.avis",
        &[
            (
                "API documentation with sidebar navigation",
                &["docs", "api", "sidebar", "nav"],
            ),
            ("Getting started guide page", &["docs", "guide"]),
        ],
    );

    send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_app,
                "role": "primary",
                "label": "Application"
            }),
        ),
    )
    .await;
    send_unwrap(
        &handler,
        tool_call(
            3,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_marketing,
                "role": "secondary",
                "label": "Marketing"
            }),
        ),
    )
    .await;
    send_unwrap(
        &handler,
        tool_call(
            4,
            "vision_workspace_add",
            json!({
                "workspace_id": ws_id,
                "path": path_docs,
                "role": "reference",
                "label": "Documentation"
            }),
        ),
    )
    .await;

    // --- Step 3: List contexts ---
    let resp = send_unwrap(
        &handler,
        tool_call(5, "vision_workspace_list", json!({ "workspace_id": ws_id })),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["count"].as_u64().unwrap(), 3);

    // --- Step 4: Query across all contexts ---
    let resp = send_unwrap(
        &handler,
        tool_call(
            6,
            "vision_workspace_query",
            json!({
                "workspace_id": ws_id,
                "query": "sidebar navigation"
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert!(
        parsed["total_matches"].as_u64().unwrap() >= 2,
        "sidebar nav should match app + docs"
    );

    // --- Step 5: Compare ---
    let resp = send_unwrap(
        &handler,
        tool_call(
            7,
            "vision_workspace_compare",
            json!({
                "workspace_id": ws_id,
                "item": "sidebar navigation"
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    let found_in = parsed["found_in"].as_array().unwrap();
    let missing_from = parsed["missing_from"].as_array().unwrap();
    assert!(found_in.len() >= 2, "Sidebar nav found in app + docs");
    assert!(
        missing_from.len() >= 1,
        "Sidebar nav missing from marketing"
    );

    // --- Step 6: Cross-reference ---
    let resp = send_unwrap(
        &handler,
        tool_call(
            8,
            "vision_workspace_xref",
            json!({
                "workspace_id": ws_id,
                "item": "sidebar navigation"
            }),
        ),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    let present_in = parsed["present_in"].as_array().unwrap();
    let absent_from = parsed["absent_from"].as_array().unwrap();
    assert!(present_in.len() >= 2);
    assert!(absent_from.len() >= 1);
    assert!(parsed["coverage"].as_str().is_some());

    // --- Step 7: Also verify grounding in the same session ---
    capture_with_desc(
        &handler,
        9,
        "Final verification screenshot of dashboard sidebar",
        vec!["dashboard", "sidebar"],
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(10, "vision_ground", json!({ "claim": "dashboard sidebar" })),
    )
    .await;
    let parsed = extract_tool_json(&resp);
    assert_eq!(parsed["status"].as_str().unwrap(), "verified");
}
