---
status: stable
---

# Concepts

## Persistent visual memory

AgenticVision stores visual observations in `.avis` so agents can reuse them across sessions.

## Capture lifecycle

1. Capture image input.
2. Generate embedding.
3. Store metadata and thumbnail.
4. Query by time, description, or similarity.

## Artifact model

- `.avis` is portable, local-first storage.
- It is designed for fast read/query in agent workflows.

## MCP-first architecture

`agentic-vision-mcp` exposes tools/resources/prompts over stdio MCP.

Any MCP client can consume the same interface without vendor-specific behavior.
