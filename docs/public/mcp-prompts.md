---
status: stable
---

# MCP Prompts

AgenticVision provides 4 built-in MCP prompts that agents can invoke for structured visual reasoning.

## `observe`

Guide for capturing and describing what you see.

### Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `context` | string | No | Optional context about what to observe |

### Behavior

The prompt instructs the agent to:

1. Use `vision_capture` to take a screenshot or load the image
2. Describe what you see in detail
3. Note any text, buttons, UI elements, or important visual features
4. If relevant, use `vision_link` to connect the observation to the memory graph

### Example

```json
{
  "name": "observe",
  "arguments": {
    "context": "Check the dashboard for any error indicators"
  }
}
```

## `compare`

Guide for comparing two visual captures.

### Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `capture_a` | string | Yes | First capture ID |
| `capture_b` | string | Yes | Second capture ID |

### Behavior

The prompt instructs the agent to:

1. Use `vision_compare` to get the similarity score
2. Use `vision_diff` for detailed change analysis
3. Summarize what changed between the captures
4. Note any significant visual differences

### Example

```json
{
  "name": "compare",
  "arguments": {
    "capture_a": "1",
    "capture_b": "2"
  }
}
```

## `track`

Guide for tracking visual changes over time.

### Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `target` | string | Yes | What to track |
| `duration` | string | No | How long to track (default: "until changes are detected") |

### Behavior

The prompt instructs the agent to:

1. Use `vision_capture` to get the initial state
2. Use `vision_track` to configure change monitoring for the region
3. Periodically capture new states with `vision_capture`
4. Use `vision_compare` to detect when changes occur
5. After tracking completes, summarize all changes observed

### Example

```json
{
  "name": "track",
  "arguments": {
    "target": "deployment progress bar",
    "duration": "5 minutes"
  }
}
```

## `describe`

Guide for describing a capture in detail.

### Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `capture_id` | string | Yes | Capture ID to describe |

### Behavior

The prompt instructs the agent to:

1. Use the `avis://capture/{capture_id}` resource to load the capture
2. Describe what you see in detail
3. Identify key UI elements, buttons, text fields
4. Note the layout and visual hierarchy
5. Note anything that might be relevant for future reference

### Example

```json
{
  "name": "describe",
  "arguments": {
    "capture_id": "42"
  }
}
```
