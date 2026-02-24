---
status: stable
---

# Runtime, Install Output, and Sync Contract

This page defines expected runtime behavior across installer output, CLI behavior, and web documentation.

## Installer profiles

- `desktop`: installs binary and merges detected desktop MCP config.
- `terminal`: installs binary without desktop-specific UX assumptions.
- `server`: installs binary without desktop config writes.

## Completion output contract

Installer must print:

1. Installed binary summary.
2. MCP restart instruction.
3. Server auth + artifact sync guidance when relevant.
4. Optional feedback instruction.

Expected completion marker:

```text
Install complete: AgenticVision (<profile>)
```

## Universal MCP config

```json
{
  "mcpServers": {
    "agentic-vision": {
      "command": "$HOME/.local/bin/agentic-vision-mcp",
      "args": ["--log-level", "error", "serve"]
    }
  }
}
```

## Workspace auto-detection behavior

- Installer writes `agentic-vision-mcp-agentra` launcher as MCP entrypoint.
- Launcher resolves vision artifact in this order:
1. Explicit override: `AGENTRA_AVIS_PATH` / `AGENTRA_VISION_PATH`.
2. Per-workspace default: `<workspace>/.agentra/<workspace-slug>.avis`.
3. Existing local fallbacks (`vision.avis`, `.vision.avis`, home default).
- If no file exists yet, launcher routes to per-workspace default path so first run creates and keeps project vision state isolated.
- Launcher enforces `--log-level error` for stdio MCP startup unless explicitly overridden, preventing stderr noise from breaking strict MCP handshakes.
- If your MCP client starts outside the project directory, set:

```bash
export AGENTRA_WORKSPACE_ROOT="/absolute/path/to/project"
```

## Server auth + sync

```bash
export AGENTIC_TOKEN="$(openssl rand -hex 32)"
```

Server deployments must sync `.avis/.amem/.acb` artifacts to server storage before runtime.

## Long-horizon storage budget policy

To target ~1-2 GB over long horizons (for example 20 years), configure:

```bash
export CORTEX_STORAGE_BUDGET_MODE=auto-rollup
export CORTEX_STORAGE_BUDGET_BYTES=2147483648
export CORTEX_STORAGE_BUDGET_HORIZON_YEARS=20
export CORTEX_STORAGE_BUDGET_TARGET_FRACTION=0.85
```

Modes:

- `auto-rollup`: prune oldest low-value captures from completed sessions when budget pressure appears.
- `warn`: emit warnings only.
- `off`: disable policy.
