---
status: stable
---

# Concepts

## Persistent visual memory

AgenticVision stores visual observations in `.avis` files so agents can reuse them across sessions. Each `.avis` file is a self-contained binary archive with a 64-byte header (magic bytes `AVIS`, version, observation count, embedding dimension, session count, timestamps) followed by a serialized payload of all observations. The format is local-first and portable -- you can copy an `.avis` file between machines and it works without any external dependencies.

## Captures

A capture is the fundamental unit of visual memory. Each capture is represented as a `VisualObservation` containing:

- **id**: Auto-incrementing unique identifier within the store
- **timestamp**: Unix epoch seconds when the capture was taken
- **session_id**: Which session produced this capture
- **source**: How the image was acquired (file, base64, screenshot, or clipboard)
- **embedding**: 512-dimensional CLIP ViT-B/32 vector for semantic similarity
- **thumbnail**: JPEG-encoded preview image (max 512x512, Lanczos3 resampling, 85% quality)
- **metadata**: Width, height, original dimensions, labels, description, and quality score
- **memory_link**: Optional link to an AgenticMemory node ID for cross-sister binding

## Capture sources

AgenticVision supports four capture input methods:

- **File**: Load from a local image path. Supports PNG, JPEG, WebP, GIF, BMP, TIFF, and ICO formats.
- **Base64**: Decode from base64-encoded data with a MIME type hint for format detection.
- **Screenshot**: Grab the current screen or a specific rectangular region. On macOS this uses `screencapture -x`; on Linux it tries `gnome-screenshot`, then `scrot`, then `maim`, then ImageMagick `import`.
- **Clipboard**: Read the current clipboard image. On macOS this uses `osascript` to extract PNG or TIFF data (with automatic `sips` conversion for TIFF). On Linux it tries `xclip` then `wl-paste`.

## Sessions

Sessions group captures into logical work units. A session starts when an MCP client connects (auto-start on `initialized` notification) and ends when the client disconnects (`shutdown` or EOF). Sessions provide:

- Scoped capture retrieval via `by_session(session_id)`
- Session count tracking in the `.avis` header
- PID-based session IDs for concurrent access scenarios

## Quality scores

Every capture receives a quality score in the range [0.0, 1.0]. The score is computed from three factors:

- **Resolution**: Higher-resolution originals score higher. A 1920x1080 capture scores better than 320x240.
- **Metadata richness**: Captures with descriptions and labels score higher than bare captures.
- **Embedding norm**: A well-formed embedding (non-zero, properly L2-normalized) contributes positively.

Quality scores are used by `vision_health` to assess overall store health and by `vision_query` to sort results.

## Labels and descriptions

Each capture can carry an array of string labels and an optional free-text description. Labels enable categorical filtering (e.g., `["ui-test", "dark-mode", "header"]`), while descriptions provide natural-language context for what the capture shows. Both are stored in `ObservationMeta` and are searchable through `vision_query` and `vision_evidence`.

## Embeddings and visual similarity

AgenticVision generates 512-dimensional embeddings using CLIP ViT-B/32 via ONNX Runtime. The preprocessing pipeline resizes images to 224x224 with Lanczos3, converts to RGB, and normalizes with CLIP mean/std values before inference. The resulting vector is L2-normalized.

Similarity search uses cosine distance computed in f64 precision for numerical stability. The `find_similar` function performs a linear scan over all observations, filters by a minimum similarity threshold, and returns the top-k results sorted by descending similarity. For 1,000 captures this completes in under 8 ms.

If no ONNX model is available at `~/.agentic-vision/models/clip-vit-base-patch32-visual.onnx`, the engine operates in fallback mode, producing zero-vector embeddings. Semantic similarity is disabled in this mode, but all other features (capture, diff, OCR, tracking) continue to work.

## Visual diff and change detection

The diff engine compares two captures at the pixel level. Both images are resized to the smaller common dimensions, converted to grayscale, and compared pixel-by-pixel. A pixel difference above a threshold of 30 (on a 0-255 scale) counts as "changed."

The output includes:

- **similarity**: 1.0 minus the ratio of changed pixels
- **pixel_diff_ratio**: Fraction of pixels that exceed the threshold
- **changed_regions**: Bounding boxes of changed areas, detected via 8x8 grid analysis with automatic merging of adjacent regions (minimum region size: 10 pixels)

## OCR

Text extraction from captures is available through `vision_ocr`. OCR performance depends on text density and image clarity. Typical extraction time is around 120 ms per capture.

## Tracking

`vision_track` configures a UI region for repeated capture. You specify a screen region (x, y, width, height), an interval in milliseconds (default 1000), a maximum capture count (default 100), and an on-change similarity threshold (default 0.95). When the similarity between consecutive frames drops below the threshold, the new frame is captured and stored. This is useful for monitoring dashboards, CI pipelines, or any visual element that changes over time.

## Visual grounding

Grounding prevents hallucination about what was visually observed. The `vision_ground` tool verifies a textual claim against stored captures by searching labels, descriptions, and metadata. It returns a status of `verified`, `partial`, or `ungrounded` depending on match quality. The `vision_evidence` tool provides detailed matching captures with timestamps and metadata, while `vision_suggest` offers alternatives when an exact match is not found.

## Multi-context workspaces

Workspaces allow loading multiple `.avis` files simultaneously for cross-project visual comparison. Operations include:

- `vision_workspace_create`: Create a named workspace
- `vision_workspace_add`: Load an `.avis` file with a role (primary, secondary, reference, archive)
- `vision_workspace_query`: Search across all loaded contexts
- `vision_workspace_compare`: Compare a visual element across contexts
- `vision_workspace_xref`: Find which contexts contain a given visual element

## MCP-first architecture

`agentic-vision-mcp` exposes tools, resources, and prompts over stdio MCP transport. The server supports the full MCP protocol including capability negotiation, tool listing, and resource templates. Any MCP client can consume the same interface without vendor-specific behavior.

The tool surface includes core operations (capture, query, similar, diff, compare, OCR, track, health, link), grounding operations (ground, evidence, suggest), workspace operations, and session lifecycle management.

## Cross-sister integration

Captures can be linked to AgenticMemory nodes via the `memory_link` field and the `vision_link` tool. This enables agents to associate a visual observation with a fact, decision, or episode in memory. The `vision_bind_code`, `vision_bind_memory`, `vision_bind_identity`, and `vision_bind_time` tools extend this to all Agentra sisters.
