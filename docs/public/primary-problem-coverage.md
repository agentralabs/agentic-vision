# Primary Problem Coverage (Vision)

This page tracks direct coverage for Vision primary problems:

- P23 UI-state blindness
- P24 non-text signal blindness

## What is implemented now

Vision provides MCP tools for capture, query, comparison, diffing, and quality scoring. This phase adds an explicit regression entrypoint:

```bash
./scripts/test-primary-problems.sh
```

The script validates:

1. Tool surface includes visual-state primitives (`vision_capture`, `vision_query`, `vision_health`, `vision_diff`)
2. Parameter safety for capture/query (`edge_cases` targeted tests)
3. Non-text workflow guardrails (`vision_track` validation, empty-description handling)

## Problem-to-capability map

| Problem | Coverage primitive |
|---|---|
| P23 | `vision_capture`, `vision_query`, `vision_compare`, `vision_diff` |
| P24 | `quality_score`, `min_quality`, `sort_by=quality`, `vision_health` |

## Why this matters

Text logs alone miss what users actually saw. Vision makes visual state queryable and versioned so agent decisions can be grounded in real screen evidence, not only text traces.

## See also

- [Initial Problem Coverage](initial-problem-coverage.md)
