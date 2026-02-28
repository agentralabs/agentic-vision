//! Phase 8: compact facade tool routing tests.

use std::sync::Arc;

use serde_json::json;
use tokio::sync::Mutex;

use agentic_vision_mcp::session::VisionSessionManager;
use agentic_vision_mcp::tools::ToolRegistry;
use agentic_vision_mcp::types::ToolContent;

fn create_test_session() -> Arc<Mutex<VisionSessionManager>> {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("compact.avis");
    let path_str = path.to_str().expect("utf8 path").to_string();
    // Keep tempdir alive for test duration.
    std::mem::forget(dir);
    Arc::new(Mutex::new(
        VisionSessionManager::open(&path_str, None).expect("open vision session"),
    ))
}

fn parse_text_json(result: &agentic_vision_mcp::types::ToolCallResult) -> serde_json::Value {
    let text = match &result.content[0] {
        ToolContent::Text { text } => text,
        _ => panic!("Expected text content"),
    };
    serde_json::from_str(text).expect("tool result should be valid JSON")
}

#[test]
fn test_compact_tool_list_has_expected_surface() {
    let tools = ToolRegistry::list_tools_compact();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(tools.len(), 9);
    assert!(names.contains(&"vision_core"));
    assert!(names.contains(&"vision_grounding"));
    assert!(names.contains(&"vision_workspace"));
    assert!(names.contains(&"vision_session"));
    assert!(names.contains(&"vision_temporal"));
    assert!(names.contains(&"vision_prediction"));
    assert!(names.contains(&"vision_cognition"));
    assert!(names.contains(&"vision_synthesis"));
    assert!(names.contains(&"vision_forensics"));
}

#[tokio::test]
async fn test_compact_core_query_routes_to_vision_query() {
    let session = create_test_session();
    let result = ToolRegistry::call(
        "vision_core",
        Some(json!({
            "operation": "query",
            "params": {
                "max_results": 1
            }
        })),
        &session,
    )
    .await
    .expect("vision_core query should route");

    let parsed = parse_text_json(&result);
    assert!(parsed["total"].is_number());
    assert!(parsed["observations"].is_array());
}

#[tokio::test]
async fn test_compact_workspace_create_routes() {
    let session = create_test_session();
    let result = ToolRegistry::call(
        "vision_workspace",
        Some(json!({
            "operation": "create",
            "params": {
                "name": "vision-compact-workspace"
            }
        })),
        &session,
    )
    .await
    .expect("vision_workspace create should route");

    let parsed = parse_text_json(&result);
    assert_eq!(parsed["name"], "vision-compact-workspace");
    assert!(parsed["workspace_id"].as_str().is_some());
}

#[tokio::test]
async fn test_compact_temporal_routes_to_invention_tools() {
    let session = create_test_session();
    let result = ToolRegistry::call(
        "vision_temporal",
        Some(json!({
            "operation": "timeline"
        })),
        &session,
    )
    .await
    .expect("vision_temporal timeline should route");

    let parsed = parse_text_json(&result);
    assert!(parsed["timeline"].is_array());
}
