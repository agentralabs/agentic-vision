---
status: stable
---

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

## 6. Enable automatic long-horizon budget enforcement

```bash
export CORTEX_STORAGE_BUDGET_MODE=auto-rollup
export CORTEX_STORAGE_BUDGET_BYTES=2147483648
export CORTEX_STORAGE_BUDGET_HORIZON_YEARS=20
export CORTEX_STORAGE_BUDGET_TARGET_FRACTION=0.85
```

When enabled, runtime automatically prunes oldest low-value captures from completed sessions when budget pressure is detected.

## Validate capabilities

```bash
./scripts/test-primary-problems.sh
```

See [Experience With vs Without](experience-with-vs-without.md) for the full capability map.
