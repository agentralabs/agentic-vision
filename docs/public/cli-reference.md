---
status: stable
---

# CLI Reference

The `avis` CLI provides command-line access to AgenticVision visual memory stores.

## Global Options

| Option | Description |
|--------|-------------|
| `--format <fmt>` | Output format: `text` (default), `json` |
| `--model <path>` | Path to CLIP ONNX model |
| `--verbose` | Enable debug logging |
| `-h, --help` | Print help information |
| `-V, --version` | Print version |

When invoked with no subcommand, `avis` launches an interactive REPL.

## Commands

### `avis init`

Create a new empty `.avis` file.

```bash
# Create with default 512-dim embeddings
avis init project.avis

# Create with custom embedding dimension
avis init project.avis --dimension 768
```

| Argument | Required | Description |
|----------|----------|-------------|
| `file` | Yes | Path for the new `.avis` file |
| `--dimension` | No | Embedding vector dimension (default: 512) |

### `avis info`

Display information about an `.avis` file.

```bash
avis info project.avis
avis info project.avis --format json
```

### `avis capture`

Capture an image and add it to the store.

```bash
# Capture from a file
avis capture project.avis screenshot.png

# Capture with labels and description
avis capture project.avis dashboard.png --labels "ui,dashboard" --description "Main dashboard view"

# Use a custom CLIP model
avis capture project.avis photo.jpg --model /path/to/clip.onnx
```

| Argument | Required | Description |
|----------|----------|-------------|
| `file` | Yes | Path to `.avis` file |
| `source` | Yes | Image source (file path) |
| `--labels` | No | Comma-separated labels |
| `--description` | No | Human-readable description |

### `avis query`

Search observations with filters.

```bash
# List all observations
avis query project.avis

# Filter by session
avis query project.avis --session 1

# Filter by labels
avis query project.avis --labels "ui,error"

# Limit results
avis query project.avis --limit 5
```

| Argument | Required | Description |
|----------|----------|-------------|
| `file` | Yes | Path to `.avis` file |
| `--session` | No | Filter by session ID |
| `--labels` | No | Comma-separated label filter |
| `--limit` | No | Maximum results (default: 20) |

### `avis similar`

Find visually similar captures by embedding.

```bash
avis similar project.avis 42
avis similar project.avis 42 --top-k 5 --min-similarity 0.8
```

| Argument | Required | Description |
|----------|----------|-------------|
| `file` | Yes | Path to `.avis` file |
| `capture_id` | Yes | Reference capture ID |
| `--top-k` | No | Maximum results (default: 10) |
| `--min-similarity` | No | Minimum similarity threshold (default: 0.5) |

### `avis compare`

Compare two captures by embedding similarity.

```bash
avis compare project.avis 1 2
```

### `avis diff`

Pixel-level diff between two captures.

```bash
avis diff project.avis 1 2
```

### `avis health`

Quality and staleness health report.

```bash
avis health project.avis
avis health project.avis --stale-hours 48 --low-quality 0.3 --max-examples 10
```

| Argument | Required | Description |
|----------|----------|-------------|
| `file` | Yes | Path to `.avis` file |
| `--stale-hours` | No | Hours before a capture is stale (default: 168) |
| `--low-quality` | No | Quality threshold (default: 0.45) |
| `--max-examples` | No | Max example IDs per category (default: 20) |

### `avis link`

Link a capture to an AgenticMemory node.

```bash
avis link project.avis 42 100
```

| Argument | Required | Description |
|----------|----------|-------------|
| `file` | Yes | Path to `.avis` file |
| `capture_id` | Yes | Visual capture ID |
| `memory_node_id` | Yes | AgenticMemory node ID |

### `avis stats`

Print aggregate statistics.

```bash
avis stats project.avis
```

### `avis export`

Export observations as JSON.

```bash
avis export project.avis
avis export project.avis --pretty
```

### `avis ground`

Verify a visual claim has capture backing.

```bash
avis ground project.avis "login button is visible"
avis ground project.avis "error dialog" --threshold 0.5
```

| Argument | Required | Description |
|----------|----------|-------------|
| `file` | Yes | Path to `.avis` file |
| `claim` | Yes | Visual claim to verify |
| `--threshold` | No | Minimum match score (default: 0.3) |

### `avis evidence`

Return visual evidence for a query.

```bash
avis evidence project.avis "dashboard chart"
avis evidence project.avis "error message" --limit 5
```

### `avis suggest`

Suggest similar captures for a phrase.

```bash
avis suggest project.avis "navigation menu"
avis suggest project.avis "sidebar" --limit 3
```

### `avis workspace`

Workspace operations across multiple `.avis` files.

```bash
# Create a workspace
avis workspace create my-project

# Add vision files
avis workspace add my-project frontend.avis --role primary --label "Frontend"
avis workspace add my-project backend.avis --role secondary

# List workspace contents
avis workspace list my-project

# Query across all files
avis workspace query my-project "login form"

# Compare an element across contexts
avis workspace compare my-project "navigation bar"

# Cross-reference an element
avis workspace xref my-project "error dialog"
```

### `avis completions`

Generate shell completion scripts.

```bash
avis completions bash > ~/.local/share/bash-completion/completions/avis
avis completions zsh > ~/.zfunc/_avis
avis completions fish > ~/.config/fish/completions/avis.fish
```
