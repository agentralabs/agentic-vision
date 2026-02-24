---
status: stable
---

# API Reference

AgenticVision exposes its capabilities through MCP tools, resources, and prompts. Run `agentic-vision-mcp info` to verify tool discovery.

## MCP Tools

### vision_capture

Capture an image and store it in visual memory.

| Parameter | Type | Required | Default | Description |
|:--|:--|:-:|:--|:--|
| source | object | yes | — | Image source: `type` is `file`, `base64`, `screenshot`, or `clipboard`. Include `path` for file, `data`+`mime` for base64, optional `region` for screenshot. |
| description | string | no | — | Human-readable label for the capture. |
| labels | string[] | no | [] | Tags for filtering and organization. |
| extract_ocr | boolean | no | false | Run OCR on the captured image. |

Returns: `{ capture_id, timestamp, dimensions, quality_score }`

### vision_query

Search captures by filters.

| Parameter | Type | Required | Default | Description |
|:--|:--|:-:|:--|:--|
| description_contains | string | no | — | Substring match on capture descriptions. |
| labels | string[] | no | — | Filter by label tags. |
| min_quality | number | no | — | Minimum quality score (0.0-1.0). |
| sort_by | string | no | "recent" | Sort order: `recent` or `quality`. |
| max_results | number | no | 20 | Maximum captures to return. |
| before | number | no | — | Unix timestamp upper bound. |
| after | number | no | — | Unix timestamp lower bound. |
| session_ids | number[] | no | — | Filter by session. |

Returns: array of capture metadata objects.

### vision_similar

Find visually similar captures using CLIP embedding distance.

| Parameter | Type | Required | Default | Description |
|:--|:--|:-:|:--|:--|
| capture_id | number | no | — | Find captures similar to this one. |
| embedding | number[] | no | — | Or provide a raw embedding vector. |
| top_k | number | no | 10 | Number of results. |
| min_similarity | number | no | 0.5 | Minimum cosine similarity threshold. |
| event_types | string[] | no | — | Filter by event type. |

Returns: array of `{ capture_id, similarity_score, metadata }`.

### vision_compare

Compare two captures for visual similarity.

| Parameter | Type | Required | Default | Description |
|:--|:--|:-:|:--|:--|
| id_a | number | yes | — | First capture ID. |
| id_b | number | yes | — | Second capture ID. |
| detailed | boolean | no | false | Include detailed diff data. |

Returns: `{ similarity_score, dimensions_match, summary }`.

### vision_diff

Pixel-level diff between two captures.

| Parameter | Type | Required | Default | Description |
|:--|:--|:-:|:--|:--|
| id_a | number | yes | — | First capture ID. |
| id_b | number | yes | — | Second capture ID. |

Returns: `{ changed_pixel_count, total_pixels, change_percentage, bounding_boxes }`.

### vision_ocr

Extract text from a capture using OCR.

| Parameter | Type | Required | Default | Description |
|:--|:--|:-:|:--|:--|
| capture_id | number | yes | — | Capture to extract text from. |
| language | string | no | "eng" | OCR language code. |

Returns: `{ text, confidence, regions }`.

### vision_track

Configure tracking for a UI region. Captures must be triggered externally.

| Parameter | Type | Required | Default | Description |
|:--|:--|:-:|:--|:--|
| region | object | yes | — | `{ x, y, w, h }` in pixels. |
| interval_ms | number | no | 1000 | Minimum interval between captures. |
| max_captures | number | no | 100 | Stop after this many captures. |
| on_change_threshold | number | no | 0.95 | Similarity threshold; below this counts as a change. |

Returns: `{ track_id, region, status }`.

### vision_link

Link a visual capture to an AgenticMemory node.

| Parameter | Type | Required | Default | Description |
|:--|:--|:-:|:--|:--|
| capture_id | number | yes | — | Capture to link. |
| memory_node_id | number | yes | — | Target memory node ID. |
| relationship | string | no | "observed_during" | One of: `observed_during`, `evidence_for`, `screenshot_of`. |

Returns: `{ link_id, capture_id, memory_node_id, relationship }`.

### vision_health

Evaluate visual memory reliability.

| Parameter | Type | Required | Default | Description |
|:--|:--|:-:|:--|:--|
| low_quality_threshold | number | no | 0.45 | Below this is flagged as low quality. |
| stale_after_hours | number | no | 168 | Hours before a capture is considered stale. |
| max_examples | number | no | 20 | Max examples per category. |

Returns: `{ total_captures, low_quality, stale, unlinked, unlabeled }`.

### session_start

Start a new vision session.

| Parameter | Type | Required | Default | Description |
|:--|:--|:-:|:--|:--|
| session_id | number | no | auto | Explicit session ID. |

Returns: `{ session_id }`.

### session_end

End the current vision session and persist.

Returns: `{ session_id, capture_count }`.

## Quality score

Every capture receives a quality score (0.0-1.0) computed from:

- **Resolution**: higher resolution scores higher.
- **Embedding confidence**: CLIP embedding norm.
- **Metadata completeness**: presence of description and labels.
- **OCR yield**: text extraction success rate (if OCR was requested).

## CLI subcommands

```bash
agentic-vision-mcp serve       # Start MCP server
agentic-vision-mcp validate    # Validate artifact path
agentic-vision-mcp info        # List available tools and capabilities
agentic-vision-mcp completions # Shell completions
agentic-vision-mcp repl        # Interactive REPL
```

## See also

- [Quickstart](quickstart.md)
- [Concepts](concepts.md)
- [Benchmarks](benchmarks.md)
