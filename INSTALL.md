# Installation Guide

## Quick Install (one-liner)

```bash
curl -fsSL https://agentralabs.tech/install/vision | bash
```

Downloads release binaries, installs to `~/.local/bin/`, and merges MCP server config into Claude Desktop and Claude Code. Vision defaults to `~/.vision.avis`. Requires `curl` and `jq`.

### Install by environment

```bash
# Desktop MCP clients (auto-merge Claude configs)
curl -fsSL https://agentralabs.tech/install/vision/desktop | bash

# Terminal-only (no desktop config writes)
curl -fsSL https://agentralabs.tech/install/vision/terminal | bash

# Remote/server host (no desktop config writes)
curl -fsSL https://agentralabs.tech/install/vision/server | bash
```

### Server auth and artifact sync

Cloud/server runtime cannot read files from your laptop directly.

```bash
export AGENTIC_TOKEN="$(openssl rand -hex 32)"
```

All MCP clients must send `Authorization: Bearer <same-token>`.
If `.avis/.amem/.acb` artifacts were created elsewhere, sync them to the server first.

---

Three ways to install AgenticVision, depending on your use case.

---

## 1. MCP Server (recommended for most users)

The MCP server gives any MCP-compatible LLM client persistent visual memory. Requires **Rust 1.70+**.

```bash
cargo install agentic-vision-mcp
```

### Configure Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "vision": {
      "command": "agentic-vision-mcp",
      "args": ["--vision", "~/.vision.avis", "serve"]
    }
  }
}
```

### Configure VS Code / Cursor

Add to `.vscode/settings.json`:

```json
{
  "mcp.servers": {
    "agentic-vision": {
      "command": "agentic-vision-mcp",
      "args": ["--vision", "${workspaceFolder}/.vision/project.avis", "serve"]
    }
  }
}
```

### Configure Windsurf

Add to `~/.codeium/windsurf/mcp_config.json`:

```json
{
  "mcpServers": {
    "vision": {
      "command": "agentic-vision-mcp",
      "args": ["--vision", "~/.vision.avis", "serve"]
    }
  }
}
```

> **Do not use `/tmp` for vision files** — macOS and Linux clear this directory periodically. Use `~/.vision.avis` for persistent storage.

### CLI Commands

```bash
# Start MCP server (default)
agentic-vision-mcp --vision ~/.vision.avis serve

# Validate a vision file
agentic-vision-mcp --vision ~/.vision.avis validate

# Print server capabilities as JSON
agentic-vision-mcp info
```

### Verify

Once connected, the LLM gains access to tools like `vision_capture`, `vision_query`, `vision_similar`, `vision_compare`, `vision_diff`, and `vision_link`. Test by asking the LLM:

> "Capture a screenshot and describe what you see."

The LLM should call `vision_capture` and confirm the image was stored.

---

## 2. Core Library (for Rust projects)

The core library provides image capture, CLIP embedding, similarity search, and the `.avis` file format. Requires **Rust 1.70+**.

```bash
cargo install agentic-vision
```

### Use as a dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
agentic-vision = "0.1"
```

### Verify

```rust
use agentic_vision::VisionStore;

let store = VisionStore::open("test.avis")?;
println!("Captures: {}", store.count());
```

---

## 3. Combined with AgenticMemory

AgenticVision links to [AgenticMemory](https://github.com/agentralabs/agentic-memory) for full cognitive + visual agent memory. Run both MCP servers:

```json
{
  "mcpServers": {
    "memory": {
      "command": "agentic-memory-mcp",
      "args": ["--memory", "~/.brain.amem", "serve"]
    },
    "vision": {
      "command": "agentic-vision-mcp",
      "args": ["--vision", "~/.vision.avis", "serve"]
    }
  }
}
```

The `vision_link` tool bridges captures to memory nodes. An agent can associate what it *sees* with what it *knows*.

---

## 4. Remote Server (coming in v0.2.0)

> **Preview** — these features are under development. Track progress in [#2](https://github.com/agentralabs/agentic-vision/issues/2).

```bash
# Remote single-user
agentic-vision-mcp serve-http \
  --port 8081 \
  --token "secret123"

# Remote multi-tenant
agentic-vision-mcp serve-http \
  --multi-tenant \
  --data-dir /data/users/ \
  --port 8081 \
  --token "secret123"
```

Docker compose with Caddy reverse proxy will also be available. See the [v0.2.0 roadmap](https://github.com/agentralabs/agentic-vision/issues/2) for details.

---

## Build from Source

```bash
git clone https://github.com/agentralabs/agentic-vision.git
cd agentic-vision

# Build entire workspace (core library + MCP server)
cargo build --release

# Install core library
cargo install --path crates/agentic-vision

# Install MCP server
cargo install --path crates/agentic-vision-mcp
```

### CLIP Model (optional)

For full CLIP embedding support, place the ONNX model in the `models/` directory:

```bash
# The model is ~350 MB
models/clip-vit-b32-visual.onnx
```

Without the model, AgenticVision uses a deterministic fallback embedding (suitable for testing and development).

### Run tests

```bash
# All workspace tests (core + MCP: 38 tests)
cargo test --workspace

# Core library only
cargo test -p agentic-vision

# MCP server only
cargo test -p agentic-vision-mcp

# Python integration tests (requires release build)
cargo build --release
python tests/integration/test_mcp_clients.py
python tests/integration/test_multi_agent.py
```

---

## Package Registry Links

| Package | Registry | Install |
|:---|:---|:---|
| **agentic-vision** | [crates.io](https://crates.io/crates/agentic-vision) | `cargo install agentic-vision` |
| **agentic-vision-mcp** | [crates.io](https://crates.io/crates/agentic-vision-mcp) | `cargo install agentic-vision-mcp` |

---

## Requirements

| Component | Minimum version |
|:---|:---|
| Rust | 1.70+ (for building from source or `cargo install`) |
| OS | macOS, Linux |
| Python | 3.10+ (only for integration tests) |

---

## Troubleshooting

### `agentic-vision-mcp: command not found` after `cargo install`

Make sure `~/.cargo/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add this line to your `~/.zshrc` or `~/.bashrc` to make it permanent.

### CLIP model not found

AgenticVision works without the CLIP ONNX model — it falls back to a deterministic embedding function. For production use with real similarity search, download the CLIP ViT-B/32 visual ONNX model and place it in `models/clip-vit-b32-visual.onnx`.

### MCP server doesn't respond

Check that the binary is accessible:

```bash
which agentic-vision-mcp
agentic-vision-mcp serve --vision ~/.vision.avis
```

The server communicates via stdin/stdout (MCP stdio transport). If running manually, send a JSON-RPC initialize request to verify:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | agentic-vision-mcp serve --vision ~/.vision.avis
```

### macOS: "can't be opened because Apple cannot check it for malicious software"

```bash
xattr -d com.apple.quarantine $(which agentic-vision-mcp)
```

### Build fails with ONNX Runtime errors

The `ort` crate (ONNX Runtime bindings) requires a C++ compiler. On macOS, ensure Xcode Command Line Tools are installed:

```bash
xcode-select --install
```
