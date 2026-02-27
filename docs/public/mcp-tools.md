---
status: stable
---

# MCP Tools

AgenticVision exposes 69 tools through the MCP protocol via `agentic-vision-mcp`: 21 core tools and 48 V3 Perception Invention tools.

## Core Vision Tools

### `vision_capture`

Capture an image and store it in visual memory.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `source` | object | Yes | Image source descriptor |
| `source.type` | string | Yes | `file`, `base64`, `screenshot`, or `clipboard` |
| `source.path` | string | Conditional | File path (required for `type=file`) |
| `source.data` | string | Conditional | Base64 data (required for `type=base64`) |
| `source.mime` | string | No | MIME type (for `type=base64`) |
| `source.region` | object | No | Screen region (for `type=screenshot`): `{x, y, w, h}` |
| `extract_ocr` | boolean | No | Extract text via OCR (default: false) |
| `description` | string | No | Human-readable description |
| `labels` | array | No | List of string labels |

**Returns:** `{ "capture_id": 1, "timestamp": 1709000000, "dimensions": {...}, "embedding_dims": 512, "quality_score": 0.85 }`

### `vision_compare`

Compare two captures for visual similarity.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id_a` | number | Yes | First capture ID |
| `id_b` | number | Yes | Second capture ID |
| `detailed` | boolean | No | Include detailed diff (default: false) |

**Returns:** `{ "similarity": 0.92, "is_same": false }`

### `vision_query`

Search visual memory by filters.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_ids` | array | No | Filter by session IDs |
| `after` | number | No | Unix timestamp (lower bound) |
| `before` | number | No | Unix timestamp (upper bound) |
| `labels` | array | No | Filter by labels |
| `description_contains` | string | No | Substring match on description |
| `min_quality` | number | No | Minimum quality score [0.0, 1.0] |
| `sort_by` | string | No | `recent` or `quality` (default: `recent`) |
| `max_results` | number | No | Maximum results (default: 20) |

### `vision_ocr`

Extract text from a capture using OCR (requires `--features ocr` in v0.2.0).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `capture_id` | number | Yes | Capture ID to extract text from |
| `language` | string | No | OCR language (default: `eng`) |

### `vision_similar`

Find visually similar captures by embedding.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `capture_id` | number | No | Find similar to this capture |
| `embedding` | array | No | Or provide embedding vector directly |
| `top_k` | number | No | Maximum results (default: 10) |
| `min_similarity` | number | No | Minimum similarity [0.0, 1.0] (default: 0.7) |

Provide exactly one of `capture_id` or `embedding`.

### `vision_track`

Configure tracking for a UI region (captures must be triggered externally).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `region` | object | Yes | Screen region: `{x, y, w, h}` |
| `interval_ms` | number | No | Capture interval in milliseconds (default: 1000) |
| `on_change_threshold` | number | No | Similarity threshold for change detection (default: 0.95) |
| `max_captures` | number | No | Maximum captures to store (default: 100) |

### `vision_diff`

Get detailed pixel-level diff between two captures.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id_a` | number | Yes | First capture ID |
| `id_b` | number | Yes | Second capture ID |

**Returns:** `{ "before_id": 1, "after_id": 2, "similarity": 0.85, "pixel_diff_ratio": 0.12, "changed_regions": [...] }`

### `vision_health`

Summarize visual memory quality, staleness, and linkage coverage.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `stale_after_hours` | number | No | Hours before a capture is stale (default: 168) |
| `low_quality_threshold` | number | No | Quality score threshold (default: 0.45) |
| `max_examples` | number | No | Max example IDs per category (default: 20) |

**Returns:** `{ "status": "pass|warn|fail", "summary": {...}, "examples": {...} }`

### `vision_link`

Link a visual capture to an AgenticMemory node.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `capture_id` | number | Yes | Visual capture ID |
| `memory_node_id` | number | Yes | AgenticMemory node ID |
| `relationship` | string | No | `observed_during`, `evidence_for`, or `screenshot_of` (default: `observed_during`) |

## Grounding Tools (Anti-Hallucination)

### `vision_ground`

Verify a visual claim has capture backing. Prevents hallucination about what was seen.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `claim` | string | Yes | The visual claim to verify against stored captures |

**Returns:** `{ "status": "verified|ungrounded", "claim": "...", "confidence": 0.8, "evidence": [...] }`

### `vision_evidence`

Get detailed capture evidence for a visual claim.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | The query to search evidence for |
| `max_results` | number | No | Maximum evidence items (default: 10) |

### `vision_suggest`

Find similar captures when a visual claim does not match exactly.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | The query to find suggestions for |
| `limit` | number | No | Maximum suggestions (default: 5) |

## Workspace Tools (Multi-Context)

### `vision_workspace_create`

Create a multi-vision workspace for loading and querying multiple `.avis` files.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Name for the workspace |

### `vision_workspace_add`

Add an `.avis` file to a vision workspace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace ID |
| `path` | string | Yes | Path to `.avis` file |
| `role` | string | No | `primary`, `secondary`, `reference`, or `archive` (default: `primary`) |
| `label` | string | No | Human-readable label |

### `vision_workspace_list`

List all loaded vision contexts in a workspace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace ID |

### `vision_workspace_query`

Search across all vision contexts in a workspace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace ID |
| `query` | string | Yes | Search query |
| `max_per_context` | number | No | Maximum matches per context (default: 10) |

### `vision_workspace_compare`

Compare a visual element across contexts.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace ID |
| `item` | string | Yes | Topic or element to compare |
| `max_per_context` | number | No | Maximum matches per context (default: 5) |

### `vision_workspace_xref`

Find which vision contexts contain a visual element.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace ID |
| `item` | string | Yes | Topic or element to cross-reference |

## Session Tools

### `session_start`

Start a new vision session.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | number | No | Optional explicit session ID |

### `session_end`

End the current vision session and save.

No parameters.

## Context Capture Tool

### `observation_log`

Log the intent and context behind a visual observation. Entries are linked into the session's temporal chain.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `intent` | string | Yes | Why you are observing -- the goal or reason |
| `observation` | string | No | What you noticed or concluded |
| `related_capture_id` | number | No | Capture ID this observation relates to |
| `topic` | string | No | Category (e.g., `ui-testing`, `layout-check`) |

## V3 Perception Invention Tools

AgenticVision V3 adds 48 tools organized into 16 inventions across 4 categories.

### Grounding Inventions (1--4)

| Tool | Description |
|------|-------------|
| `vision_ground_claim` | Ground a specific visual claim with citation |
| `vision_verify_claim` | Verify a visual claim with detailed evidence chain |
| `vision_cite` | Generate a citation for a visual observation |
| `vision_contradict` | Find evidence that contradicts a visual claim |
| `vision_hallucination_check` | Check a statement for visual hallucination |
| `vision_hallucination_fix` | Suggest corrections for hallucinated visual claims |
| `vision_truth_check` | Check if a visual truth still holds |
| `vision_truth_refresh` | Refresh a visual truth with new evidence |
| `vision_truth_history` | Get the history of a visual truth |
| `vision_compare_contexts` | Compare visual state across different contexts |
| `vision_compare_sites` | Compare visual appearance across sites |
| `vision_compare_versions` | Compare visual appearance across versions |
| `vision_compare_devices` | Compare visual appearance across devices |

### Temporal Inventions (5--8)

| Tool | Description |
|------|-------------|
| `vision_at_time` | Retrieve visual state at a specific timestamp |
| `vision_timeline` | Generate a visual timeline of changes |
| `vision_reconstruct` | Reconstruct visual state from historical captures |
| `vision_archaeology_dig` | Excavate historical visual layers |
| `vision_archaeology_reconstruct` | Reconstruct deleted or overwritten visual state |
| `vision_archaeology_report` | Generate an archaeology report of visual history |
| `vision_consolidate` | Consolidate redundant visual captures |
| `vision_consolidate_preview` | Preview what consolidation would remove |
| `vision_consolidate_policy` | Configure automatic consolidation policy |
| `vision_dejavu_check` | Check if a visual pattern has been seen before |
| `vision_dejavu_patterns` | List recurring visual patterns |
| `vision_dejavu_alert` | Configure alerts for recurring visual patterns |

### Prediction Inventions (9--12)

| Tool | Description |
|------|-------------|
| `vision_prophecy` | Predict future visual state based on trends |
| `vision_prophecy_diff` | Diff between predicted and actual visual state |
| `vision_prophecy_compare` | Compare multiple prophecy outcomes |
| `vision_regression_predict` | Predict visual regressions from code changes |
| `vision_regression_test` | Test for visual regressions |
| `vision_regression_history` | Get history of visual regressions |
| `vision_attention_predict` | Predict where users will look |
| `vision_attention_optimize` | Suggest attention optimization |
| `vision_attention_compare` | Compare attention maps across versions |
| `vision_phantom_create` | Create a phantom (hypothetical) visual capture |
| `vision_phantom_compare` | Compare phantom with real captures |
| `vision_phantom_ab_test` | A/B test phantom visual variations |

### Cognition Inventions (13--16)

| Tool | Description |
|------|-------------|
| `vision_semantic_analyze` | Analyze semantic meaning of visual content |
| `vision_semantic_find` | Find captures by semantic meaning |
| `vision_semantic_intent` | Infer user intent from visual content |
| `vision_reason` | Reason about visual content |
| `vision_reason_about` | Reason about relationships between captures |
| `vision_reason_diagnose` | Diagnose visual issues |
| `vision_bind_code` | Bind a visual capture to a codebase symbol |
| `vision_bind_memory` | Bind a visual capture to a memory node |
| `vision_bind_identity` | Bind a visual capture to an identity record |
| `vision_bind_time` | Bind a visual capture to a time entry |
| `vision_traverse_binding` | Traverse cross-modal bindings from a capture |
| `vision_gestalt_analyze` | Analyze overall visual gestalt (harmony, balance) |
| `vision_gestalt_harmony` | Score visual harmony of a capture |
| `vision_gestalt_improve` | Suggest visual improvements based on gestalt principles |
