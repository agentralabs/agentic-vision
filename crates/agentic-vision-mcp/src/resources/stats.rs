//! Resource: avis://stats and avis://recent

use std::sync::Arc;
use tokio::sync::Mutex;

use serde_json::json;

use crate::session::VisionSessionManager;
use crate::types::{McpResult, ReadResourceResult, ResourceContent};

pub async fn read_stats(
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ReadResourceResult> {
    let session = session.lock().await;
    let store = session.store();
    let storage_budget = session.storage_budget_status();

    let content = json!({
        "total_captures": store.count(),
        "embedding_dim": store.embedding_dim,
        "session_count": store.session_count,
        "next_id": store.next_id,
        "created_at": store.created_at,
        "updated_at": store.updated_at,
        "file_path": session.file_path().display().to_string(),
        "storage_budget": storage_budget,
    });

    Ok(ReadResourceResult {
        contents: vec![ResourceContent {
            uri: "avis://stats".to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(serde_json::to_string_pretty(&content).unwrap_or_default()),
            blob: None,
        }],
    })
}

pub async fn read_recent(
    session: &Arc<Mutex<VisionSessionManager>>,
) -> McpResult<ReadResourceResult> {
    let session = session.lock().await;
    let recent = session.store().recent(20);

    let obs_list: Vec<_> = recent
        .iter()
        .map(|o| {
            json!({
                "id": o.id,
                "timestamp": o.timestamp,
                "session_id": o.session_id,
                "dimensions": {
                    "width": o.metadata.original_width,
                    "height": o.metadata.original_height,
                },
                "labels": o.metadata.labels,
                "description": o.metadata.description,
                "memory_link": o.memory_link,
            })
        })
        .collect();

    let content = json!({
        "count": obs_list.len(),
        "captures": obs_list,
    });

    Ok(ReadResourceResult {
        contents: vec![ResourceContent {
            uri: "avis://recent".to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(serde_json::to_string_pretty(&content).unwrap_or_default()),
            blob: None,
        }],
    })
}
