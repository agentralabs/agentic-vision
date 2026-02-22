# Quickstart

## 1. Install

```bash
curl -fsSL https://agentralabs.tech/install/vision | bash
```

Profile-specific commands are listed in [Installation](installation.md).

## 2. Validate artifact path

```bash
agentic-vision-mcp -v ~/.vision.avis validate
```

## 3. Inspect capabilities

```bash
agentic-vision-mcp info
```

Expected tool list includes:

- `vision_capture`
- `vision_query`
- `vision_similar`
- `vision_compare`
- `vision_diff`
- `vision_ocr`
- `vision_link`
- `vision_health`

## 4. Start MCP server

```bash
$HOME/.local/bin/agentic-vision-mcp serve
```

Use `Ctrl+C` to stop after startup verification.

## 5. Query quality-aware results

Use MCP `vision_query` args:

```json
{
  "description_contains": "error",
  "min_quality": 0.5,
  "sort_by": "quality",
  "max_results": 10
}
```

Run `vision_health` periodically to monitor stale captures, unlabeled captures, and unlinked memory references.
