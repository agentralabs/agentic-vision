# Command Surface (Canonical Sync Source)

This page is an authoritative command catalog for AgenticVision and is intended as a source file for web-doc synchronization.

## Install Commands

```bash
# Recommended one-liner
curl -fsSL https://agentralabs.tech/install/vision | bash

# Explicit profiles
curl -fsSL https://agentralabs.tech/install/vision/desktop | bash
curl -fsSL https://agentralabs.tech/install/vision/terminal | bash
curl -fsSL https://agentralabs.tech/install/vision/server | bash

# MCP binary install
cargo install agentic-vision-mcp

# Core library for embedding in your Rust project
cargo add agentic-vision
```

## Binary vs Library

- `agentic-vision-mcp` is the executable MCP server binary.
- `agentic-vision` is a library crate (added to Rust projects via `cargo add`).

## `agentic-vision-mcp` Commands

```bash
agentic-vision-mcp serve
agentic-vision-mcp validate
agentic-vision-mcp info
agentic-vision-mcp completions
agentic-vision-mcp repl
```

`serve` options include:

- `--vision <file.avis>`
- `--model <clip_model.onnx>`
- `--log-level trace|debug|info|warn|error`

## Universal MCP Entry (Any MCP Client)

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

## Default Vision Artifact

- Default shared vision path used by installer conventions: `~/.vision.avis`

## Verification Commands

```bash
# Binary checks
agentic-vision-mcp --version
agentic-vision-mcp --help

# Vision file checks
agentic-vision-mcp validate --vision ~/.vision.avis
agentic-vision-mcp info

# MCP startup check (Ctrl+C after startup)
$HOME/.local/bin/agentic-vision-mcp serve
```

## Artifact Contract

- Primary artifact: `.avis`
- For cross-sister server workflows, sync all required artifacts to server storage: `.avis`, `.amem`, `.acb`

## Publish Commands

```bash
# In repo root
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings

# Dry run (paired crates)
cargo publish -p agentic-vision --dry-run
cargo publish -p agentic-vision-mcp --dry-run

# Release (core first)
cargo publish -p agentic-vision
cargo publish -p agentic-vision-mcp
```

## Operator Notes

- Desktop/terminal profiles merge MCP config for detected clients.
- Server profile does not write desktop MCP config files.
- After install, restart MCP clients so new config is loaded.
- Optional feedback: https://github.com/agentralabs/agentic-vision/issues
