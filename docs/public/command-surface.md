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

## Key tools exposed by MCP

- `vision_capture` (now returns `quality_score`)
- `vision_query` (supports `description_contains`, `min_quality`, `sort_by`)
- `vision_health` (quality + staleness + linkage summary)
- `vision_similar`
- `vision_compare`
- `vision_diff`
- `vision_ocr`
- `vision_link`
- `session_start`
- `session_end`

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
