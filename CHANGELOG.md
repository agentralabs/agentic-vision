# Changelog

All notable changes to AgenticVision will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5] - 2026-02-23

### Fixed
- MCP stdio shutdown reliability: server now exits immediately after a successful `shutdown` request instead of waiting for stdin EOF.
- Prevents lingering/orphan stdio MCP processes in clients that keep pipes open after shutdown.

## [0.1.4] - 2026-02-23

### Fixed
- Publish parity fix: aligned `agentic-vision` and `agentic-vision-mcp` crate versions so crates.io verification matches the current core metadata/API surface.

## [0.1.3] - 2026-02-22

### Fixed
- Hardened MCP stdio framing for clients using Content-Length transport.
- Formatting and compatibility follow-ups for MCP server command handling.

### Changed
- Updated workspace orchestration and install profile documentation.

## [Unreleased] — v0.2.0 Remote Server Support

### Planned

- **Remote HTTP/SSE transport** (`serve-http` command)
  - `--token` flag for bearer authentication
  - `--multi-tenant --data-dir` for per-user vision files
  - `/health` endpoint for monitoring
  - `--tls-cert` / `--tls-key` for native HTTPS (optional)

- **OCR with Tesseract** (`--features ocr`)
  - `extract_text` tool for text extraction from images

- **Clipboard TIFF fix**
  - Improved TIFF handling for macOS clipboard captures

- **New CLI commands**
  - `delete` — remove a specific vision entry
  - `export` — export vision data to JSON
  - `compact` — defragment and optimize data file

- **Infrastructure**
  - Docker image (`agenticrevolution/agentic-vision-mcp`)
  - docker-compose with Caddy reverse proxy
  - Systemd service file
  - `docs/remote-deployment.md`

- **New error codes**
  - `UNAUTHORIZED (-32803)`, `USER_NOT_FOUND (-32804)`, `RATE_LIMITED (-32805)`

Tracking: [#2](https://github.com/agentralabs/agentic-vision/issues/2)

## [0.1.1] - 2026-02-19

Native screenshot and clipboard capture support.

### Added

- **Screenshot capture** (`source.type = "screenshot"`)
  - macOS: `screencapture -x` with optional `-R x,y,w,h` region
  - Linux: fallback chain — `gnome-screenshot` → `scrot` → `maim` (full screen); `maim` → `import` (region)
  - Clear error messages for permission denied (macOS Screen Recording) and missing tools (Linux)

- **Clipboard capture** (`source.type = "clipboard"`)
  - macOS: AppleScript via `osascript` — tries PNG (`PNGf`) first, falls back to TIFF (`TIFF`) + `sips` conversion (handles `screencapture -c` output)
  - Linux: `xclip` → `wl-paste` (Wayland) fallback chain
  - Clear error when clipboard contains no image data

- **New error variant** `VisionError::Capture(String)` for capture-specific failures

- **RAII temp file cleanup** (`TempFileGuard`) ensures temporary files are removed on all code paths

- **Region parameter** in MCP tool schema: `source.region` object with `{ x, y, w, h }` fields for partial-screen capture

- **Refactored session manager** — extracted shared `store_capture()` helper to eliminate code duplication

### Test coverage

- 3 new unit tests (CI-safe — accept `VisionError::Capture` on headless environments)
- Rust core: 38 tests (was 35)
- Total across all suites: 91 tests (was 88)

### No new dependencies

Uses `std::process::Command` to invoke platform tools — zero new crate dependencies.

## [0.1.0] - 2026-02-19

First release. Two crates published to crates.io. 88 tests passing across all suites.

### Added

- **Core Library (`agentic-vision` v0.1.0)**
  - Binary `.avis` file format — 64-byte fixed header (magic `0x41564953`, version, capture count, timestamps), JSON payload with embedded JPEG thumbnails and 512-dim float vectors
  - CLIP ViT-B/32 image embedding via ONNX Runtime (512-dimensional vectors)
  - Deterministic fallback embedding when ONNX model is not present
  - Cosine similarity search with f64 intermediate precision
  - Pixel-level visual diff with 8×8 grid region detection (threshold 30/255)
  - Image capture from file paths, base64 data, and URLs
  - JPEG thumbnail generation and storage
  - Memory-mapped I/O via `memmap2`
  - Published to crates.io: `cargo install agentic-vision`

- **MCP Server (`agentic-vision-mcp` v0.1.0)**
  - 10 MCP tools: `vision_capture`, `vision_compare`, `vision_query`, `vision_ocr`, `vision_similar`, `vision_track`, `vision_diff`, `vision_link`, `session_start`, `session_end`
  - 6 resources via `avis://` URIs (capture, session, timeline, similar, stats, recent)
  - 4 prompt templates: observe, compare, track, describe
  - Stdio transport (MCP protocol v2024-11-05)
  - Session management with named observation sessions
  - Cross-system linking to AgenticMemory `.amem` nodes via `vision_link`
  - Published to crates.io: `cargo install agentic-vision-mcp`

- **Monorepo structure**
  - Cargo workspace with `crates/agentic-vision/` (core) and `crates/agentic-vision-mcp/` (MCP server)
  - Python integration tests in `tests/integration/`
  - Multi-agent scenario tests (shared files, vision-memory linking, rapid handoff)

- **Research Paper**
  - [Paper II: AgenticVision-MCP — 8 pages, 4 TikZ figures, 7 booktabs tables, 18 references](publication/paper-ii-agentic-vision-mcp/agentic-vision-mcp-paper.pdf)
  - All benchmark data from real test runs (zero fabrication)

### Test coverage

- Rust core: 35 tests (unit + integration)
- Python SDK: 47 tests (edge cases, format validation)
- MCP integration: 3 tests (Python → Rust stdio transport)
- Multi-agent: 3 tests (shared file, vision-memory linking, 5-agent rapid handoff)
- Total across all suites: 88 tests

### Performance (Apple M4, macOS 26.2, Rust 1.90.0 --release)

| Operation | Latency |
|:---|---:|
| Image capture (embed + store) | 47 ms |
| Similarity search (top-5) | 1-2 ms |
| Visual diff | <1 ms |
| MCP tool round-trip | 7.2 ms |
| Storage per capture | ~4.26 KB |

[0.1.1]: https://github.com/agentralabs/agentic-vision/releases/tag/v0.1.1
[0.1.0]: https://github.com/agentralabs/agentic-vision/releases/tag/v0.1.0
