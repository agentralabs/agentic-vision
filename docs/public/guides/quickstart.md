# Quickstart: From Zero to Typed API in 90 Seconds

## Install

```bash
# Install Cortex
cargo install cortex-runtime
```

That's it. No other setup needed.

## Map Your First Site

```bash
cortex map example.com
```

On first run, Cortex will automatically:
1. Start the background daemon
2. Map the site via HTTP-first layered acquisition (no browser needed)
3. Download Chromium for Testing (~130 MB) only if browser fallback is needed

Most sites are mapped entirely via HTTP. Chromium is downloaded on-demand for sites with very low structured data coverage or when using ACT operations.

Output:
```
Mapping example.com...
Map complete:
  Domain:   example.com
  Nodes:    15
  Edges:    22
  Rendered: 10
  Duration: 4.32s
```

## Compile Into a Typed API

```bash
cortex compile example.com
```

This infers typed schemas from the map and generates client libraries:

```
Compiled example.com:
  Models:  3 (Article, Home, Contact)
  Fields:  12 total
  Actions: 2
  Output:  ~/.cortex/compiled/example.com/
           ├── client.py
           ├── client.ts
           ├── openapi.yaml
           ├── schema.graphql
           └── mcp.json
```

## Query with WQL

```bash
# SQL-like queries on your mapped data
cortex wql "SELECT * FROM Article ORDER BY content_length DESC LIMIT 5"
```

## Use from Python

```python
from cortex_client import CortexClient

client = CortexClient()

# Map and compile
site = client.map("example.com")
schema = client.compile("example.com")

# Use typed models
for model in schema.models:
    print(f"{model.name}: {len(model.fields)} fields")

# Query with WQL
results = client.wql("SELECT * FROM Article LIMIT 5")

# Or use the low-level map API
products = site.filter(page_type=4, limit=10)
for p in products:
    print(f"{p.url} (confidence: {p.confidence:.2f})")

# Find path from home to checkout
path = site.pathfind(0, target_node)
print(f"Path: {path.hops} hops, {len(path.required_actions)} actions needed")
```

## Use from TypeScript

```typescript
import { map } from 'cortex-web-client';

const site = await map('example.com');
const results = await site.filter({ pageType: 4, limit: 10 });
results.forEach(r => console.log(r.url));
```

## Track Changes Over Time

```bash
# Push maps to the registry for temporal tracking
cortex registry list

# Query price history
cortex history amazon.com "https://amazon.com/product/123" --dim price --since 2025-01-01

# Detect patterns
cortex patterns amazon.com "https://amazon.com/product/123" --dim price
```

See [Temporal Intelligence](temporal.md), [Web Compiler](web-compiler.md), [WQL](wql.md), and [Collective Graph](collective-graph.md) for full details.

## Other Useful Commands

```bash
# Check environment
cortex doctor

# Search by page type and features
cortex query example.com --type product_detail --price-lt 100

# Find shortest path between nodes
cortex pathfind example.com --from 0 --to 42

# Machine-readable output
cortex map example.com --json

# Manage the registry
cortex registry list
cortex registry stats

# Clear cached maps
cortex cache clear

# Stop the daemon
cortex stop
```

## Shell Completions

```bash
# Bash
cortex completions bash >> ~/.bashrc

# Zsh
cortex completions zsh >> ~/.zshrc

# Fish
cortex completions fish > ~/.config/fish/completions/cortex.fish
```
