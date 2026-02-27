---
status: stable
---

# Configuration

AgenticVision configuration options for all runtime modes.

## Environment Variables

| Variable | Default | Allowed Values | Effect |
|----------|---------|----------------|--------|
| `AVIS_FILE` | Auto-detected | Path to `.avis` file | Explicit vision file path |
| `AGENTIC_TOKEN` | None | String | Bearer token for HTTP server authentication |
| `RUST_LOG` | `info` | `trace`, `debug`, `info`, `warn`, `error` | Logging verbosity (tracing-subscriber) |

## MCP Server Configuration

The MCP server (`agentic-vision-mcp`) accepts the following arguments:

### Stdio Mode (default)

```json
{
  "mcpServers": {
    "agentic-vision": {
      "command": "~/.local/bin/agentic-vision-mcp-agentra",
      "args": ["serve"]
    }
  }
}
```

### With Explicit Vision File

```json
{
  "mcpServers": {
    "agentic-vision": {
      "command": "~/.local/bin/agentic-vision-mcp-agentra",
      "args": ["serve", "--vision", "/path/to/project.avis"]
    }
  }
}
```

### CLI Arguments

| Argument | Description |
|----------|-------------|
| `--vision <path>` / `-v <path>` | Path to `.avis` vision file |
| `--model <path>` | Path to CLIP ONNX model |
| `--log-level <level>` | Log level: `trace`, `debug`, `info`, `warn`, `error` (default: `info`) |

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `serve` | Start MCP server over stdio (default) |
| `serve-http` | Start MCP server over HTTP (requires `sse` feature) |
| `validate` | Validate a `.avis` vision file |
| `info` | Print server capabilities as JSON |
| `completions <shell>` | Generate shell completion scripts |
| `repl` | Launch interactive REPL mode |

### HTTP Mode (with `sse` feature)

```bash
agentic-vision-mcp serve-http --addr 127.0.0.1:3100
```

| Argument | Description |
|----------|-------------|
| `--addr <host:port>` | Listen address (default: `127.0.0.1:3100`) |
| `--vision <path>` / `-v <path>` | Path to `.avis` file (single-user mode) |
| `--model <path>` | Path to CLIP ONNX model |
| `--token <token>` | Bearer token for authentication (also reads `AGENTIC_TOKEN` env var) |
| `--multi-tenant` | Enable multi-tenant mode (per-user vision files) |
| `--data-dir <dir>` | Data directory for multi-tenant files (each user gets `{dir}/{user-id}.avis`) |

## File Location Resolution

AgenticVision resolves the `.avis` file in this order:

1. Explicit `--vision` CLI argument (if set)
2. `AVIS_FILE` environment variable (if set)
3. `.avis/vision.avis` in the current working directory
4. `$HOME/.agentic-vision/vision.avis` (default fallback)

## Runtime Modes

| Mode | Trigger | Behavior |
|------|---------|----------|
| `stdio` | Default / `serve` | Full MCP server over stdio, auto-session |
| `http` | `serve-http` | HTTP/SSE transport, optional token auth |
| `multi-tenant` | `serve-http --multi-tenant` | Per-user `.avis` files, requires `--data-dir` |
| `repl` | `repl` | Interactive REPL for manual testing |
