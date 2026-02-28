---
status: stable
---

# Command Surface

Install commands are documented in [Installation](installation.md).

## Binary

- `agentic-vision-mcp`

## Top-level commands

```bash
agentic-vision-mcp serve
agentic-vision-mcp validate
agentic-vision-mcp info
agentic-vision-mcp completions
agentic-vision-mcp repl
```

## Common options

- `-v, --vision <file.avis>`
- `--model <clip_model.onnx>`
- `--log-level trace|debug|info|warn|error`

## Runtime budget controls

```bash
export CORTEX_STORAGE_BUDGET_MODE=auto-rollup
export CORTEX_STORAGE_BUDGET_BYTES=2147483648
export CORTEX_STORAGE_BUDGET_HORIZON_YEARS=20
export CORTEX_STORAGE_BUDGET_TARGET_FRACTION=0.85
```

## MCP Tools

All tools exposed by the `agentic-vision-mcp` MCP server:

### Core Tools

| Tool | Purpose |
|------|---------|
| `vision_capture` | Capture an image and store it in visual memory (returns `quality_score`) |
| `vision_query` | Search visual memory by filters (`description_contains`, `min_quality`, `sort_by`) |
| `vision_compare` | Compare two captures for visual similarity |
| `vision_diff` | Get detailed pixel-level diff between two captures |
| `vision_similar` | Find visually similar captures by embedding |
| `vision_ocr` | Extract text from a capture using OCR |
| `vision_track` | Configure tracking for a UI region |
| `vision_health` | Quality + staleness + linkage summary |
| `vision_link` | Link a visual capture to an AgenticMemory node |

### Context Capture Tools

| Tool | Purpose |
|------|---------|
| `observation_log` | Log observation context and auto-capture visual interactions |

### Grounding Tools (v0.2)

| Tool | Purpose |
|------|---------|
| `vision_ground` | Verify a visual claim has capture backing |
| `vision_evidence` | Get evidence for a visual claim from stored captures |
| `vision_suggest` | Find similar captures when a claim doesn't match exactly |

### Workspace Tools (v0.2)

| Tool | Purpose |
|------|---------|
| `vision_workspace_create` | Create a multi-vision workspace |
| `vision_workspace_add` | Add a .avis file to a workspace |
| `vision_workspace_list` | List loaded vision files in a workspace |
| `vision_workspace_query` | Query across all loaded vision files |
| `vision_workspace_compare` | Compare across sites and time periods |
| `vision_workspace_xref` | Find visual topic distribution across contexts |

### Session Tools

| Tool | Purpose |
|------|---------|
| `session_start` | Start a new vision session |
| `session_end` | End the current vision session and save |
| `vision_session_resume` | Load context from previous vision sessions |

### Compact Facade Tools (v0.3+)

Use these to keep MCP tool surfaces small while preserving backward compatibility:

| Tool | Purpose |
|------|---------|
| `vision_core` | Unified core operations via `operation` |
| `vision_grounding` | Unified grounding operations via `operation` |
| `vision_workspace` | Unified workspace operations via `operation` |
| `vision_session` | Unified session operations via `operation` |
| `vision_temporal` | Unified temporal operations via `operation` |
| `vision_prediction` | Unified prediction operations via `operation` |
| `vision_cognition` | Unified cognition operations via `operation` |
| `vision_synthesis` | Unified synthesis operations via `operation` |
| `vision_forensics` | Unified forensics operations via `operation` |

Compact list mode:

```bash
export AVIS_MCP_TOOL_SURFACE=compact
```

In compact mode, `tools/list` returns only the 9 facade tools above, while all legacy tool names remain callable.

### Advanced Tools

| Tool | Purpose |
|------|---------|
| `vision_at_time` | Get visual state at a specific time |
| `vision_cite` | Get citation for a visual element from captures |
| `vision_prophecy` | Predict visual impact of a proposed change |
| `vision_reason` | Build a reasoning chain from visual observations |
| `vision_timeline` | Get visual timeline for an element or page |

## Universal MCP entry

```json
{
  "mcpServers": {
    "agentic-vision": {
      "command": "$HOME/.local/bin/agentic-vision-mcp",
      "args": ["serve"]
    }
  }
}
```
