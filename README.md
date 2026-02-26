<p align="center">
  <img src="assets/github-hero-pane.svg" alt="AgenticVision hero pane" width="980">
</p>

<p align="center">
  <a href="https://crates.io/crates/agentic-vision"><img src="https://img.shields.io/badge/cargo_install-agentic--vision-F59E0B?style=for-the-badge&logo=rust&logoColor=white" alt="cargo install"></a>
  <a href="https://crates.io/crates/agentic-vision-mcp"><img src="https://img.shields.io/badge/cargo_install-agentic--vision--mcp-3B82F6?style=for-the-badge&logo=rust&logoColor=white" alt="cargo install mcp"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-8B5CF6?style=for-the-badge" alt="MIT License"></a>
  <a href="publication/paper-ii-agentic-vision-mcp/agentic-vision-mcp-paper.pdf"><img src="https://img.shields.io/badge/Research-Paper-EF4444?style=for-the-badge" alt="Research Paper"></a>
  <a href="docs/api-reference.md"><img src="https://img.shields.io/badge/format-.avis-3B82F6?style=for-the-badge" alt=".avis format"></a>
</p>

<p align="center">
  <a href="#quickstart">Quickstart</a> · <a href="#problems-solved">Problems Solved</a> · <a href="#why-agenticvision">Why</a> · <a href="#benchmarks">Benchmarks</a> · <a href="#how-it-works">How It Works</a> · <a href="#install">Install</a> · <a href="INSTALL.md">Full Install Guide</a> · <a href="publication/paper-ii-agentic-vision-mcp/agentic-vision-mcp-paper.pdf">Paper</a>
</p>

---

## AI agents can't see across sessions.

Your agent takes a screenshot, analyzes it, and forgets. Next session — blank slate. It can't compare what a page looks like now versus yesterday. It can't recall what the error dialog said three conversations ago. It can't search its own visual history.

Text-based memory exists. Visual memory doesn't — until now.

**AgenticVision** gives AI agents persistent visual memory. Capture images, embed them with CLIP ViT-B/32, store them in a compact binary format, and query them by similarity, time, or description. Every capture is a first-class MCP resource that any LLM can access.

<a name="problems-solved"></a>

## Problems Solved (Read This First)

- **Problem:** agents cannot remember what they saw last session.  
  **Solved:** `.avis` keeps persistent visual history across sessions and model changes.
- **Problem:** visual regressions are noticed late or missed.  
  **Solved:** built-in compare and diff workflows surface change quickly.
- **Problem:** screenshots pile up with no searchable structure.  
  **Solved:** each capture is embedded, timestamped, and queryable by similarity and metadata.
- **Problem:** image context stays trapped in one tool.  
  **Solved:** MCP tools/resources expose visual memory to any compatible client.
- **Problem:** what an agent sees is disconnected from what it remembers.  
  **Solved:** memory linking connects visual captures directly to cognitive graph nodes.

```bash
cargo install agentic-vision-cli agentic-vision-mcp
```

CLI + MCP binaries. 21 MCP tools. Persistent `.avis` files. Works with Claude Desktop, VS Code, Cursor, Windsurf, and any MCP-compatible client.

<p align="center">
  <img src="assets/github-terminal-pane.svg" alt="AgenticVision terminal pane" width="980">
</p>

---

<a name="benchmarks"></a>

## Benchmarks

<p align="center">
  <img src="assets/benchmark-chart.svg" alt="Performance benchmarks" width="800">
</p>

Rust core. CLIP ViT-B/32 via ONNX Runtime. Binary `.avis` format. Real numbers from `cargo test --release`:

| Operation | Time | Notes |
|:---|---:|:---|
| Image capture (file → embed → store) | **47 ms** | CLIP ViT-B/32, 512-dim |
| Similarity search (top-5) | **1-2 ms** | Brute-force cosine, f64 precision |
| Visual diff (pixel-level) | **<1 ms** | 8×8 grid region detection |
| MCP tool round-trip | **7.2 ms** | Including process startup (~6.1 ms) |
| Storage per capture | **~4.26 KB** | Embedding + JPEG thumbnail |
| Capacity per GB | **~250K** | Observations |

> All benchmarks on Apple M4, macOS 26.2, Rust 1.90.0 `--release`. ONNX Runtime for CLIP inference. Fallback mode available when ONNX model is not present.

<p align="center">
  <img src="assets/vision-runtime-flow-agentra.svg" alt="AgenticVision runtime flow from capture to embedding, storage, query, and MCP response" width="980">
</p>

---

<a name="why-agenticvision"></a>

## Why AgenticVision

**Agents need visual continuity.** A debugging agent should remember what the UI looked like before and after a code change. A monitoring agent should detect visual regressions. A research agent should build a visual knowledge base over time.

**Capture once, query forever.** Every image is embedded into a 512-dimensional CLIP vector and stored with its JPEG thumbnail, timestamp, and description. Query by cosine similarity, time range, or text search — in milliseconds.

**Binary format, not a database.** The `.avis` file is a single portable binary — 64-byte header, JSON payload, JPEG thumbnails. Copy it, share it, back it up. No server, no database, no dependencies.

**Works with every MCP client.** AgenticVision-MCP exposes 21 tools, 6 resources, and 4 prompts via the Model Context Protocol. Any LLM that speaks MCP gains visual memory automatically.

**Links to AgenticMemory.** The `vision_link` tool connects visual captures to [AgenticMemory](https://github.com/agentralabs/agentic-memory) cognitive graph nodes — bridging what an agent *sees* with what it *knows*.

---

## Ghost Writer

> **New in v0.2.4** -- Auto-syncs visual context to your AI coding tools every 5 seconds.

| Client | Config Location | Status |
|:---|:---|:---|
| **Claude Code** | `~/.claude/memory/VISION_CONTEXT.md` | Full support |
| **Cursor** | `~/.cursor/memory/agentic-vision.md` | Full support |
| **Windsurf** | `~/.windsurf/memory/agentic-vision.md` | Full support |
| **Cody** | `~/.sourcegraph/cody/memory/agentic-vision.md` | Full support |

Syncs: recent captures, observations, visual tool calls. **Zero configuration.** Context survives sessions automatically.

## MCP Hardening

> **New in v0.2.5** -- Production-grade stdio transport.

- Content-Length framing with 8 MiB limit
- JSON-RPC 2.0 validation
- Atomic writes (temp + rename + fsync)
- No silent fallbacks

---

<a name="how-it-works"></a>

## How It Works

<p align="center">
  <img src="assets/architecture-agentra-v2.svg" alt="AgenticVision architecture map with MCP clients, transport, tools, resources, prompts, and storage" width="980">
</p>

1. **Capture** — `vision_capture` accepts images from files, base64, screenshots, or the system clipboard. Each image is resized, embedded via CLIP ViT-B/32 into a 512-dimensional vector, compressed to JPEG thumbnail, and stored in the `.avis` binary file. Screenshots support optional region capture; clipboard reads the current image from the OS clipboard.

2. **Query** — `vision_query` retrieves captures by time range, description, recency, and quality constraints (`min_quality`, `sort_by`). Results include capture metadata, quality scores, thumbnails, and similarity scores.

3. **Compare** — `vision_compare` places two captures side-by-side for LLM analysis. `vision_diff` performs pixel-level differencing with 8×8 grid region detection to identify exactly what changed.

4. **Link** — `vision_link` connects captures to AgenticMemory nodes, bridging visual observations with the agent's cognitive graph. An agent can recall "what did the UI look like when I made that decision?"

**The `.avis` binary format** uses a 64-byte fixed header (magic `0x41564953`, version, counts, timestamps) followed by a JSON payload containing captures with embedded JPEG thumbnails and 512-dim float vectors. Single-file, portable, no external dependencies.

<details>
<summary><strong>MCP surface area</strong></summary>

<br>

**21 Tools** (core 11 + grounding 3 + workspace 5 + observation 1 + session 1):

| Tool | Description |
|:---|:---|
| `vision_capture` | Capture and embed an image (file, base64, screenshot, clipboard), with metadata redaction and quality scoring |
| `vision_compare` | Side-by-side comparison of two captures |
| `vision_query` | Query captures by time, description, recency |
| `vision_ocr` | Extract text from a captured image |
| `vision_similar` | Find visually similar captures (cosine similarity) |
| `vision_track` | Track visual changes to a target over time |
| `vision_diff` | Pixel-level diff between two captures |
| `vision_health` | Quality + staleness + memory-link coverage summary |
| `vision_link` | Link a capture to an AgenticMemory node |
| `session_start` | Begin a named observation session |
| `session_end` | End the current session |

**6 Resources:**

| URI | Description |
|:---|:---|
| `avis://capture/{id}` | Single capture with metadata and thumbnail |
| `avis://session/{id}` | All captures in a session |
| `avis://timeline/{start}/{end}` | Captures within a time range |
| `avis://similar/{id}` | Visually similar captures |
| `avis://stats` | Storage statistics and counts |
| `avis://recent` | Most recent captures |

**4 Prompts:**

| Prompt | Description |
|:---|:---|
| `observe` | Guided visual observation workflow |
| `compare` | Structured comparison between captures |
| `track` | Change tracking over time |
| `describe` | Detailed image description |

</details>

---

<a name="install"></a>

## Install

**One-liner** (desktop profile, backwards-compatible):
```bash
curl -fsSL https://agentralabs.tech/install/vision | bash
```

**Environment profiles** (one command per environment):
```bash
# Desktop MCP clients (auto-merge Claude Desktop + Claude Code when detected)
curl -fsSL https://agentralabs.tech/install/vision/desktop | bash

# Terminal-only (no desktop config writes)
curl -fsSL https://agentralabs.tech/install/vision/terminal | bash

# Remote/server hosts (no desktop config writes)
curl -fsSL https://agentralabs.tech/install/vision/server | bash
```

| Channel | Command | Result |
|:---|:---|:---|
| GitHub installer (official) | `curl -fsSL https://agentralabs.tech/install/vision \| bash` | Installs release binaries when available, otherwise source fallback; merges MCP config |
| GitHub installer (desktop profile) | `curl -fsSL https://agentralabs.tech/install/vision/desktop \| bash` | Explicit desktop profile behavior |
| GitHub installer (terminal profile) | `curl -fsSL https://agentralabs.tech/install/vision/terminal \| bash` | Installs binaries only; no desktop config writes |
| GitHub installer (server profile) | `curl -fsSL https://agentralabs.tech/install/vision/server \| bash` | Installs binaries only; server-safe behavior |
| crates.io + Cargo deps (official) | `cargo install agentic-vision-cli agentic-vision-mcp` + `cargo add agentic-vision` | Installs `avis`, MCP server binary, and adds the core library crate to your project |
| npm (wasm) | `npm install @agenticamem/vision` | WASM-based vision SDK for Node.js and browser |

### Server auth and artifact sync

For cloud/server runtime:

```bash
export AGENTIC_TOKEN="$(openssl rand -hex 32)"
```

All MCP clients must send `Authorization: Bearer <same-token>`.
If `.avis/.amem/.acb` files are on another machine, sync them to the server first.

<p align="center">
  <img src="assets/architecture-agentra.svg" alt="AgenticVision architecture in Agentra Labs design system" width="980">
</p>

**CLI + MCP Server** (for Claude Desktop, VS Code, Cursor, Windsurf):
```bash
cargo install agentic-vision-cli agentic-vision-mcp
```

**Core library** (for Rust projects):
```bash
cargo add agentic-vision
```

**Configure Claude Desktop** (`~/Library/Application Support/Claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "agentic-vision": {
      "command": "agentic-vision-mcp",
      "args": ["--vision", "~/.vision.avis", "serve"]
    }
  }
}
```

> See [INSTALL.md](INSTALL.md) for full installation guide, VS Code / Cursor configuration, build from source, and troubleshooting.

> **Do not use `/tmp` for vision files** — macOS and Linux clear this directory periodically. Use `~/.vision.avis` for persistent storage.

## Deployment Model

- **Standalone by default:** AgenticVision is independently installable and operable. Integration with AgenticMemory or AgenticCodebase is optional, never required.
- **Autonomic operations by default:** daemon/runtime maintenance uses safe profile-based defaults with cache hygiene, migration safeguards, and health-ledger snapshots.

| Area | Default behavior | Controls |
|:---|:---|:---|
| Autonomic profile | Conservative local-first posture | `CORTEX_AUTONOMIC_PROFILE=desktop|cloud|aggressive` |
| Cache + registry maintenance | Periodic expiry cleanup and registry GC | `CORTEX_MAINTENANCE_TICK_SECS`, `CORTEX_REGISTRY_GC_EVERY_TICKS`, `CORTEX_REGISTRY_GC_KEEP_DELTAS` |
| Storage migration | Policy-gated with checkpointed auto-safe path | `CORTEX_STORAGE_MIGRATION_POLICY=auto-safe|strict|off` |
| Storage budget policy | 20-year projection + capture rollup under pressure | `CORTEX_STORAGE_BUDGET_MODE=auto-rollup|warn|off`, `CORTEX_STORAGE_BUDGET_BYTES`, `CORTEX_STORAGE_BUDGET_HORIZON_YEARS`, `CORTEX_STORAGE_BUDGET_TARGET_FRACTION` |
| Maintenance throttling | SLA-aware under sustained cache pressure | `CORTEX_SLA_MAX_CACHE_ENTRIES_BEFORE_GC_THROTTLE` |
| Health ledger | Periodic operational snapshots (default: `~/.agentra/health-ledger`) | `CORTEX_HEALTH_LEDGER_DIR`, `AGENTRA_HEALTH_LEDGER_DIR`, `CORTEX_HEALTH_LEDGER_EMIT_SECS` |

---

<a name="quickstart"></a>

## Quickstart

### MCP (Claude Desktop, VS Code, Cursor)

After configuring the MCP server (see [Install](#install)), ask your agent:

> "Take a screenshot and remember it."

The LLM calls `vision_capture` automatically. Then later:

> "What did the screen look like earlier?"

The LLM calls `vision_query` to retrieve and display past captures.

### Rust API

```rust
use agentic_vision::{VisionStore, CaptureSource};

let mut store = VisionStore::open("observations.avis")?;

// Capture from file
let id = store.capture(
    CaptureSource::File("screenshot.png"),
    "Homepage after deploy"
)?;

// Find similar
let matches = store.similar(id, 5)?;
for m in matches {
    println!("  {} (similarity: {:.3})", m.description, m.score);
}
```

---

## Common Workflows

1. **Track UI regression** -- After a deploy, capture before/after screenshots and compare:
   ```
   vision_capture  (before deploy screenshot, label: "pre-deploy")
   vision_capture  (after deploy screenshot,  label: "post-deploy")
   vision_diff     id_a=<before_id> id_b=<after_id>    # Pixel-level region diff
   ```

2. **Build visual evidence trail** -- During debugging, attach screenshots to memory nodes:
   ```
   vision_capture  source=screenshot, labels=["bug-123", "dialog-state"]
   vision_link     capture_id=<id> memory_node_id=<node> relationship="evidence_for"
   ```

3. **Find similar UI states** -- When diagnosing a recurring visual bug:
   ```
   vision_similar  capture_id=<current_issue_id> top_k=5 min_similarity=0.8
   ```

4. **Audit capture quality** -- Periodic maintenance to clean up stale or low-quality captures:
   ```
   vision_health   stale_after_hours=168 low_quality_threshold=0.45
   ```

---

## Validation

| Suite | Tests | Notes |
|:---|---:|:---|
| Rust core (`agentic-vision`) | **38** | Unit + integration (includes screenshot/clipboard) |
| Python SDK tests | **47** | Edge cases, format validation |
| MCP integration suite | **3** | Python → Rust stdio transport |
| Multi-agent suite | **3** | Shared file, vision-memory linking, rapid handoff |
| **Total** | **91** | All passing |

**Two research papers:**
- [Paper I: Cortex — Web Cartography (10 pages, 8 figures, 13 tables)](publication/paper-i-cortex/cortex-paper.pdf)
- [Paper II: AgenticVision-MCP — Persistent Visual Memory via MCP (8 pages, 4 figures, 7 tables)](publication/paper-ii-agentic-vision-mcp/agentic-vision-mcp-paper.pdf)

---

## Repository Structure

This is a Cargo workspace monorepo containing the core library, CLI, MCP server, and FFI bindings.

```
agentic-vision/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── agentic-vision/           # Core library (crates.io: agentic-vision v0.2.2)
│   ├── agentic-vision-cli/       # CLI (crates.io: agentic-vision-cli v0.2.2)
│   ├── agentic-vision-mcp/       # MCP server (crates.io: agentic-vision-mcp v0.2.2)
│   └── agentic-vision-ffi/       # FFI bindings (crates.io: agentic-vision-ffi v0.2.2)
├── tests/                        # Integration tests (Python → Rust, multi-agent)
├── models/                       # ONNX model directory (CLIP ViT-B/32)
├── publication/                  # Research papers (I, II)
├── assets/                       # SVG diagrams and visuals
└── docs/                         # Guides and reference
```

### Running Tests

```bash
# All workspace tests (unit + integration)
cargo test --workspace

# Core library only
cargo test -p agentic-vision

# MCP server only
cargo test -p agentic-vision-mcp

# Python integration tests
python tests/integration/test_mcp_clients.py
python tests/integration/test_multi_agent.py
```

### MCP Server Quick Start

```bash
cargo install agentic-vision-cli agentic-vision-mcp
```

Configure Claude Desktop (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "agentic-vision": {
      "command": "agentic-vision-mcp",
      "args": ["--vision", "~/.vision.avis", "serve"]
    }
  }
}
```

`agentic-vision-mcp` supports both line-delimited JSON-RPC and `Content-Length` framed MCP stdio messages.

---

## Roadmap: Next — Remote Server Support

The next release is planned to add HTTP/SSE transport for remote deployments. Track progress in [#2](https://github.com/agentralabs/agentic-vision/issues/2).

| Feature | Status |
|:---|:---|
| `--token` bearer auth | Planned |
| `--multi-tenant` per-user vision files | Planned |
| `/health` endpoint | Planned |
| `--tls-cert` / `--tls-key` native HTTPS | Planned |
| OCR with Tesseract (`--features ocr`) | Planned |
| Clipboard TIFF fix | Planned |
| `delete` / `export` / `compact` CLI commands | Planned |
| Docker image + compose | Planned |
| Remote deployment docs | Planned |

Planned CLI shape (not available in current release):

```text
agentic-vision-mcp serve-http --port 8081 --token "<token>"
agentic-vision-mcp serve-http --multi-tenant --data-dir /data/users --port 8081 --token "<token>"
```

---

## The .avis File

Your agent's visual memory. Everything it's seen.

| | |
|-|-|
| Size | ~5-8 GB over 20 years |
| Format | Binary captures with embeddings |
| Works with | Any vision-capable model |

## v0.2: Grounding & Workspaces

**Grounding**: Agent cannot claim "page shows X" without capture evidence.

**Workspaces**: Compare across sites and time periods.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). The fastest ways to help:

1. **Try it** and [file issues](https://github.com/agentralabs/agentic-vision/issues)
2. **Add an MCP tool** — extend the visual memory surface
3. **Write an example** — show a real use case
4. **Improve docs** — every clarification helps someone

---

## Privacy and Security

- All captures stay local in `.avis` files -- no telemetry, no cloud sync by default.
- Metadata scrubbing removes EXIF and location data from captured images before storage.
- Storage budget policy prevents unbounded disk growth with 20-year projection and capture rollup.
- Server mode requires an explicit `AGENTIC_TOKEN` environment variable for bearer auth.
- Quality scoring helps identify and prune low-value captures to keep the store lean.

---

<p align="center">
  <sub>Built by <a href="https://github.com/agentralabs"><strong>Agentra Labs</strong></a></sub>
</p>
