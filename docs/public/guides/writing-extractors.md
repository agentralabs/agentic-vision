# Writing Custom Extractors

Cortex uses TypeScript extractors that run inside the browser (Layer 3 fallback) to
extract structured data from web pages. For HTTP-based extraction (the primary path),
you can also contribute CSS selector patterns to `runtime/src/acquisition/css_selectors.json`
and platform action templates to `runtime/src/acquisition/platform_actions.json`.

## Extractor Structure

Each extractor is a self-contained IIFE (Immediately Invoked Function
Expression) that returns a JSON result:

```typescript
(() => {
  // Your extraction logic here
  return {
    type: "custom",
    data: { /* your structured data */ },
    confidence: 0.9,
  };
})();
```

## Built-in Extractors

- **content.ts** — Headings, paragraphs, prices, ratings, images
- **actions.ts** — Buttons, links, form inputs with OpCode mapping
- **navigation.ts** — Internal/external links, pagination, breadcrumbs
- **structure.ts** — Page regions (header, nav, main, sidebar, footer)
- **metadata.ts** — JSON-LD, OpenGraph, meta tags, schema.org

## Creating a Custom Extractor

1. Create a new file in `extractors/core/`:

```typescript
// extractors/core/my-extractor.ts
export function extractMyData(document: Document) {
  const results = [];
  // ... extraction logic
  return results;
}
```

2. Build it:

```bash
cd extractors && npm run build
```

3. The compiled bundle will be in `extractors/dist/`.

## Best Practices

- Use `document.querySelector` / `querySelectorAll` for DOM queries
- Set confidence based on signal strength (schema.org > aria > heuristic)
- Include bounding box data for visual elements
- Handle shadow DOM when `options.includeShadowDom` is true
