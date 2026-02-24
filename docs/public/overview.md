---
status: stable
---

# AgenticVision Overview

AgenticVision provides persistent visual memory for agents through a portable `.avis` artifact and an MCP server.

## What it does

- Captures images from files, base64 payloads, screenshots, and clipboard.
- Stores embeddings and metadata in `.avis`.
- Supports visual search, comparison, diff, OCR, and memory linking.
- Exposes tools to any MCP-compatible client through `agentic-vision-mcp`.

## Why teams adopt AgenticVision

Teams adopt AgenticVision because it closes both legacy and current visual-intelligence gaps:

- Foundational problems already solved: no persistent visual evidence store, no queryable visual history, no structured visual comparison/diff workflow, no visual quality diagnostics, no vision-to-memory bridge, and no universal MCP visual runtime.
- New high-scale problems now solved: UI-state blindness during automation and non-text signal blindness in decision workflows.
- Practical outcome for teams: decisions are grounded in what the system actually saw, not only text logs, with portable evidence that can be queried later.

For full reference mapping, see:

- [Initial Problem Coverage](initial-problem-coverage.md)
- [Primary Problem Coverage](primary-problem-coverage.md)

## Artifact

- Primary artifact: `.avis`
- Cross-sister server workflows can pair `.avis` with `.amem` and `.acb`

## Start here

- [Installation](installation.md)
- [Quickstart](quickstart.md)
- [Command Surface](command-surface.md)
- [Runtime and Sync](runtime-install-sync.md)
- [Integration Guide](integration-guide.md)
- [Initial Problem Coverage](initial-problem-coverage.md)
- [Primary Problem Coverage](primary-problem-coverage.md)
