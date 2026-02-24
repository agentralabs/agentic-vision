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
| Artifact format | `.avis` (LZ4 compressed) |

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

## Artifact size

| Captures | Approximate `.avis` size |
|:--|--:|
| 100 | ~2 MB |
| 1,000 | ~18 MB |
| 10,000 | ~170 MB |

Sizes vary with image thumbnails and metadata. LZ4 compression typically achieves 2-3x reduction.

## Notes

These numbers are directional and depend on hardware, image size, and embedding model. Real-world performance may differ based on:

- Disk I/O speed for artifact writes
- Image resolution and format (PNG vs JPEG)
- Number of existing captures (affects search time)
- OCR text density

## Run local validation

```bash
cargo test --workspace
cargo build --release
agentic-vision-mcp info
```
