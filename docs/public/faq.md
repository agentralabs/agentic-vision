---
status: stable
---

# FAQ

## General

### Is there a `cortex` CLI in this project?

No. Public command surface is `agentic-vision-mcp`.

### Which artifact does Vision use?

`.avis`. It is a binary format with a 64-byte header (magic bytes `AVIS`, version 1) followed by a JSON-serialized payload of all visual observations.

### What image formats are supported?

PNG, JPEG, WebP, GIF, BMP, TIFF, and ICO. Format detection is automatic for file captures. For base64 input, provide a MIME type hint (`image/png`, `image/jpeg`, `image/webp`, `image/gif`).

## Installation

### What platforms does Vision support?

macOS and Linux are fully supported. Screenshot capture uses `screencapture` on macOS and `gnome-screenshot`/`scrot`/`maim` on Linux. Clipboard capture uses `osascript` on macOS and `xclip`/`wl-paste` on Linux.

### Does Vision require a GPU?

No. CLIP inference runs on CPU via ONNX Runtime with a single thread. If no ONNX model is installed, Vision operates in fallback mode with zero-vector embeddings. All features except semantic similarity continue to work.

### Where does the CLIP model go?

Place the ONNX model at `~/.agentic-vision/models/clip-vit-base-patch32-visual.onnx`. You can also pass a custom path via the `model_path` parameter when initializing the embedding engine.

## Usage

### How do I capture a screenshot of a specific region?

Use the `vision_capture` tool with `source.type` set to `screenshot` and provide a `region` object with `x`, `y`, `w`, `h` fields specifying the rectangle in pixels.

### How does similarity search work?

It computes cosine similarity between the query embedding and all stored embeddings, then returns the top-k results above a minimum similarity threshold. The search is a linear scan, completing in about 8 ms for 1,000 captures.

### What is a quality score?

A value between 0.0 and 1.0 computed from image resolution, metadata richness (labels and description), and embedding quality. Higher scores indicate more useful captures.

## MCP

### Does it work with all MCP clients?

Yes. Use a standard MCP server entry and restart the client after install. The server communicates over stdio transport following the MCP protocol specification.

### How many tools does the MCP server expose?

The server exposes core tools (capture, query, similar, diff, compare, OCR, track, health, link), grounding tools (ground, evidence, suggest), workspace tools (create, add, list, query, compare, xref), session tools (start, end), observation logging, and V3 invention tools for advanced cognition and prediction.

### Can server runtime read laptop files directly?

No. Sync artifacts to server storage first.

## Data and security

### Where is visual data stored?

All data stays in local `.avis` files. No data is sent to external servers. The default storage location is determined by your project root path.

### Can I back up or migrate my visual memory?

Yes. Copy the `.avis` file to another location. The format is self-contained and portable across machines. You can also use workspaces to load multiple `.avis` files for cross-project comparison.

### How does concurrent access work?

File locking uses a sidecar `.avis.lock` file. Sessions are identified by PID. On save, a merge-on-save strategy resolves concurrent writes.

## Cross-sister integration

### Can captures link to AgenticMemory nodes?

Yes. Each capture has an optional `memory_link` field. Use `vision_link` to associate a capture ID with a memory node ID. The V3 `vision_bind_memory` tool provides richer cross-modal binding.

### What should users do after installation?

- Restart their MCP client.
- Run `agentic-vision-mcp info`.
- Optionally share feedback in the issue tracker.
