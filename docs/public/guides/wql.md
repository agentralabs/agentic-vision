# WQL (Web Query Language) Guide

WQL is a SQL-like query language for querying website data across mapped domains. Instead of writing code to filter and sort SiteMap data, agents can express queries in a familiar syntax.

## Basic Syntax

```sql
SELECT fields FROM ModelType [WHERE conditions] [ORDER BY field ASC|DESC] [LIMIT n] [ACROSS domains]
```

## Examples

### Simple Select

```sql
-- Get all products, limit to 10
SELECT * FROM Product LIMIT 10

-- Get specific fields
SELECT name, price, rating FROM Product LIMIT 20
```

### Filtered Queries

```sql
-- Products under $200
SELECT * FROM Product WHERE price < 200

-- Products with high ratings
SELECT * FROM Product WHERE rating > 4.0

-- Combined filters
SELECT * FROM Product WHERE price < 200 AND rating > 4.0
```

### Ordered Results

```sql
-- Cheapest products first
SELECT * FROM Product ORDER BY price ASC LIMIT 10

-- Highest rated first
SELECT * FROM Product ORDER BY rating DESC LIMIT 5
```

### Cross-Domain Queries

```sql
-- Compare products across sites
SELECT * FROM Product ACROSS amazon.com, bestbuy.com LIMIT 30

-- Find articles across news sites
SELECT * FROM Article ACROSS nytimes.com, bbc.com, reuters.com LIMIT 20
```

### Temporal Queries

```sql
-- Products with predicted price (parsed, executor support in progress)
SELECT name, PREDICTED(price, 7d) FROM Product WHERE TREND(price) = "decreasing"
```

## Supported Model Types

| Model | PageType | Typical Fields |
|-------|----------|---------------|
| Product | ProductDetail | price, rating, availability, discount |
| Category | ProductListing | -- |
| Article | Article | content_length, heading_count |
| Review | ReviewList | -- |
| FAQ | Faq | -- |
| Organization | AboutPage | -- |
| Contact | ContactPage | -- |
| Cart | Cart | -- |
| Checkout | Checkout | -- |
| Account | Account | -- |
| Media | MediaPage | -- |
| Home | Home | link_count |
| Event | Calendar | -- |
| Documentation | Documentation | content_length |
| Forum | Forum | -- |
| Search | SearchResults | -- |

## CLI Usage

```bash
# Run a WQL query
cortex wql "SELECT * FROM Product WHERE price < 100 LIMIT 5"

# Cross-domain query
cortex wql "SELECT * FROM Product ACROSS amazon.com, bestbuy.com LIMIT 20"
```

## Error Handling

WQL handles malformed queries gracefully:

- Missing `FROM` clause returns a clear error message
- Unknown model types return empty results (mapped to `PageType::Unknown`)
- Invalid syntax returns a parse error with position information
- The parser never crashes on malformed input

## Limitations

- Domain names with hyphens are not supported in ACROSS clauses (use underscore-free names or the programmatic API)
- Temporal functions (PREDICTED, TREND, HISTORY) are parsed but executor support is in progress
- JOIN queries are parsed but cross-model joins are not yet executed
- No aggregation functions (COUNT, SUM, AVG) yet
