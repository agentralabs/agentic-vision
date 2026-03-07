//! Integration tests for the Perception Revolution MCP tools.
//!
//! Tests: vision_grammar_learn, vision_grammar_get, vision_grammar_status,
//! vision_grammar_update, vision_grammar_pin, vision_dom_extract,
//! vision_intent_extract, vision_perception_route.
//!
//! Covers edge cases, error paths, and stress scenarios.

use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use agentic_vision_mcp::protocol::ProtocolHandler;
use agentic_vision_mcp::session::VisionSessionManager;
use agentic_vision_mcp::types::*;

// ─────────────────────── helpers ───────────────────────

fn temp_session(dir: &tempfile::TempDir) -> VisionSessionManager {
    let path = dir.path().join("perception.avis");
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
            "clientInfo": { "name": "perception-test", "version": "1.0" }
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

fn tool_call(id: i64, tool: &str, args: Value) -> Value {
    mcp_request(id, "tools/call", json!({ "name": tool, "arguments": args }))
}

fn parse_tool_text(resp: &Value) -> Value {
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    serde_json::from_str(text).unwrap()
}

// ═══════════════════════════════════════════════════════════
// GRAMMAR LEARN
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_grammar_learn_basic() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "amazon.com",
                "content_map": {
                    "product_price": ".a-price-whole",
                    "product_title": "#productTitle"
                },
                "intent_routes": [
                    { "intent": "find_price", "content_keys": ["product_price"] }
                ]
            }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["status"], "learned");
    assert_eq!(data["domain"], "amazon.com");
    assert_eq!(data["content_map_entries"], 2);
    assert_eq!(data["intent_routes"], 1);
}

#[tokio::test]
async fn test_grammar_learn_empty_domain() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_grammar_learn", json!({ "domain": "" })),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["status"], "learned");
}

#[tokio::test]
async fn test_grammar_learn_update_existing() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    // Learn initial
    send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "test.com",
                "content_map": { "title": "h1" }
            }),
        ),
    )
    .await;

    // Update with more fields
    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_grammar_learn",
            json!({
                "domain": "test.com",
                "content_map": { "price": ".price", "rating": ".stars" }
            }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["status"], "updated");
    assert_eq!(data["content_map_entries"], 3); // title + price + rating
}

#[tokio::test]
async fn test_grammar_learn_case_insensitive_domain() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "AMAZON.COM",
                "content_map": { "title": "h1" }
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(2, "vision_grammar_get", json!({ "domain": "amazon.com" })),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["found"], true);
}

#[tokio::test]
async fn test_grammar_learn_with_interaction_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "twitter.com",
                "content_map": {
                    "tweet_compose": "[data-testid='tweetTextarea_0']",
                    "tweet_submit": "[data-testid='tweetButtonInline']"
                },
                "interaction_patterns": [
                    {
                        "name": "post_tweet",
                        "steps": {
                            "input": "[data-testid='tweetTextarea_0']",
                            "submit": "[data-testid='tweetButtonInline']"
                        },
                        "success_indicator": "[data-testid='toast']"
                    }
                ],
                "state_indicators": [
                    { "state_name": "loading", "selector": "[data-testid='cellInnerDiv']" }
                ],
                "navigation_type": "spa"
            }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["status"], "learned");
    assert_eq!(data["content_map_entries"], 2);
}

#[tokio::test]
async fn test_grammar_learn_missing_domain_fails() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(&handler, tool_call(1, "vision_grammar_learn", json!({}))).await;

    // Should be an error response
    assert!(resp["error"].is_object() || resp["result"]["isError"] == true);
}

// ═══════════════════════════════════════════════════════════
// GRAMMAR GET
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_grammar_get_found() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "github.com",
                "content_map": { "repo_files": "[role=gridcell]" }
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(2, "vision_grammar_get", json!({ "domain": "github.com" })),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["found"], true);
    assert!(data["grammar"]["content_map"]["repo_files"].is_object());
}

#[tokio::test]
async fn test_grammar_get_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_grammar_get", json!({ "domain": "unknown.com" })),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["found"], false);
}

// ═══════════════════════════════════════════════════════════
// GRAMMAR STATUS
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_grammar_status_single() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "test.com",
                "content_map": { "a": ".a", "b": ".b" }
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(2, "vision_grammar_status", json!({ "domain": "test.com" })),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["domain"], "test.com");
    assert_eq!(data["status"], "learning");
    assert_eq!(data["content_map_entries"], 2);
}

#[tokio::test]
async fn test_grammar_status_all() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    for domain in ["a.com", "b.com", "c.com"] {
        send_unwrap(
            &handler,
            tool_call(
                1,
                "vision_grammar_learn",
                json!({
                    "domain": domain,
                    "content_map": { "field": ".sel" }
                }),
            ),
        )
        .await;
    }

    let resp = send_unwrap(&handler, tool_call(2, "vision_grammar_status", json!({}))).await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["grammar_count"], 3);
    assert_eq!(data["grammars"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_grammar_status_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_grammar_status", json!({ "domain": "nope.com" })),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["found"], false);
}

// ═══════════════════════════════════════════════════════════
// GRAMMAR UPDATE
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_grammar_update_content() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "test.com",
                "content_map": { "price": ".old-price" }
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_grammar_update",
            json!({
                "domain": "test.com",
                "content_updates": { "price": ".new-price" },
                "mark_verified": true,
                "structural_hash": "blake3:abc123"
            }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["status"], "updated");
    assert!(data["updates_applied"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_grammar_update_nonexistent_fails() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_grammar_update", json!({ "domain": "nope.com" })),
    )
    .await;

    // Error can come through as JSON-RPC error or as isError tool result
    let is_error = resp["error"].is_object() || resp["result"]["isError"] == true;
    assert!(is_error, "Expected error response, got: {resp}");
}

// ═══════════════════════════════════════════════════════════
// GRAMMAR PIN
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_grammar_pin_and_unpin() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({ "domain": "important.com" }),
        ),
    )
    .await;

    // Pin
    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_grammar_pin",
            json!({ "domain": "important.com" }),
        ),
    )
    .await;
    let data = parse_tool_text(&resp);
    assert_eq!(data["pinned"], true);

    // Unpin
    let resp = send_unwrap(
        &handler,
        tool_call(
            3,
            "vision_grammar_pin",
            json!({ "domain": "important.com", "unpin": true }),
        ),
    )
    .await;
    let data = parse_tool_text(&resp);
    assert_eq!(data["pinned"], false);
}

// ═══════════════════════════════════════════════════════════
// DOM EXTRACT
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_dom_extract_with_grammar() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "amazon.com",
                "content_map": {
                    "product_price": ".a-price-whole",
                    "product_title": "#productTitle"
                }
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_dom_extract",
            json!({
                "url": "https://amazon.com/dp/B09G9HD6PD",
                "fields": ["product_price", "product_title"]
            }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["layer"], "L0_dom_extraction");
    assert_eq!(data["grammar_used"], true);
    assert_eq!(data["tokens_used"], 0);
    assert_eq!(data["resolved_selectors"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_dom_extract_without_grammar() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_dom_extract",
            json!({
                "url": "https://unknown-site.example.com/page",
                "fields": ["price"]
            }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["grammar_used"], false);
    assert!(data["common_selectors"].is_object());
}

#[tokio::test]
async fn test_dom_extract_with_direct_selectors() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_dom_extract",
            json!({
                "url": "https://example.com",
                "selectors": ["h1", ".price", "#main-content"]
            }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["grammar_used"], false);
}

// ═══════════════════════════════════════════════════════════
// INTENT EXTRACT
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_intent_extract_with_grammar_route() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "amazon.com",
                "content_map": { "product_price": ".a-price-whole" },
                "intent_routes": [
                    { "intent": "find_price", "content_keys": ["product_price"] }
                ]
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_intent_extract",
            json!({
                "url": "https://amazon.com/dp/X",
                "intent": "find_price"
            }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["layer"], "L1_grammar_lookup");
    assert_eq!(data["route_found"], true);
    assert_eq!(data["estimated_tokens"], 0);
}

#[tokio::test]
async fn test_intent_extract_without_route() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_intent_extract",
            json!({
                "url": "https://unknown.com",
                "intent": "find_price"
            }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["route_found"], false);
    assert!(data["suggestion"].as_str().unwrap().contains("grammar"));
}

#[tokio::test]
async fn test_intent_extract_various_intents() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let intents = vec![
        ("find_price", "L0_dom_extraction"),
        ("check_stock", "L0_dom_extraction"),
        ("read_content", "L2_intent_scoped"),
        ("monitor_changes", "L3_delta_vision"),
        ("analyze_chart", "L4_scoped_screenshot"),
        ("unknown_intent", "L0_dom_extraction"),
    ];

    for (intent, expected_layer) in intents {
        let resp = send_unwrap(
            &handler,
            tool_call(
                1,
                "vision_intent_extract",
                json!({
                    "url": "https://example.com",
                    "intent": intent
                }),
            ),
        )
        .await;

        let data = parse_tool_text(&resp);
        assert_eq!(
            data["layer"], expected_layer,
            "Intent '{}' should route to {}",
            intent, expected_layer
        );
    }
}

// ═══════════════════════════════════════════════════════════
// PERCEPTION ROUTE
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_perception_route_with_grammar() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "test.com",
                "content_map": { "title": "h1" },
                "intent_routes": [
                    { "intent": "find_title", "content_keys": ["title"] }
                ]
            }),
        ),
    )
    .await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            2,
            "vision_perception_route",
            json!({
                "url": "https://test.com/page",
                "intent": "find_title"
            }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["has_grammar"], true);
    assert_eq!(data["grammar"]["has_route_for_intent"], true);
    assert_eq!(data["routing"]["estimated_tokens"], 0);
}

#[tokio::test]
async fn test_perception_route_without_grammar() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_perception_route",
            json!({
                "url": "https://unknown.com",
                "intent": "find_price"
            }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["has_grammar"], false);
    assert!(data["routing"]["recommendation"]
        .as_str()
        .unwrap()
        .contains("vision_dom_extract"));
}

// ═══════════════════════════════════════════════════════════
// PERSISTENCE: GRAMMAR SURVIVES SAVE/LOAD
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_grammar_persists_across_sessions() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("persist.avis");
    let path_str = path.to_str().unwrap();

    // Session 1: learn grammar
    {
        let session = Arc::new(Mutex::new(
            VisionSessionManager::open(path_str, None).unwrap(),
        ));
        let handler = ProtocolHandler::new(session.clone());
        send_unwrap(&handler, init_request()).await;

        send_unwrap(
            &handler,
            tool_call(
                1,
                "vision_grammar_learn",
                json!({
                    "domain": "persist.com",
                    "content_map": { "price": ".price", "title": "h1" },
                    "intent_routes": [
                        { "intent": "find_price", "content_keys": ["price"] }
                    ]
                }),
            ),
        )
        .await;

        // Force save
        session.lock().await.save().unwrap();
    }

    // Session 2: verify grammar survived
    {
        let session = Arc::new(Mutex::new(
            VisionSessionManager::open(path_str, None).unwrap(),
        ));
        let handler = ProtocolHandler::new(session.clone());
        send_unwrap(&handler, init_request()).await;

        let resp = send_unwrap(
            &handler,
            tool_call(1, "vision_grammar_get", json!({ "domain": "persist.com" })),
        )
        .await;

        let data = parse_tool_text(&resp);
        assert_eq!(data["found"], true);
        assert!(data["grammar"]["content_map"]["price"].is_object());
        assert!(data["grammar"]["content_map"]["title"].is_object());

        // Intent route should also survive
        let resp = send_unwrap(
            &handler,
            tool_call(
                2,
                "vision_intent_extract",
                json!({
                    "url": "https://persist.com/page",
                    "intent": "find_price"
                }),
            ),
        )
        .await;
        let data = parse_tool_text(&resp);
        assert_eq!(data["route_found"], true);
    }
}

// ═══════════════════════════════════════════════════════════
// STRESS: RAPID GRAMMAR OPERATIONS
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_stress_learn_100_grammars() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    for i in 0..100 {
        send_unwrap(
            &handler,
            tool_call(
                i + 1,
                "vision_grammar_learn",
                json!({
                    "domain": format!("site-{i}.com"),
                    "content_map": {
                        "field_a": format!(".sel-a-{i}"),
                        "field_b": format!(".sel-b-{i}")
                    },
                    "intent_routes": [
                        { "intent": "default", "content_keys": ["field_a"] }
                    ]
                }),
            ),
        )
        .await;
    }

    let resp = send_unwrap(&handler, tool_call(200, "vision_grammar_status", json!({}))).await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["grammar_count"], 100);
}

#[tokio::test]
async fn test_stress_intent_extract_rapid() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    // Learn one grammar
    send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "fast.com",
                "content_map": { "data": ".data" },
                "intent_routes": [
                    { "intent": "get_data", "content_keys": ["data"] }
                ]
            }),
        ),
    )
    .await;

    // 200 rapid intent extractions
    for i in 0..200 {
        let resp = send_unwrap(
            &handler,
            tool_call(
                i + 10,
                "vision_intent_extract",
                json!({
                    "url": format!("https://fast.com/page/{i}"),
                    "intent": "get_data"
                }),
            ),
        )
        .await;

        let data = parse_tool_text(&resp);
        assert_eq!(data["route_found"], true, "Iteration {i} failed");
    }
}

#[tokio::test]
async fn test_stress_dom_extract_alternating_grammar_no_grammar() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    // Learn grammar for even-numbered sites
    for i in (0..50).step_by(2) {
        send_unwrap(
            &handler,
            tool_call(
                i + 1,
                "vision_grammar_learn",
                json!({
                    "domain": format!("site-{i}.com"),
                    "content_map": { "field": ".sel" }
                }),
            ),
        )
        .await;
    }

    // Extract from all 50, alternating grammar/no-grammar
    for i in 0..50 {
        let resp = send_unwrap(
            &handler,
            tool_call(
                100 + i,
                "vision_dom_extract",
                json!({
                    "url": format!("https://site-{i}.com/page"),
                    "fields": ["field"]
                }),
            ),
        )
        .await;

        let data = parse_tool_text(&resp);
        let expected_grammar = i % 2 == 0;
        assert_eq!(
            data["grammar_used"], expected_grammar,
            "Site {i} grammar_used mismatch"
        );
    }
}

#[tokio::test]
async fn test_stress_grammar_update_overwrite_cycle() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "evolving.com",
                "content_map": { "price": ".old" }
            }),
        ),
    )
    .await;

    // 50 rapid updates simulating drift corrections
    for i in 0..50 {
        send_unwrap(
            &handler,
            tool_call(
                i + 10,
                "vision_grammar_update",
                json!({
                    "domain": "evolving.com",
                    "content_updates": { "price": format!(".selector-v{i}") }
                }),
            ),
        )
        .await;
    }

    // Verify final state
    let resp = send_unwrap(
        &handler,
        tool_call(
            100,
            "vision_grammar_get",
            json!({ "domain": "evolving.com" }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    let selector = data["grammar"]["content_map"]["price"]["selector"]
        .as_str()
        .unwrap();
    assert_eq!(selector, ".selector-v49");
}

// ═══════════════════════════════════════════════════════════
// EDGE: TOOL REGISTRATION
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_perception_tools_registered() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(&handler, mcp_request(10, "tools/list", json!({}))).await;
    let tools = resp["result"]["tools"].as_array().unwrap();

    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

    let expected = [
        "vision_dom_extract",
        "vision_intent_extract",
        "vision_perception_route",
        "vision_grammar_learn",
        "vision_grammar_get",
        "vision_grammar_status",
        "vision_grammar_update",
        "vision_grammar_pin",
    ];

    for expected_tool in &expected {
        assert!(
            tool_names.contains(expected_tool),
            "Missing tool: {expected_tool}"
        );
    }
}

#[tokio::test]
async fn test_unknown_perception_tool_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(1, "vision_nonexistent_perception_tool", json!({})),
    )
    .await;

    assert!(resp["error"].is_object());
}

// ═══════════════════════════════════════════════════════════
// EDGE: UNICODE AND SPECIAL CHARACTERS
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_grammar_unicode_selectors() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_grammar_learn",
            json!({
                "domain": "unicode.jp",
                "content_map": {
                    "日本語フィールド": ".価格",
                    "emoji_field": ".🎉-party"
                }
            }),
        ),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert_eq!(data["content_map_entries"], 2);

    // Retrieve and verify
    let resp = send_unwrap(
        &handler,
        tool_call(2, "vision_grammar_get", json!({ "domain": "unicode.jp" })),
    )
    .await;

    let data = parse_tool_text(&resp);
    assert!(data["grammar"]["content_map"]["日本語フィールド"].is_object());
}

#[tokio::test]
async fn test_dom_extract_url_with_special_chars() {
    let dir = tempfile::tempdir().unwrap();
    let session = arc_session(&dir);
    let handler = ProtocolHandler::new(session.clone());
    send_unwrap(&handler, init_request()).await;

    let resp = send_unwrap(
        &handler,
        tool_call(
            1,
            "vision_dom_extract",
            json!({
                "url": "https://example.com/path?q=hello%20world&lang=日本語#section",
                "fields": ["title"]
            }),
        ),
    )
    .await;

    // Should not crash
    let data = parse_tool_text(&resp);
    assert!(data["layer"].is_string());
}
