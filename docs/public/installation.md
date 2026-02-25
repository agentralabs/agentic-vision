---
status: stable
---

# Installation

## Recommended

```bash
curl -fsSL https://agentralabs.tech/install/vision | bash
```

## Profiles

```bash
curl -fsSL https://agentralabs.tech/install/vision/desktop | bash
curl -fsSL https://agentralabs.tech/install/vision/terminal | bash
curl -fsSL https://agentralabs.tech/install/vision/server | bash
```

## Cargo

```bash
cargo install agentic-vision-mcp
```

## npm

```bash
npm install @agenticamem/vision
```

## Verify

```bash
agentic-vision-mcp --version
agentic-vision-mcp --help
agentic-vision-mcp info
```

## Server auth

```bash
export AGENTIC_TOKEN="$(openssl rand -hex 32)"
```

Use the same bearer token across server MCP clients.
