# Benchmarks

Measured on Apple Silicon with release builds.

| Operation | Typical Time |
|:--|--:|
| Image capture (file -> embed -> store) | ~47 ms |
| Similarity search (top-5) | ~1-2 ms |
| Visual diff | <1 ms |
| MCP tool round-trip | ~7 ms |

These numbers are directional and depend on hardware, model path, and image size.

Run local validation:

```bash
cargo test --workspace
cargo build --release
agentic-vision-mcp info
```
