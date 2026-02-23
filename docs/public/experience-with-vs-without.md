# Why Teams Adopt AgenticVision

Simulation date: 2026-02-23

## Why this matters

Most production UI incidents are visual first and textual second.

If your team cannot query what users saw, incident response slows down and confidence drops.

## Core capabilities (simple language)

1. **Capture and store visual states**
   - Keep screenshot-level memory in `.avis` artifacts.
2. **Query and compare visual history**
   - Find similar captures and compute diffs quickly.
3. **Extract visible text (OCR)**
   - Pull text from UI captures for audit and analysis.
4. **Track quality and linkage health**
   - Use quality metrics and linkage to memory where needed.
5. **Control long-horizon storage growth**
   - Budget policy can prune low-value captures over time.

## Compelling scenario

A checkout button appears to "randomly disappear" in one environment.

Without AgenticVision:
- each person shares ad hoc screenshots and opinions.

With AgenticVision:
- captures are queryable,
- diffs are reproducible,
- OCR and similarity provide structured evidence.

That turns incident response into a process instead of a debate.

## With vs without (real simulation)

### Without

```bash
file <capture-or-store>
```

You can inspect metadata only. No visual-query pipeline exists.

### With

```bash
agentic-vision-mcp info
agentic-vision-mcp repl
# /tools
# /validate
```

Observed simulation output:
- MCP tool surface reported with `tool_count: 11`
- tools included `vision_capture`, `vision_query`, `vision_ocr`, `vision_diff`, `vision_health`, `vision_link`
- REPL exposed interactive runtime validation commands

## Numbers that make it real

From current docs/benchmarks:
- capture pipeline (file -> embed -> store): about **47 ms** typical
- similarity search top-5: about **1-2 ms**
- visual diff: sub-millisecond class
- MCP tool round-trip: around **7 ms** typical

## Long-horizon retention and tradeoffs

Budget policy can target ~1-2 GB over long horizons using:
- `CORTEX_STORAGE_BUDGET_MODE=auto-rollup`
- `CORTEX_STORAGE_BUDGET_BYTES`
- `CORTEX_STORAGE_BUDGET_HORIZON_YEARS`
- `CORTEX_STORAGE_BUDGET_TARGET_FRACTION`

Tradeoffs to understand:
- higher capture frequency and larger images increase growth faster
- OCR quality depends on source image quality
- cross-host workflows require explicit artifact sync (`.avis/.amem/.acb`)

## What this means for technical readers

- You get scriptable visual operations, not manual screenshot threads.
- You can standardize visual regression and incident analysis workflows.
- You can manage storage pressure without deleting all history blindly.

## What this means for non-technical readers

- Faster root-cause understanding for UI issues.
- Easier communication with before/after visual evidence.
- Less ambiguity in postmortems.

## Multi-LLM fit

Claude, Gemini, OpenAI/Codex, Cursor, VS Code, and Windsurf teams can consume the same MCP visual capability surface.

## Start in 5 minutes

```bash
agentic-vision-mcp info
agentic-vision-mcp repl
```

Success signal:
- your team can list tools, validate runtime state, and run visual workflows from one interface.
