---
status: stable
---

# MCP Resources

AgenticVision exposes visual memory data through the `avis://` URI scheme in the MCP Resources API.

## URI Scheme

All resources use the `avis://` prefix.

## Static Resources

### `avis://stats`

Return visual memory statistics.

**Format:** JSON object.

```json
{
  "capture_count": 42,
  "session_count": 5,
  "embedding_dim": 512,
  "file_path": "/path/to/vision.avis"
}
```

### `avis://recent`

Return the most recent 20 captures.

**Format:** JSON array of observation objects with metadata.

## Resource Templates

### `avis://capture/{id}`

Return a single visual capture with metadata and thumbnail.

**Format:** JSON object.

```json
{
  "id": 1,
  "timestamp": 1709000000,
  "session_id": 1,
  "dimensions": { "width": 1920, "height": 1080 },
  "labels": ["ui", "dashboard"],
  "description": "Main dashboard view",
  "quality_score": 0.85,
  "memory_link": null
}
```

### `avis://session/{id}`

Return all captures from a specific session.

**Format:** JSON array of observation objects.

```json
[
  {
    "id": 1,
    "timestamp": 1709000000,
    "labels": ["ui"],
    "description": "Landing page"
  },
  {
    "id": 2,
    "timestamp": 1709000060,
    "labels": ["ui", "form"],
    "description": "Login form"
  }
]
```

### `avis://timeline/{start}/{end}`

Return captures within a timestamp range.

| Parameter | Type | Description |
|-----------|------|-------------|
| `start` | number | Unix timestamp (inclusive lower bound) |
| `end` | number | Unix timestamp (inclusive upper bound) |

**Format:** JSON array of observation objects within the time window, sorted chronologically.

### `avis://similar/{id}`

Return the top 10 visually similar captures for a given capture ID.

**Format:** JSON array of similarity match objects.

```json
[
  { "id": 3, "similarity": 0.95 },
  { "id": 7, "similarity": 0.89 }
]
```

## Cross-Sister Resources

When running alongside other Agentra sisters, AgenticVision resources can be referenced in memory nodes, codebase analysis, and temporal graphs:

- Memory nodes can link to `avis://capture/{id}` for visual evidence
- Codebase analysis can reference `avis://capture/{id}` for UI regression context
- Time entries can reference `avis://timeline/{start}/{end}` for visual change history
