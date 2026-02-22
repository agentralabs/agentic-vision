# Web Compiler Guide

The Web Compiler transforms raw SiteMaps into typed schemas and auto-generated client libraries. Instead of working with raw feature vectors, agents can interact with websites through typed APIs.

## How It Works

1. **Map a site**: `cortex map amazon.com`
2. **Compile the schema**: `cortex compile amazon.com`
3. **Use the generated client**: Import the Python/TypeScript client and call typed methods

## Schema Inference

The compiler analyzes feature vectors across all nodes in a SiteMap to detect:

- **Models**: Groups of pages with similar characteristics (e.g., Product pages, Article pages)
- **Fields**: Meaningful features for each model (e.g., price, rating, availability for Products)
- **Actions**: Discoverable operations (add to cart, submit form, etc.)
- **Relationships**: How models connect (e.g., Category -> Product navigation)

## Generated Clients

The compiler produces clients in 5 formats:

| Format | File | Use Case |
|--------|------|----------|
| Python | `client.py` | Python agents and scripts |
| TypeScript | `client.ts` | Browser and Node.js agents |
| OpenAPI | `openapi.yaml` | API documentation and tooling |
| GraphQL | `schema.graphql` | GraphQL-based agent frameworks |
| MCP | `mcp.json` | Claude and MCP-compatible agents |

## CLI Usage

```bash
# Compile a single domain
cortex compile amazon.com

# Compile with custom output directory
cortex compile amazon.com --output ./my-clients/

# Compile and unify all cached domains
cortex compile amazon.com --all
```

## Cross-Site Unification

When compiling with `--all`, the compiler unifies schemas across multiple domains:

- Merges models with the same name (e.g., "Product" from amazon.com and bestbuy.com)
- Combines fields from all sources
- Generates a universal client that works across all compiled sites

## Example: E-Commerce

After mapping and compiling an e-commerce site:

```python
from cortex_client import CortexClient

client = CortexClient()
schema = client.compile("amazon.com")

# Schema contains typed models
for model in schema.models:
    print(f"Model: {model.name}")
    for field in model.fields:
        print(f"  {field.name}: {field.field_type}")
```

## Limitations

- Schema inference depends on feature vector quality from the mapping phase
- Sites with very few pages may produce limited or no models
- Generated code is a starting point; complex business logic may need manual additions
