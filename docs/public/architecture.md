---
status: stable
---

# Architecture

AgenticVision is a 4-crate Rust workspace for persistent visual memory.

## Workspace Structure

```
agentic-vision/
  Cargo.toml                    (workspace root)
  crates/
    agentic-vision/             (core library)
    agentic-vision-mcp/         (MCP server)
    agentic-vision-cli/         (CLI binary: avis)
    agentic-vision-ffi/         (C FFI shared library)
```

## Crate Responsibilities

### agentic-vision

The core library. All visual memory logic lives here.

- Image capture: file, base64, screenshot, clipboard sources
- Thumbnail generation and quality scoring
- Embedding engine: CLIP ONNX model for visual embeddings (512-dim default)
- Similarity search: cosine similarity over embedding vectors
- Pixel-level diff: region-based change detection
- File format: `.avis` binary format (magic `AVIS`, version 1)
- Storage: `AvisReader` / `AvisWriter` for `.avis` file I/O
- No MCP, CLI, or FFI concerns

### agentic-vision-mcp

The MCP server binary (`agentic-vision-mcp`).

- JSON-RPC 2.0 over stdio (default) or HTTP/SSE (with `sse` feature)
- 21 core MCP tools + 48 V3 Perception Invention tools (69 total)
- MCP resources via `avis://` URI scheme
- 4 MCP prompts (observe, compare, track, describe)
- Auto-session lifecycle management
- Multi-tenant mode for HTTP deployments
- Ghost Writer background sync to Claude, Cursor, Windsurf, Cody
- Content-Length framing with 8 MiB frame limit
- Input validation: no silent fallback for invalid parameters

### agentic-vision-cli

The command-line interface binary (`avis`).

- Human-friendly terminal output
- All core operations exposed as subcommands
- Text and JSON output formats
- File validation and info commands
- Workspace management across multiple `.avis` files
- Interactive REPL mode (no arguments)
- Shell completion generation (bash, zsh, fish, powershell, elvish)

### agentic-vision-ffi

C-compatible shared library for cross-language integration.

- Minimal FFI facade exposing the crate version
- Designed for future expansion with opaque handle pattern
- Library name: `libagentic_vision_ffi`

## Data Flow

```
Agent (Claude/GPT/etc.)
  |
  | MCP protocol (JSON-RPC 2.0 over stdio)
  v
agentic-vision-mcp
  |
  | Rust function calls
  v
agentic-vision (core)
  |
  | Binary I/O
  v
project.avis (file)
```

## File Format

The `.avis` binary format:

| Offset | Size | Field |
|--------|------|-------|
| 0 | 4 | Magic bytes: `AVIS` (0x41564953) |
| 4 | 2 | Version: `0x0001` |
| 6 | 58 | Header (reserved, 64 bytes total) |
| 64 | ... | Observation data (JSON-serialized payload) |

The payload contains all visual observations with embeddings, metadata, thumbnails, and session information.

## Cross-Sister Integration

AgenticVision integrates with other Agentra sisters:

- **AgenticMemory**: Visual captures link to memory nodes via `vision_link`. Grounding tools verify visual claims against stored evidence.
- **AgenticCodebase**: Cross-modal binding connects visual captures to code symbols via `vision_bind_code`.
- **AgenticIdentity**: Visual observations bind to identity records via `vision_bind_identity`.
- **AgenticTime**: Temporal vision tools reconstruct visual state at specific timestamps. Captures bind to time entries via `vision_bind_time`.

## Runtime Isolation

Each project gets its own `.avis` file. The MCP server resolves the file path via explicit argument, environment variable, or deterministic default path. File-level locking ensures safe concurrent access. Multi-tenant HTTP mode isolates per-user vision files in a data directory.
