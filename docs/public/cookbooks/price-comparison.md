# Cookbook: Price Comparison Across 5 Sites

Compare prices for a product across multiple e-commerce sites.

## Python

```python
import cortex_client

domains = [
    "shop-a.example.com",
    "shop-b.example.com",
    "shop-c.example.com",
    "shop-d.example.com",
    "shop-e.example.com",
]

# Map all sites
sites = cortex_client.map_many(domains, max_render=50)

# Find product pages with prices
all_products = []
for site in sites:
    products = site.filter(
        page_type=4,       # ProductDetail
        features={48: {"min": 0.01}},  # Has a price (FEAT_PRICE > 0)
        sort_by=(48, "asc"),           # Sort by price ascending
        limit=20,
    )
    for p in products:
        all_products.append({
            "domain": site.domain,
            "url": p.url,
            "price": p.features.get(48, 0),
            "rating": p.features.get(52, 0),
        })

# Sort by price
all_products.sort(key=lambda x: x["price"])

# Print comparison
print(f"{'Domain':<30} {'Price':>8} {'Rating':>7} URL")
print("-" * 90)
for p in all_products[:20]:
    print(f"{p['domain']:<30} ${p['price']:>7.2f} {p['rating']:>6.1f} {p['url']}")
```

## Feature Dimensions Used

| Dimension | Name | Description |
|-----------|------|-------------|
| 48 | FEAT_PRICE | Current price |
| 49 | FEAT_PRICE_ORIGINAL | Original/list price |
| 50 | FEAT_DISCOUNT_PERCENT | Discount percentage |
| 52 | FEAT_RATING | Star rating (0-5) |
| 53 | FEAT_REVIEW_COUNT_LOG | Log of review count |
