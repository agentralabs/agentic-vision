//! Criterion benchmarks for agentic-vision.
//!
//! These benchmarks cover the core operations that do NOT require a GPU or ONNX
//! model: similarity computation, .avis file I/O, store operations, image diffing,
//! thumbnail generation, and observation serialization.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::io::Cursor;
use tempfile::tempdir;

use agentic_vision::{
    compute_diff, cosine_similarity, find_similar, generate_thumbnail, AvisReader, AvisWriter,
    CaptureSource, ObservationMeta, VisualMemoryStore, VisualObservation,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a synthetic observation with an embedding of the given dimension.
fn make_observation(id: u64, session_id: u32, embedding_dim: usize) -> VisualObservation {
    let mut embedding = Vec::with_capacity(embedding_dim);
    // Deterministic pseudo-random values seeded by id
    let mut val = (id as f32 + 1.0) * 0.1;
    for _ in 0..embedding_dim {
        val = (val * 1.5 + 0.3).sin().abs();
        embedding.push(val);
    }

    VisualObservation {
        id,
        timestamp: 1_700_000_000 + id,
        session_id,
        source: CaptureSource::File {
            path: format!("/capture/{id}.png"),
        },
        embedding,
        thumbnail: vec![0xFF; 128], // small fake JPEG bytes
        metadata: ObservationMeta {
            width: 512,
            height: 512,
            original_width: 1920,
            original_height: 1080,
            labels: vec!["bench".to_string()],
            description: Some(format!("Observation {id}")),
            quality_score: 0.9,
        },
        memory_link: None,
    }
}

/// Build a store pre-populated with `n` observations.
fn make_store(n: usize, embedding_dim: usize) -> VisualMemoryStore {
    let mut store = VisualMemoryStore::new(embedding_dim as u32);
    for i in 0..n {
        store.add(make_observation(0, (i % 4) as u32, embedding_dim));
    }
    store
}

/// Build a random-ish f32 vector of given length, seeded by `seed`.
fn make_vector(dim: usize, seed: u32) -> Vec<f32> {
    let mut v = Vec::with_capacity(dim);
    let mut val = (seed as f32 + 0.7) * 0.3;
    for _ in 0..dim {
        val = (val * 2.1 + 0.5).cos().abs();
        v.push(val);
    }
    v
}

// ---------------------------------------------------------------------------
// 1. Similarity computation
// ---------------------------------------------------------------------------

fn bench_cosine_similarity(c: &mut Criterion) {
    let mut group = c.benchmark_group("cosine_similarity");

    for &dim in &[128, 512, 1024] {
        let a = make_vector(dim, 1);
        let b = make_vector(dim, 2);

        group.bench_with_input(BenchmarkId::new("dim", dim), &dim, |bench, _| {
            bench.iter(|| cosine_similarity(black_box(&a), black_box(&b)));
        });
    }
    group.finish();
}

fn bench_find_similar(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_similar");

    for &n in &[10, 100, 1000] {
        let store = make_store(n, 512);
        let query = make_vector(512, 99);

        group.bench_with_input(BenchmarkId::new("observations", n), &n, |bench, _| {
            bench.iter(|| find_similar(black_box(&query), black_box(&store.observations), 10, 0.0));
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 2. .avis serialization / deserialization (in-memory)
// ---------------------------------------------------------------------------

fn bench_avis_write_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("avis_write_memory");

    for &n in &[0, 10, 100] {
        let store = make_store(n, 512);

        group.bench_with_input(BenchmarkId::new("observations", n), &n, |bench, _| {
            bench.iter(|| {
                let mut buf = Vec::with_capacity(4096);
                AvisWriter::write_to(black_box(&store), &mut buf).unwrap();
                black_box(buf);
            });
        });
    }
    group.finish();
}

fn bench_avis_read_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("avis_read_memory");

    for &n in &[0, 10, 100] {
        let store = make_store(n, 512);
        let mut buf = Vec::new();
        AvisWriter::write_to(&store, &mut buf).unwrap();

        group.bench_with_input(BenchmarkId::new("observations", n), &n, |bench, _| {
            bench.iter(|| {
                let mut cursor = Cursor::new(black_box(&buf));
                let loaded = AvisReader::read_from(&mut cursor).unwrap();
                black_box(loaded);
            });
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 3. .avis file I/O (disk roundtrip)
// ---------------------------------------------------------------------------

fn bench_avis_file_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("avis_file_io");

    for &n in &[10, 100] {
        let store = make_store(n, 512);

        group.bench_with_input(BenchmarkId::new("write_file", n), &n, |bench, _| {
            bench.iter_batched(
                || tempdir().unwrap(),
                |dir| {
                    let path = dir.path().join("bench.avis");
                    AvisWriter::write_to_file(black_box(&store), &path).unwrap();
                    black_box(path);
                },
                BatchSize::SmallInput,
            );
        });

        // Prepare a file on disk for read benchmarks
        let dir = tempdir().unwrap();
        let path = dir.path().join("bench_read.avis");
        AvisWriter::write_to_file(&store, &path).unwrap();

        group.bench_with_input(BenchmarkId::new("read_file", n), &n, |bench, _| {
            bench.iter(|| {
                let loaded = AvisReader::read_from_file(black_box(&path)).unwrap();
                black_box(loaded);
            });
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 4. VisualMemoryStore operations
// ---------------------------------------------------------------------------

fn bench_store_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("store_operations");

    // Benchmark: add observation
    group.bench_function("add_observation", |bench| {
        bench.iter_batched(
            || make_store(0, 512),
            |mut store| {
                let obs = make_observation(0, 1, 512);
                store.add(black_box(obs));
                black_box(store);
            },
            BatchSize::SmallInput,
        );
    });

    // Benchmark: get by ID (from a store with 1000 observations)
    let large_store = make_store(1000, 512);
    group.bench_function("get_by_id_1000", |bench| {
        bench.iter(|| {
            let obs = large_store.get(black_box(500));
            black_box(obs);
        });
    });

    // Benchmark: by_session
    group.bench_function("by_session_1000", |bench| {
        bench.iter(|| {
            let results = large_store.by_session(black_box(2));
            black_box(results);
        });
    });

    // Benchmark: in_time_range
    group.bench_function("in_time_range_1000", |bench| {
        bench.iter(|| {
            let results =
                large_store.in_time_range(black_box(1_700_000_200), black_box(1_700_000_800));
            black_box(results);
        });
    });

    // Benchmark: recent
    group.bench_function("recent_10_from_1000", |bench| {
        bench.iter(|| {
            let results = large_store.recent(black_box(10));
            black_box(results);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 5. Observation JSON serialization
// ---------------------------------------------------------------------------

fn bench_observation_serde(c: &mut Criterion) {
    let mut group = c.benchmark_group("observation_serde");

    let obs = make_observation(1, 1, 512);
    let json_bytes = serde_json::to_vec(&obs).unwrap();

    group.bench_function("serialize_json", |bench| {
        bench.iter(|| {
            let bytes = serde_json::to_vec(black_box(&obs)).unwrap();
            black_box(bytes);
        });
    });

    group.bench_function("deserialize_json", |bench| {
        bench.iter(|| {
            let loaded: VisualObservation = serde_json::from_slice(black_box(&json_bytes)).unwrap();
            black_box(loaded);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 6. Image diff (compute_diff)
// ---------------------------------------------------------------------------

fn bench_compute_diff(c: &mut Criterion) {
    let mut group = c.benchmark_group("compute_diff");

    for &size in &[64, 256] {
        // Create two images: one black, one with some white pixels
        let img_a = image::DynamicImage::new_rgb8(size, size);
        let mut img_b = image::DynamicImage::new_rgb8(size, size);
        if let Some(rgb) = img_b.as_mut_rgb8() {
            for (i, pixel) in rgb.pixels_mut().enumerate() {
                if i % 3 == 0 {
                    *pixel = image::Rgb([200, 200, 200]);
                }
            }
        }

        group.bench_with_input(
            BenchmarkId::new("pixels", format!("{size}x{size}")),
            &size,
            |bench, _| {
                bench.iter(|| {
                    compute_diff(
                        black_box(1),
                        black_box(2),
                        black_box(&img_a),
                        black_box(&img_b),
                    )
                    .unwrap()
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 7. Thumbnail generation
// ---------------------------------------------------------------------------

fn bench_generate_thumbnail(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_thumbnail");

    for &(w, h) in &[(256, 256), (1920, 1080)] {
        let img = image::DynamicImage::new_rgb8(w, h);

        group.bench_with_input(
            BenchmarkId::new("size", format!("{w}x{h}")),
            &(w, h),
            |bench, _| {
                bench.iter(|| {
                    let thumb = generate_thumbnail(black_box(&img));
                    black_box(thumb);
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion groups + main
// ---------------------------------------------------------------------------

criterion_group!(similarity, bench_cosine_similarity, bench_find_similar,);

criterion_group!(
    avis_format,
    bench_avis_write_memory,
    bench_avis_read_memory,
    bench_avis_file_roundtrip,
);

criterion_group!(store, bench_store_operations,);

criterion_group!(serde, bench_observation_serde,);

criterion_group!(imaging, bench_compute_diff, bench_generate_thumbnail,);

criterion_main!(similarity, avis_format, store, serde, imaging);
