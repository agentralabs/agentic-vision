# Quickstart

## 1. Install

```bash
curl -fsSL https://agentralabs.tech/install/vision | bash
```

Profile-specific commands are listed in [Installation](installation.md).

## 2. Validate artifact path

```bash
agentic-vision-mcp -v ~/.vision.avis validate
```

## 3. Inspect capabilities

```bash
agentic-vision-mcp info
```

Expected tool list includes:

- `vision_capture`
- `vision_query`
- `vision_similar`
- `vision_compare`
- `vision_diff`
- `vision_ocr`
- `vision_link`

## 4. Start MCP server

```bash
$HOME/.local/bin/agentic-vision-mcp serve
```

Use `Ctrl+C` to stop after startup verification.
