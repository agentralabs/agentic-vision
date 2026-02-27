---
status: stable
---

# Troubleshooting

Common issues and solutions for AgenticVision.

## Installation Issues

### Binary not found after install

Ensure `~/.local/bin` is in your PATH:

```bash
export PATH="$HOME/.local/bin:$PATH"
# Add to ~/.bashrc or ~/.zshrc for persistence
```

### Install script fails with "jq not found"

The installer needs `jq` or `python3` for MCP config merging:

```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt install jq

# Or use python3 (usually pre-installed)
python3 --version
```

### Cargo build fails

Ensure you have the latest stable Rust toolchain:

```bash
rustup update stable
```

## MCP Server Issues

### Server not appearing in MCP client

1. Verify the binary exists: `ls ~/.local/bin/agentic-vision-mcp`
2. Check config was merged: look for `agentic-vision` in your MCP client config
3. Restart your MCP client completely (not just reload)
4. Run manually to check for errors: `agentic-vision-mcp serve`

### "AGENTIC_TOKEN required" error

This occurs in HTTP server mode with token authentication enabled. Set the token:

```bash
export AGENTIC_TOKEN="$(openssl rand -hex 32)"
```

### Server crashes on startup

Check for permission issues with the vision file:

```bash
ls -la ~/.agentic-vision/vision.avis
# Ensure the file is readable and writable
```

### "--data-dir is required" error

This occurs when using `--multi-tenant` without specifying a data directory:

```bash
agentic-vision-mcp serve-http --multi-tenant --data-dir /var/lib/agentic-vision/tenants
```

## File Format Issues

### "Invalid magic bytes" error

The file is not a valid `.avis` file. Check:

```bash
xxd -l 4 project.avis
# Should show: 4156 4953 (AVIS)
```

### "Version mismatch" error

The file was created with a newer version of AgenticVision. Update your installation:

```bash
curl -fsSL https://agentralabs.tech/install/vision | bash
```

### Empty .avis file

If a file appears empty after operations, ensure the session was ended properly. The MCP server auto-saves on session end. For CLI usage, verify the file was written:

```bash
avis info project.avis
```

## Vision Operation Issues

### Captures returning low quality scores

Quality scores below 0.45 indicate potential issues:

- Image may be too small (below 64x64 pixels)
- Image may be heavily compressed or corrupted
- Check with: `avis health project.avis --low-quality 0.3`

### Similarity search returning no results

Verify the store has embeddings:

```bash
avis stats project.avis
# Check that capture count > 0 and embedding dimension is non-zero
```

If using `vision_similar` with `capture_id`, ensure the ID exists:

```bash
avis query project.avis --format json
```

### vision_ground returning "ungrounded" unexpectedly

The grounding tool matches claims against observation descriptions and labels using word overlap. Improve grounding by:

1. Adding descriptive labels when capturing: `--labels "login,button,error"`
2. Adding descriptions: `--description "Login page with error message"`
3. Using `vision_suggest` to find what terms match existing captures

### vision_track not detecting changes

The tracking tool configures monitoring but captures must be triggered externally. Workflow:

1. Call `vision_track` to configure the region
2. Periodically call `vision_capture` with `source.type = "screenshot"`
3. Call `vision_compare` between consecutive captures to detect changes

### OCR not available

OCR is not available in the current release. It will be added in a future version with `--features ocr` (Tesseract integration). The `vision_ocr` tool returns an informational message about this.

## Workspace Issues

### "workspace not found" error

Workspace state is persisted in `~/.agentic/vision/workspaces.json`. If the file is missing or corrupted:

```bash
# Check workspace state
cat ~/.agentic/vision/workspaces.json

# Recreate the workspace
avis workspace create my-project
avis workspace add my-project frontend.avis --role primary
```

### Workspace query returning empty results

Ensure the `.avis` files referenced in the workspace still exist at their original paths:

```bash
avis workspace list my-project
# Verify each listed path is accessible
```

## Performance Issues

### Slow startup with large .avis files

For stores with many captures, consider:

1. Splitting by project: use separate `.avis` files per project
2. Running `vision_consolidate` to remove redundant captures
3. Using `vision_health` to identify and clean up stale captures

### High memory usage

The entire visual memory store is loaded into memory. For very large stores:

1. Archive old captures: `avis export project.avis --pretty > archive.json`
2. Create a fresh store and re-import only recent captures
3. Use workspaces to keep per-project stores small

## Getting Help

- GitHub Issues: https://github.com/agentralabs/agentic-vision/issues
- Documentation: https://agentralabs.tech/docs/vision
