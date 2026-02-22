# Quickstart — From Zero to Typed API in 5 Minutes

## 1. Install AgenticVision

Use the one-liner installer (recommended):
```bash
curl -fsSL https://agentralabs.tech/install/vision | bash
```

Install by environment:
```bash
# Desktop MCP clients
curl -fsSL https://agentralabs.tech/install/vision/desktop | bash

# Terminal-only (no desktop config writes)
curl -fsSL https://agentralabs.tech/install/vision/terminal | bash

# Remote/server host (no desktop config writes)
curl -fsSL https://agentralabs.tech/install/vision/server | bash
```

For server mode:

```bash
export AGENTIC_TOKEN="$(openssl rand -hex 32)"
```

Cloud/server runtime cannot read laptop files directly. Sync `.avis/.amem/.acb` artifacts to server storage first.

Or install MCP + library manually:
```bash
cargo install agentic-vision-mcp
cargo add agentic-vision
```

## 2. Map Your First Site

```bash
cortex map example.com
```

Output:
```
Mapping example.com...
  Layer 0 — Metadata (robots.txt, sitemap.xml)
  Layer 1 — Structured Data (JSON-LD, OpenGraph)
  Layer 1.5 — Pattern Engine (CSS selectors)
Map complete:
  Domain:   example.com
  Nodes:    15
  Edges:    22
  Size:     9.2 KB
  Duration: 2.1s
```

The binary `.ctx` map is cached in `~/.cortex/maps/`. Subsequent queries are instant.

## 3. Compile Into a Typed API

```bash
cortex compile example.com
```

```
Compiled example.com:
  Models:  3 (Article, Home, Contact)
  Fields:  12 total
  Output:  ~/.cortex/compiled/example.com/
           ├── client.py          # Python client
           ├── client.ts          # TypeScript client
           ├── openapi.yaml       # OpenAPI 3.0 spec
           ├── schema.graphql     # GraphQL schema
           └── mcp.json           # MCP tool definitions
```

## 4. Query with WQL

WQL (Web Query Language) — SQL for websites:

```bash
cortex wql "SELECT url, page_type FROM Node LIMIT 5"
```

More examples:
```bash
# Find products under $200 sorted by price
cortex wql "SELECT name, price FROM Product WHERE price < 200 ORDER BY price ASC LIMIT 10"

# Articles sorted by content length
cortex wql "SELECT url, title FROM Article ORDER BY content_length DESC LIMIT 5"

# Products with high ratings
cortex wql "SELECT name, price, rating FROM Product WHERE rating > 4.0 LIMIT 10"
```

## 5. Use from Python

```bash
pip install cortex-agent
```

```python
import cortex_client

# Map a website into a navigable graph
site = cortex_client.map("amazon.com")
print(f"Mapped {len(site.nodes)} pages, {len(site.edges)} links")

# Find products under $300
products = site.filter(page_type=0x04, features={48: {"lt": 300}}, limit=10)
for p in products:
    print(f"  {p.url}  price=${p.features[48]:.2f}")

# Pathfind between pages
path = site.pathfind(0, products[0].index)
print(f"Path: {' → '.join(str(n) for n in path.nodes)}")

# WQL from Python
results = cortex_client.wql("SELECT name, price FROM Product WHERE price < 200 LIMIT 5")

# Compare across sites
result = cortex_client.compare(["amazon.com", "ebay.com", "walmart.com"])
```

## 6. Use from TypeScript

```bash
npm install cortex-web-client
```

```typescript
import { map } from "cortex-web-client";

const site = await map("amazon.com");
console.log(`Mapped ${site.nodeCount} pages, ${site.edgeCount} links`);

const products = await site.filter({
  pageType: 4,
  features: { 48: { lt: 300 } },
  limit: 10,
});

const path = await site.pathfind(0, products[0].index);
```

## 7. Connect to Your AI Agent

```bash
cortex plug
```

This auto-discovers Claude Desktop, Claude Code, Cursor, Windsurf, Continue, and Cline — and injects 9 Cortex tools via MCP.

Then ask your agent:
- *"Map amazon.com and find laptops under $500"*
- *"Compare prices for headphones across amazon.com and ebay.com"*
- *"Find the checkout flow on example.com"*

The agent uses `cortex_map`, `cortex_query`, `cortex_pathfind`, and other tools automatically.

## 8. Track Changes Over Time

```bash
# Push to the registry
cortex registry list

# Query price history
cortex history amazon.com "https://amazon.com/dp/B0ABCDEF" --dim price --since 2026-01-01

# Detect trends and anomalies
cortex patterns amazon.com "https://amazon.com/dp/B0ABCDEF" --dim price
```

## Other Useful Commands

```bash
cortex query example.com --type product_detail --price-lt 100   # Search by type + features
cortex pathfind example.com --from 0 --to 42                    # Shortest path
cortex perceive "https://example.com/page"                      # Single page analysis
cortex map example.com --json                                   # Machine-readable output
cortex cache clear                                              # Clear cached maps
cortex status                                                   # Show runtime status
cortex stop                                                     # Stop daemon
```

## Shell Completions

```bash
cortex completions bash >> ~/.bashrc     # Bash
cortex completions zsh >> ~/.zshrc       # Zsh
cortex completions fish > ~/.config/fish/completions/cortex.fish  # Fish
```

## Next Steps

- [Concepts](concepts.md) — Understand SiteMap format, feature vectors, acquisition layers
- [API Reference](api-reference.md) — Full Python, TypeScript, CLI, REST, and MCP APIs
- [Web Compiler Guide](guides/web-compiler.md) — Turn any website into a typed API
- [WQL Guide](guides/wql.md) — SQL-like queries for web data
- [Integration Guide](integration-guide.md) — MCP setup, framework adapters
