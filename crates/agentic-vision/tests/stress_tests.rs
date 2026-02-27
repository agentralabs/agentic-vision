//! Stress, edge-case, and boundary tests for agentic-vision.
//!
//! This suite covers:
//! - Edge cases: empty stores, invalid IDs, zero-dimension embeddings, unicode labels
//! - Boundary conditions: very large embeddings, similarity corner cases
//! - Stress tests: high-volume store operations, large .avis round-trips
//! - File format: corrupted .avis data, truncated files, header validation

use agentic_vision::{
    cosine_similarity, find_similar, AvisReader, AvisWriter, VisualMemoryStore, EMBEDDING_DIM,
};
use agentic_vision::{CaptureSource, ObservationMeta, VisualObservation};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal observation with the given embedding and labels.
fn make_obs(embedding: Vec<f32>, labels: Vec<String>) -> VisualObservation {
    VisualObservation {
        id: 0, // assigned by store
        timestamp: 1_700_000_000,
        session_id: 1,
        source: CaptureSource::Clipboard,
        embedding,
        thumbnail: vec![0xFF, 0xD8, 0xFF], // tiny fake JPEG header
        metadata: ObservationMeta {
            width: 64,
            height: 64,
            original_width: 1920,
            original_height: 1080,
            labels,
            description: None,
            quality_score: 0.5,
        },
        memory_link: None,
    }
}

/// Build a simple observation with a uniform embedding of given dimension.
fn make_obs_dim(dim: usize, val: f32) -> VisualObservation {
    make_obs(vec![val; dim], vec!["dim-test".into()])
}

// ===========================================================================
// Edge-case tests
// ===========================================================================

#[test]
fn edge_empty_store_get_returns_none() {
    let store = VisualMemoryStore::new(EMBEDDING_DIM);
    assert_eq!(store.count(), 0);
    assert!(store.get(1).is_none());
    assert!(store.get(0).is_none());
    assert!(store.get(u64::MAX).is_none());
}

#[test]
fn edge_empty_store_by_session_returns_empty() {
    let store = VisualMemoryStore::new(EMBEDDING_DIM);
    assert!(store.by_session(0).is_empty());
    assert!(store.by_session(1).is_empty());
    assert!(store.by_session(u32::MAX).is_empty());
}

#[test]
fn edge_empty_store_recent_returns_empty() {
    let store = VisualMemoryStore::new(EMBEDDING_DIM);
    assert!(store.recent(0).is_empty());
    assert!(store.recent(10).is_empty());
    assert!(store.recent(usize::MAX).is_empty());
}

#[test]
fn edge_empty_store_in_time_range_returns_empty() {
    let store = VisualMemoryStore::new(EMBEDDING_DIM);
    assert!(store.in_time_range(0, u64::MAX).is_empty());
}

#[test]
fn edge_get_invalid_id_after_additions() {
    let mut store = VisualMemoryStore::new(3);
    let id = store.add(make_obs(vec![1.0, 2.0, 3.0], vec![]));
    assert!(store.get(id).is_some());
    // IDs that were never assigned
    assert!(store.get(0).is_none());
    assert!(store.get(id + 1).is_none());
    assert!(store.get(u64::MAX).is_none());
}

#[test]
fn edge_zero_dimension_embedding() {
    // An observation with an empty embedding should still be storable.
    let mut store = VisualMemoryStore::new(0);
    let id = store.add(make_obs(vec![], vec![]));
    assert_eq!(store.count(), 1);
    let obs = store.get(id).unwrap();
    assert!(obs.embedding.is_empty());
}

#[test]
fn edge_unicode_labels() {
    let labels = vec![
        "\u{1F600}".to_string(),                // emoji
        "\u{4e16}\u{754c}".to_string(),         // Chinese characters
        "\u{0410}\u{0411}\u{0412}".to_string(), // Cyrillic
        "caf\u{00e9}".to_string(),              // accented Latin
        "".to_string(),                         // empty string label
        " ".to_string(),                        // whitespace label
    ];
    let mut store = VisualMemoryStore::new(3);
    let id = store.add(make_obs(vec![0.1, 0.2, 0.3], labels.clone()));
    let obs = store.get(id).unwrap();
    assert_eq!(obs.metadata.labels, labels);
}

#[test]
fn edge_empty_labels_vec() {
    let mut store = VisualMemoryStore::new(3);
    let id = store.add(make_obs(vec![1.0, 2.0, 3.0], vec![]));
    let obs = store.get(id).unwrap();
    assert!(obs.metadata.labels.is_empty());
}

// ===========================================================================
// Boundary-condition tests for cosine_similarity
// ===========================================================================

#[test]
fn boundary_cosine_identical_vectors() {
    let v = vec![0.3, 0.5, 0.7, 0.11, 0.13];
    let sim = cosine_similarity(&v, &v);
    assert!(
        (sim - 1.0).abs() < 1e-5,
        "identical vectors should yield ~1.0, got {sim}"
    );
}

#[test]
fn boundary_cosine_opposite_vectors() {
    let a = vec![1.0, 2.0, 3.0];
    let b: Vec<f32> = a.iter().map(|x| -x).collect();
    let sim = cosine_similarity(&a, &b);
    assert!(
        (sim + 1.0).abs() < 1e-5,
        "opposite vectors should yield ~-1.0, got {sim}"
    );
}

#[test]
fn boundary_cosine_orthogonal_vectors() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 1.0, 0.0];
    let sim = cosine_similarity(&a, &b);
    assert!(
        sim.abs() < 1e-5,
        "orthogonal vectors should yield ~0.0, got {sim}"
    );
}

#[test]
fn boundary_cosine_zero_vectors() {
    let z = vec![0.0; 128];
    assert_eq!(cosine_similarity(&z, &z), 0.0, "zero vs zero should be 0");
    let v = vec![1.0; 128];
    assert_eq!(
        cosine_similarity(&z, &v),
        0.0,
        "zero vs non-zero should be 0"
    );
}

#[test]
fn boundary_cosine_mismatched_lengths() {
    assert_eq!(cosine_similarity(&[1.0, 2.0], &[1.0]), 0.0);
    assert_eq!(cosine_similarity(&[], &[1.0]), 0.0);
    assert_eq!(cosine_similarity(&[1.0], &[]), 0.0);
}

#[test]
fn boundary_cosine_large_dimension_vector() {
    // Verify numerical stability with a high-dimensional vector.
    let dim = 4096;
    let a: Vec<f32> = (0..dim).map(|i| (i as f32) * 0.001).collect();
    let b: Vec<f32> = (0..dim).map(|i| ((dim - i) as f32) * 0.001).collect();
    let sim = cosine_similarity(&a, &b);
    assert!(sim.is_finite(), "similarity should be finite, got {sim}");
    assert!(
        (-1.0..=1.0).contains(&sim),
        "similarity should be in [-1, 1], got {sim}"
    );
}

#[test]
fn boundary_cosine_very_small_values() {
    let a = vec![1e-38_f32; 16];
    let b = vec![1e-38_f32; 16];
    let sim = cosine_similarity(&a, &b);
    // Both vectors point the same direction; result should be ~1.0 or 0.0 if
    // underflow collapses norms.
    assert!(sim.is_finite(), "should not be NaN/Inf, got {sim}");
}

// ===========================================================================
// Boundary-condition tests for find_similar
// ===========================================================================

#[test]
fn boundary_find_similar_empty_observations() {
    let matches = find_similar(&[1.0, 0.0, 0.0], &[], 10, 0.0);
    assert!(matches.is_empty());
}

#[test]
fn boundary_find_similar_top_k_zero() {
    let obs = vec![make_obs(vec![1.0, 0.0, 0.0], vec![])];
    let matches = find_similar(&[1.0, 0.0, 0.0], &obs, 0, 0.0);
    assert!(matches.is_empty(), "top_k=0 should return nothing");
}

#[test]
fn boundary_find_similar_high_min_similarity_filters_all() {
    let mut store = VisualMemoryStore::new(3);
    store.add(make_obs(vec![1.0, 0.0, 0.0], vec![]));
    store.add(make_obs(vec![0.0, 1.0, 0.0], vec![]));
    // Query with min_similarity > 1.0 (impossible threshold)
    let matches = find_similar(&[0.5, 0.5, 0.0], &store.observations, 10, 1.1);
    assert!(matches.is_empty(), "impossible threshold should filter all");
}

// ===========================================================================
// Stress tests: high-volume operations
// ===========================================================================

#[test]
fn stress_add_1000_observations() {
    let count = 1_000;
    let mut store = VisualMemoryStore::new(EMBEDDING_DIM);
    for i in 0..count {
        let emb = vec![(i as f32) / (count as f32); EMBEDDING_DIM as usize];
        store.add(make_obs(emb, vec![format!("label-{i}")]));
    }
    assert_eq!(store.count(), count);

    // Verify IDs are sequential 1..=count
    for i in 1..=count {
        assert!(
            store.get(i as u64).is_some(),
            "observation {i} should exist"
        );
    }
    assert!(store.get(0).is_none());
    assert!(store.get((count + 1) as u64).is_none());
}

#[test]
fn stress_find_similar_at_scale() {
    let count = 500;
    let dim = 64;
    let mut observations = Vec::with_capacity(count);
    for i in 0..count {
        let mut emb = vec![0.0f32; dim];
        emb[i % dim] = 1.0; // each observation is a one-hot-ish vector
        let mut obs = make_obs(emb, vec![]);
        obs.id = (i + 1) as u64;
        observations.push(obs);
    }

    let query = {
        let mut q = vec![0.0f32; dim];
        q[0] = 1.0;
        q
    };

    let matches = find_similar(&query, &observations, 5, 0.0);
    assert!(!matches.is_empty(), "should find at least one match");
    // The best match should be the observations with emb[0]=1.0
    assert!(
        matches[0].similarity > 0.9,
        "top match should be very similar, got {}",
        matches[0].similarity
    );
}

#[test]
fn stress_avis_round_trip_heavy() {
    let count = 500;
    let dim = 32;
    let mut store = VisualMemoryStore::new(dim as u32);

    for i in 0..count {
        let emb: Vec<f32> = (0..dim).map(|d| ((i * dim + d) as f32) * 0.01).collect();
        store.add(make_obs(emb, vec![format!("obs-{i}")]));
    }

    // Serialize to in-memory buffer
    let mut buf = Vec::new();
    AvisWriter::write_to(&store, &mut buf).unwrap();
    assert!(buf.len() > 64, "buffer should contain header + payload");

    // Deserialize
    let loaded = AvisReader::read_from(&mut &buf[..]).unwrap();
    assert_eq!(loaded.count(), count);
    assert_eq!(loaded.embedding_dim, dim as u32);

    // Spot-check a few observations
    let first = loaded.get(1).unwrap();
    assert_eq!(first.metadata.labels, vec!["obs-0".to_string()]);
    let last = loaded.get(count as u64).unwrap();
    assert_eq!(last.metadata.labels, vec![format!("obs-{}", count - 1)]);
}

#[test]
fn stress_avis_file_round_trip_heavy() {
    let count = 200;
    let dim = 16;
    let mut store = VisualMemoryStore::new(dim as u32);
    for i in 0..count {
        store.add(make_obs_dim(dim, i as f32 * 0.1));
    }

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("heavy.avis");

    AvisWriter::write_to_file(&store, &path).unwrap();
    let loaded = AvisReader::read_from_file(&path).unwrap();
    assert_eq!(loaded.count(), count);
}

// ===========================================================================
// File-format edge cases
// ===========================================================================

#[test]
fn edge_avis_corrupted_magic_bytes() {
    // Valid header length but wrong magic
    let mut buf = [0u8; 128];
    buf[0..4].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
    let result = AvisReader::read_from(&mut &buf[..]);
    assert!(result.is_err(), "corrupted magic should fail");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Invalid magic"),
        "error should mention invalid magic: {err_msg}"
    );
}

#[test]
fn edge_avis_truncated_header() {
    // Header is 64 bytes; supply only 10
    let buf = [0u8; 10];
    let result = AvisReader::read_from(&mut &buf[..]);
    assert!(result.is_err(), "truncated header should fail");
}

#[test]
fn edge_avis_empty_file() {
    let buf: &[u8] = &[];
    let result = AvisReader::read_from(&mut &buf[..]);
    assert!(result.is_err(), "empty buffer should fail");
}

#[test]
fn edge_avis_valid_header_corrupted_payload() {
    // Write a valid store, then corrupt the payload bytes
    let store = VisualMemoryStore::new(3);
    let mut buf = Vec::new();
    AvisWriter::write_to(&store, &mut buf).unwrap();

    // Corrupt payload: overwrite bytes after the 64-byte header
    if buf.len() > 68 {
        for b in buf[64..68].iter_mut() {
            *b = 0xFF;
        }
    }
    let result = AvisReader::read_from(&mut &buf[..]);
    // May succeed (JSON is resilient) or fail; either way it must not panic.
    let _ = result;
}

#[test]
fn edge_avis_wrong_version() {
    let store = VisualMemoryStore::new(3);
    let mut buf = Vec::new();
    AvisWriter::write_to(&store, &mut buf).unwrap();

    // Overwrite version field (bytes 4-5) with 99
    buf[4] = 99;
    buf[5] = 0;

    let result = AvisReader::read_from(&mut &buf[..]);
    assert!(result.is_err(), "wrong version should fail");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Unsupported version"),
        "error should mention version: {err_msg}"
    );
}

// ===========================================================================
// Store mutation edge cases
// ===========================================================================

#[test]
fn edge_get_mut_and_modify() {
    let mut store = VisualMemoryStore::new(3);
    let id = store.add(make_obs(vec![1.0, 2.0, 3.0], vec!["original".into()]));

    // Mutate in place
    let obs = store.get_mut(id).unwrap();
    obs.metadata.labels = vec!["modified".into()];
    obs.memory_link = Some(42);

    // Read back
    let obs = store.get(id).unwrap();
    assert_eq!(obs.metadata.labels, vec!["modified".to_string()]);
    assert_eq!(obs.memory_link, Some(42));
}

#[test]
fn edge_get_mut_invalid_id() {
    let mut store = VisualMemoryStore::new(3);
    store.add(make_obs(vec![1.0, 2.0, 3.0], vec![]));
    assert!(store.get_mut(999).is_none());
}

// ===========================================================================
// Store query edge cases
// ===========================================================================

#[test]
fn boundary_by_session_filters_correctly() {
    let mut store = VisualMemoryStore::new(3);

    // Session 1
    let mut obs1 = make_obs(vec![1.0, 0.0, 0.0], vec!["s1".into()]);
    obs1.session_id = 1;
    store.add(obs1);

    // Session 2
    let mut obs2 = make_obs(vec![0.0, 1.0, 0.0], vec!["s2".into()]);
    obs2.session_id = 2;
    store.add(obs2);

    // Session 1 again
    let mut obs3 = make_obs(vec![0.0, 0.0, 1.0], vec!["s1-again".into()]);
    obs3.session_id = 1;
    store.add(obs3);

    assert_eq!(store.by_session(1).len(), 2);
    assert_eq!(store.by_session(2).len(), 1);
    assert_eq!(store.by_session(3).len(), 0);
}

#[test]
fn boundary_in_time_range_filters_correctly() {
    let mut store = VisualMemoryStore::new(3);

    let mut obs_early = make_obs(vec![1.0, 0.0, 0.0], vec![]);
    obs_early.timestamp = 100;
    store.add(obs_early);

    let mut obs_mid = make_obs(vec![0.0, 1.0, 0.0], vec![]);
    obs_mid.timestamp = 500;
    store.add(obs_mid);

    let mut obs_late = make_obs(vec![0.0, 0.0, 1.0], vec![]);
    obs_late.timestamp = 900;
    store.add(obs_late);

    assert_eq!(store.in_time_range(0, 1000).len(), 3);
    assert_eq!(store.in_time_range(100, 100).len(), 1); // exact match
    assert_eq!(store.in_time_range(101, 499).len(), 0);
    assert_eq!(store.in_time_range(500, 900).len(), 2);
    assert_eq!(store.in_time_range(901, u64::MAX).len(), 0);
}

#[test]
fn boundary_recent_ordering_and_limit() {
    let mut store = VisualMemoryStore::new(3);

    for t in [300u64, 100, 500, 200, 400] {
        let mut obs = make_obs(vec![1.0, 0.0, 0.0], vec![format!("t-{t}")]);
        obs.timestamp = t;
        store.add(obs);
    }

    let recent2 = store.recent(2);
    assert_eq!(recent2.len(), 2);
    assert_eq!(recent2[0].timestamp, 500);
    assert_eq!(recent2[1].timestamp, 400);

    let recent_all = store.recent(100);
    assert_eq!(recent_all.len(), 5);
    // Should be descending
    for w in recent_all.windows(2) {
        assert!(w[0].timestamp >= w[1].timestamp);
    }
}

// ===========================================================================
// Round-trip preserves unicode and special data
// ===========================================================================

#[test]
fn stress_avis_round_trip_unicode_labels() {
    let mut store = VisualMemoryStore::new(3);
    let labels = vec![
        "\u{1F4A9}".to_string(), // poop emoji
        "\u{0000}".to_string(),  // null char
        "\n\t\r".to_string(),    // whitespace
        "a".repeat(10_000),      // very long label
    ];
    store.add(make_obs(vec![0.1, 0.2, 0.3], labels.clone()));

    let mut buf = Vec::new();
    AvisWriter::write_to(&store, &mut buf).unwrap();
    let loaded = AvisReader::read_from(&mut &buf[..]).unwrap();
    assert_eq!(loaded.count(), 1);
    assert_eq!(loaded.get(1).unwrap().metadata.labels, labels);
}
