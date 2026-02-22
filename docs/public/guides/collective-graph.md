# Collective Web Graph Guide

The Collective Web Graph enables multiple Cortex instances to share and synchronize map data. Instead of each agent mapping a site independently, agents can push their maps to a shared registry and pull maps from other agents.

## Core Concepts

### Delta Computation

When a site changes between mapping sessions, Cortex computes a delta (diff) rather than re-sharing the entire map:

- **Nodes added**: New pages discovered
- **Nodes removed**: Pages that disappeared
- **Nodes modified**: Features that changed (e.g., price updates)

Deltas are typically much smaller than full maps, enabling efficient synchronization.

### Local Registry

The registry stores map snapshots and deltas on disk:

```bash
# List all maps in the registry
cortex registry list

# Show registry statistics
cortex registry stats

# Garbage collect old deltas
cortex registry gc
```

### Privacy Stripping

Before sharing maps, Cortex automatically strips sensitive data:

- All session features (dims 112-127) are zeroed
- Cookie consent and popup data are cleared
- Auth-walled page features are zeroed

This ensures no private browsing data leaks through shared maps.

## How It Works

1. **Agent A maps a site**: Creates a SiteMap with full feature vectors
2. **Privacy strip**: Session and auth data are removed
3. **Push to registry**: The sanitized map is stored locally
4. **Agent B pulls**: Retrieves the map from the registry
5. **Delta sync**: On subsequent mappings, only changes are pushed

## CLI Usage

```bash
# List registered maps
cortex registry list

# View statistics
cortex registry stats

# Clean up old deltas (keep last 10 per domain)
cortex registry gc
```

## Delta Format

Deltas include:
- Domain name and timestamp
- Contributing instance ID
- Lists of added, removed, and modified nodes
- Edge changes
- Schema changes (if any)

## Limitations

- Currently local-only; peer-to-peer sync is planned for v2.0
- Privacy stripping is conservative; some non-sensitive session data may also be cleared
- Large sites with frequent changes may accumulate many deltas; use `registry gc` to clean up
