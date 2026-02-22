# Integration Guide

## MCP (recommended)

AgenticVision is consumed through MCP. Any MCP client can use the same server entry.

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

After adding the entry, restart the MCP client.

## With AgenticMemory

Run both servers to link visual captures to cognitive nodes:

```json
{
  "mcpServers": {
    "agentic-memory": {
      "command": "$HOME/.local/bin/agentic-memory-mcp",
      "args": ["serve"]
    },
    "agentic-vision": {
      "command": "$HOME/.local/bin/agentic-vision-mcp",
      "args": ["serve"]
    }
  }
}
```

Use `vision_link` with a memory node ID to connect what the agent sees to what it remembers.

## Server runtime

Cloud servers cannot read laptop-local artifacts directly. Sync `.avis/.amem/.acb` to server storage first, then start MCP services there.
