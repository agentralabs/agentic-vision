---
status: stable
---

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

## Agentic Flow Examples

Once the MCP server is running, your AI agent has access to visual tools. Here are example prompts and the tool chains they trigger.

### Track a UI regression

> Take a screenshot of the login page. Now compare it to yesterday's capture. What changed?

The agent calls `vision_capture`, then `vision_query` to find yesterday's capture, then `vision_diff` to compare them pixel by pixel.

### Build visual evidence

> Capture screenshots of every page in the checkout flow. Label them step-1 through step-5.

The agent calls `vision_capture` for each page with description and labels. The captures are queryable later by label or content.

### Find similar UI states

> I saw a layout bug last week that looks like this. Find any past captures that look similar.

The agent uses `vision_similar` with the current capture's embedding to find visually similar past states.

### Link screenshots to decisions

> Take a screenshot of this error state and link it to memory node 42 as evidence.

The agent calls `vision_capture` then `vision_link` to connect the visual evidence to a cognitive event in AgenticMemory.

### Key tools available to your agent

| Tool | What it does |
|:---|:---|
| `vision_capture` | Screenshot or image capture with embedding |
| `vision_query` | Search past captures by time, label, or content |
| `vision_diff` | Pixel-level comparison between two captures |
| `vision_similar` | Find visually similar past captures |
| `vision_link` | Connect visual evidence to memory nodes |

---

## Server runtime

Cloud servers cannot read laptop-local artifacts directly. Sync `.avis/.amem/.acb` to server storage first, then start MCP services there.
