---
status: stable
---

# Benchmarks

Measured on Apple Silicon (M2 Pro, 16 GB) with release builds. All times are wall-clock averages over 100 iterations unless noted.

## Test environment

| Component | Value |
|:--|:--|
| Hardware | Apple M2 Pro, 16 GB RAM |
| OS | macOS 14 |
| Rust | 1.76 (release build, LTO) |
| Embedding model | CLIP ViT-B/32 (512-dim) |
| Artifact format | `.avis` (64-byte header + JSON payload) |
| Image library | `image` crate with Lanczos3 resampling |
| ONNX runtime | `ort` crate (single intra-thread) |

## Core operations

| Operation | Typical Time | Notes |
|:--|--:|:--|
| Image capture (file to embed to store) | ~47 ms | 1024x768 PNG |
| Image capture (screenshot) | ~62 ms | Includes screen grab |
| Image capture (4K image) | ~95 ms | 3840x2160 |
| Similarity search (top-5, 100 captures) | ~1-2 ms | Cosine distance on embeddings |
| Similarity search (top-5, 1000 captures) | ~8 ms | Linear scan |
| Visual diff | <1 ms | Pixel comparison, same dimensions |
| Visual compare | ~2 ms | Embedding cosine + optional diff |
| Quality score computation | <1 ms | Resolution + metadata + embedding norm |
| OCR extraction | ~120 ms | Depends on text density |
| MCP tool round-trip | ~7 ms | stdio transport overhead |

## Performance tiers

Operations fall into three latency tiers:

| Tier | Latency | Operations |
|:--|:--|:--|
| Sub-millisecond | <1 ms | Quality score, visual diff (same dimensions), single embedding cosine |
| Low millisecond | 1-10 ms | Similarity search (up to 1K captures), visual compare, MCP tool overhead |
| Capture-bound | 40-120 ms | Image capture with embedding, OCR extraction, screenshot grab |

The bottleneck for capture operations is CLIP inference (224x224 resize + ONNX forward pass). Without a model loaded (fallback mode), capture drops to ~5 ms since only thumbnail generation runs.

## Artifact size

| Captures | Approximate `.avis` size |
|:--|--:|
| 100 | ~2 MB |
| 1,000 | ~18 MB |
| 10,000 | ~170 MB |

Sizes vary with image thumbnails and metadata. Thumbnails are JPEG-encoded at 85% quality with a maximum dimension of 512 pixels, which is the primary size driver. Embedding vectors (512 x 4 bytes = 2 KB each) are a smaller contributor.

## Scaling analysis

| Captures | Similarity search (top-5) | File open time | Memory (RSS) |
|:--|--:|--:|--:|
| 100 | ~1 ms | ~5 ms | ~12 MB |
| 1,000 | ~8 ms | ~40 ms | ~45 MB |
| 10,000 | ~80 ms | ~350 ms | ~380 MB |

Similarity search scales linearly with capture count because it performs a full scan over embeddings. File open time scales linearly with payload size (JSON deserialization). For stores above 10,000 captures, consider using workspaces to partition captures across multiple `.avis` files.

## Comparison with alternatives

| Aspect | AgenticVision (.avis) | SQLite + blobs | Filesystem + JSON sidecar |
|:--|:--|:--|:--|
| Embedding storage | Native (per-observation) | Manual schema | Separate files |
| Similarity search | Built-in cosine scan | Manual query | External library |
| Visual diff | Built-in pixel comparison | Not available | External tool |
| Portability | Single file | Single file | Directory tree |
| Concurrent access | File locking + PID sessions | WAL mode | No safety |
| MCP integration | Native stdio transport | Manual adapter | Manual adapter |

## Reproducing benchmarks

To reproduce these numbers on your own hardware:

```bash
# Clone and build
git clone https://github.com/agentralabs/agentic-vision
cd agentic-vision

# Run the benchmark suite
cargo bench --package agentic-vision

# Run the stress tests for MCP tool latency
cargo test --package agentic-vision-mcp --test phase0_stress -- --nocapture
cargo test --package agentic-vision-mcp --test phase1_v2_stress -- --nocapture

# Quick validation
cargo test --workspace
cargo build --release
agentic-vision-mcp info
```

Benchmark results depend on available hardware, background load, and whether a CLIP ONNX model is installed. Numbers above reflect a quiet system with the model loaded.

## Notes

These numbers are directional and depend on hardware, image size, and embedding model. Real-world performance may differ based on:

- Disk I/O speed for artifact writes
- Image resolution and format (PNG vs JPEG)
- Number of existing captures (affects search time)
- OCR text density
- Whether the CLIP model is loaded or running in fallback mode
- Background system load during screenshot capture
