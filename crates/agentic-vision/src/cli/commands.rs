//! CLI command implementations for the `avis` binary.

use std::path::Path;

use crate::storage::{AvisReader, AvisWriter};
use crate::types::{VisionResult, VisualMemoryStore};

/// Create a new empty .avis file.
pub fn cmd_create(path: &Path, dimension: u32) -> VisionResult<()> {
    let store = VisualMemoryStore::new(dimension);
    AvisWriter::write_to_file(&store, path)?;
    println!("Created {}", path.display());
    Ok(())
}

/// Display information about an .avis file.
pub fn cmd_info(path: &Path, json: bool) -> VisionResult<()> {
    let store = AvisReader::read_from_file(path)?;
    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    if json {
        let info = serde_json::json!({
            "file": path.display().to_string(),
            "observations": store.count(),
            "embedding_dim": store.embedding_dim,
            "sessions": store.session_count,
            "next_id": store.next_id,
            "created_at": store.created_at,
            "updated_at": store.updated_at,
            "file_bytes": file_size,
        });
        println!("{}", serde_json::to_string_pretty(&info).unwrap_or_default());
    } else {
        println!("File:          {}", path.display());
        println!("Observations:  {}", store.count());
        println!("Embedding dim: {}", store.embedding_dim);
        println!("Sessions:      {}", store.session_count);
        println!("Next ID:       {}", store.next_id);
        println!("Created:       {}", format_ts(store.created_at));
        println!("Updated:       {}", format_ts(store.updated_at));
        println!("File size:     {} bytes", file_size);
    }

    Ok(())
}

/// Capture an image and add it to the .avis store.
pub fn cmd_capture(
    path: &Path,
    source_path: &str,
    labels: Vec<String>,
    description: Option<String>,
    model_path: Option<&str>,
    json: bool,
) -> VisionResult<()> {
    let mut store = if path.exists() {
        AvisReader::read_from_file(path)?
    } else {
        VisualMemoryStore::new(crate::EMBEDDING_DIM as u32)
    };

    let (img, source) = crate::capture_from_file(source_path)?;
    let thumbnail = crate::generate_thumbnail(&img);
    let mut engine = crate::EmbeddingEngine::new(model_path)?;
    let embedding = engine.embed(&img)?;

    let (width, height) = (img.width(), img.height());
    let obs = crate::types::VisualObservation {
        id: 0,
        timestamp: now_secs(),
        session_id: 0,
        source,
        embedding,
        thumbnail,
        metadata: crate::types::ObservationMeta {
            width: std::cmp::min(width, 512),
            height: std::cmp::min(height, 512),
            original_width: width,
            original_height: height,
            labels,
            description,
            quality_score: compute_quality(width, height),
        },
        memory_link: None,
    };

    let id = store.add(obs);
    AvisWriter::write_to_file(&store, path)?;

    if json {
        let out = serde_json::json!({ "id": id, "file": path.display().to_string() });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        println!("Captured observation {} -> {}", id, path.display());
    }

    Ok(())
}

/// Query observations with filters.
pub fn cmd_query(
    path: &Path,
    session: Option<u32>,
    labels: Option<Vec<String>>,
    limit: usize,
    json: bool,
) -> VisionResult<()> {
    let store = AvisReader::read_from_file(path)?;
    let mut results: Vec<_> = store.observations.iter().collect();

    if let Some(sid) = session {
        results.retain(|o| o.session_id == sid);
    }
    if let Some(ref lbls) = labels {
        results.retain(|o| lbls.iter().any(|l| o.metadata.labels.contains(l)));
    }

    results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    results.truncate(limit);

    if json {
        let items: Vec<_> = results.iter().map(|o| obs_summary(o)).collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap_or_default());
    } else {
        if results.is_empty() {
            println!("No observations found.");
            return Ok(());
        }
        for o in &results {
            println!(
                "  [{:>4}]  session={}  {}x{}  labels={:?}  q={:.2}  {}",
                o.id, o.session_id, o.metadata.original_width, o.metadata.original_height,
                o.metadata.labels, o.metadata.quality_score, format_ts(o.timestamp),
            );
        }
        println!("{} observation(s)", results.len());
    }

    Ok(())
}

/// Find visually similar captures.
pub fn cmd_similar(
    path: &Path,
    capture_id: u64,
    top_k: usize,
    min_similarity: f32,
    json: bool,
) -> VisionResult<()> {
    let store = AvisReader::read_from_file(path)?;
    let obs = store
        .get(capture_id)
        .ok_or(crate::types::VisionError::CaptureNotFound(capture_id))?;

    let matches = crate::find_similar(&obs.embedding, &store.observations, top_k, min_similarity);

    if json {
        println!("{}", serde_json::to_string_pretty(&matches).unwrap_or_default());
    } else {
        if matches.is_empty() {
            println!("No similar captures found.");
            return Ok(());
        }
        for m in &matches {
            println!("  [{:>4}]  similarity={:.4}", m.id, m.similarity);
        }
    }

    Ok(())
}

/// Compare two captures.
pub fn cmd_compare(path: &Path, id_a: u64, id_b: u64, json: bool) -> VisionResult<()> {
    let store = AvisReader::read_from_file(path)?;
    let a = store.get(id_a).ok_or(crate::types::VisionError::CaptureNotFound(id_a))?;
    let b = store.get(id_b).ok_or(crate::types::VisionError::CaptureNotFound(id_b))?;

    let sim = crate::cosine_similarity(&a.embedding, &b.embedding);

    if json {
        let out = serde_json::json!({
            "id_a": id_a, "id_b": id_b, "similarity": sim, "is_same": sim > 0.95,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        println!("Compare {} vs {}", id_a, id_b);
        println!("  Embedding similarity: {:.4}", sim);
        println!("  Same image:           {}", if sim > 0.95 { "yes" } else { "no" });
    }

    Ok(())
}

/// Pixel-level diff between two captures.
pub fn cmd_diff(path: &Path, id_a: u64, id_b: u64, json: bool) -> VisionResult<()> {
    let store = AvisReader::read_from_file(path)?;
    let a = store.get(id_a).ok_or(crate::types::VisionError::CaptureNotFound(id_a))?;
    let b = store.get(id_b).ok_or(crate::types::VisionError::CaptureNotFound(id_b))?;

    let img_a = image::load_from_memory(&a.thumbnail)
        .map_err(|e| crate::types::VisionError::Image(e))?;
    let img_b = image::load_from_memory(&b.thumbnail)
        .map_err(|e| crate::types::VisionError::Image(e))?;

    let diff = crate::compute_diff(id_a, id_b, &img_a, &img_b)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&diff).unwrap_or_default());
    } else {
        println!("Diff {} vs {}", id_a, id_b);
        println!("  Similarity:      {:.4}", diff.similarity);
        println!("  Pixel diff:      {:.2}%", diff.pixel_diff_ratio * 100.0);
        println!("  Changed regions: {}", diff.changed_regions.len());
        for (i, r) in diff.changed_regions.iter().enumerate() {
            println!("    [{}] x={} y={} {}x{}", i, r.x, r.y, r.w, r.h);
        }
    }

    Ok(())
}

/// Health / quality report.
pub fn cmd_health(
    path: &Path,
    stale_hours: u64,
    low_quality_threshold: f32,
    max_examples: usize,
    json: bool,
) -> VisionResult<()> {
    let store = AvisReader::read_from_file(path)?;
    let now = now_secs();
    let stale_cutoff = now.saturating_sub(stale_hours * 3600);

    let low_quality: Vec<u64> = store.observations.iter()
        .filter(|o| o.metadata.quality_score < low_quality_threshold)
        .take(max_examples).map(|o| o.id).collect();

    let stale: Vec<u64> = store.observations.iter()
        .filter(|o| o.timestamp < stale_cutoff)
        .take(max_examples).map(|o| o.id).collect();

    let unlinked: Vec<u64> = store.observations.iter()
        .filter(|o| o.memory_link.is_none())
        .take(max_examples).map(|o| o.id).collect();

    let unlabeled: Vec<u64> = store.observations.iter()
        .filter(|o| o.metadata.labels.is_empty())
        .take(max_examples).map(|o| o.id).collect();

    let status = if low_quality.is_empty() && stale.is_empty() {
        "pass"
    } else if low_quality.len() > 5 || stale.len() > 10 {
        "fail"
    } else {
        "warn"
    };

    if json {
        let out = serde_json::json!({
            "status": status,
            "total_observations": store.count(),
            "low_quality_ids": low_quality,
            "stale_ids": stale,
            "unlinked_memory_ids": unlinked,
            "unlabeled_ids": unlabeled,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        println!("Health: {}", status.to_uppercase());
        println!("  Total observations: {}", store.count());
        println!("  Low quality (< {:.2}): {}", low_quality_threshold, low_quality.len());
        println!("  Stale (> {}h):        {}", stale_hours, stale.len());
        println!("  Unlinked memory:     {}", unlinked.len());
        println!("  Unlabeled:           {}", unlabeled.len());
    }

    Ok(())
}

/// Link a capture to a memory node.
pub fn cmd_link(path: &Path, capture_id: u64, memory_node_id: u64, json: bool) -> VisionResult<()> {
    let mut store = AvisReader::read_from_file(path)?;
    let obs = store.get_mut(capture_id)
        .ok_or(crate::types::VisionError::CaptureNotFound(capture_id))?;

    obs.memory_link = Some(memory_node_id);
    AvisWriter::write_to_file(&store, path)?;

    if json {
        let out = serde_json::json!({ "status": "linked", "capture_id": capture_id, "memory_node_id": memory_node_id });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        println!("Linked capture {} -> memory node {}", capture_id, memory_node_id);
    }

    Ok(())
}

/// Display statistics about the store.
pub fn cmd_stats(path: &Path, json: bool) -> VisionResult<()> {
    let store = AvisReader::read_from_file(path)?;
    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let total = store.count();
    let linked = store.observations.iter().filter(|o| o.memory_link.is_some()).count();
    let labeled = store.observations.iter().filter(|o| !o.metadata.labels.is_empty()).count();
    let avg_quality = if total > 0 {
        store.observations.iter().map(|o| o.metadata.quality_score).sum::<f32>() / total as f32
    } else {
        0.0
    };

    let mut sessions = std::collections::HashSet::new();
    for o in &store.observations {
        sessions.insert(o.session_id);
    }

    if json {
        let out = serde_json::json!({
            "observations": total, "sessions": sessions.len(),
            "linked_to_memory": linked, "labeled": labeled,
            "avg_quality": avg_quality, "embedding_dim": store.embedding_dim,
            "file_bytes": file_size,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        println!("Observations:     {}", total);
        println!("Sessions:         {}", sessions.len());
        println!("Linked to memory: {}", linked);
        println!("Labeled:          {}", labeled);
        println!("Avg quality:      {:.3}", avg_quality);
        println!("Embedding dim:    {}", store.embedding_dim);
        println!("File size:        {} bytes", file_size);
    }

    Ok(())
}

/// Export the store as JSON.
pub fn cmd_export(path: &Path, pretty: bool) -> VisionResult<()> {
    let store = AvisReader::read_from_file(path)?;
    let items: Vec<_> = store.observations.iter().map(|o| obs_full(o)).collect();

    let output = if pretty {
        serde_json::to_string_pretty(&items)
    } else {
        serde_json::to_string(&items)
    };

    println!("{}", output.unwrap_or_else(|_| "[]".to_string()));
    Ok(())
}

// -- Helpers --

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn format_ts(ts: u64) -> String {
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| ts.to_string())
}

fn compute_quality(w: u32, h: u32) -> f32 {
    let pixels = (w as f64) * (h as f64);
    let max_pixels = 1920.0 * 1080.0;
    (pixels / max_pixels).min(1.0) as f32
}

fn obs_summary(o: &crate::types::VisualObservation) -> serde_json::Value {
    serde_json::json!({
        "id": o.id, "session_id": o.session_id, "timestamp": o.timestamp,
        "width": o.metadata.original_width, "height": o.metadata.original_height,
        "labels": o.metadata.labels, "description": o.metadata.description,
        "quality_score": o.metadata.quality_score, "memory_link": o.memory_link,
    })
}

fn obs_full(o: &crate::types::VisualObservation) -> serde_json::Value {
    serde_json::json!({
        "id": o.id, "session_id": o.session_id, "timestamp": o.timestamp,
        "source": o.source, "embedding_len": o.embedding.len(),
        "thumbnail_bytes": o.thumbnail.len(), "metadata": o.metadata,
        "memory_link": o.memory_link,
    })
}
