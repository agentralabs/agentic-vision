# Initial Problem Coverage (Vision)

This page records the **foundational problems AgenticVision already solved** before the newer primary-problem expansion.

## Reference set

| Ref | Initial problem solved | Shipped capability |
|---|---|---|
| IAV-I01 | No persistent visual evidence store | `.avis` artifact for capture history |
| IAV-I02 | No queryable visual history | `vision_query` with recency/quality filters |
| IAV-I03 | No structured visual comparisons | `vision_compare`, `vision_diff`, `vision_similar` |
| IAV-I04 | No visual quality diagnostics | `quality_score` + `vision_health` |
| IAV-I05 | No bridge from visual to cognitive memory | `vision_link` to memory nodes |
| IAV-I06 | No universal MCP visual runtime | `agentic-vision-mcp` tool surface |

## AgenticCodebase verification snapshot

Verification method used: AgenticCodebase scanning AgenticVision source.

```bash
acb -f json compile <agentic-vision-repo> -o /tmp/acb_vision_repo.acb --exclude target --exclude .git --include-tests
acb -f json info /tmp/acb_vision_repo.acb
acb -f json query /tmp/acb_vision_repo.acb symbol --name vision_capture
acb -f json query /tmp/acb_vision_repo.acb symbol --name vision_query
acb -f json query /tmp/acb_vision_repo.acb symbol --name vision_health
acb -f json query /tmp/acb_vision_repo.acb symbol --name vision_diff
```

Observed snapshot (2026-02-24):

- Units: `3492`
- Edges: `3371`
- Languages: `5`
- Compile status: `ok`
- Symbol evidence:
  - `vision_capture` module
  - `vision_query` module + param validation test
  - `vision_health` module
  - `vision_diff` module

## Status

All initial references `IAV-I01` to `IAV-I06` are implemented and actively testable from MCP/CLI surfaces.

## See also

- [Primary Problem Coverage](primary-problem-coverage.md)
- [Quickstart](quickstart.md)
