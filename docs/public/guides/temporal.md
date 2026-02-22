# Temporal Intelligence Guide

Temporal Intelligence enables Cortex to track how websites change over time. By analyzing delta history, agents can detect trends, predict future values, and receive alerts when conditions are met.

## Core Capabilities

### History Queries

Query the history of any feature dimension for a node:

```bash
cortex history amazon.com "https://amazon.com/product/123" --dim price --since 2025-01-01
```

### Pattern Detection

Detect patterns in temporal data:

```bash
cortex patterns amazon.com "https://amazon.com/product/123" --dim price
```

Detectable patterns:
- **Trends**: Increasing, decreasing, or stable over time
- **Periodicity**: Recurring cycles (weekly, monthly)
- **Anomalies**: Unusual spikes or drops
- **Seasonality**: Predictable seasonal variations

### Predictions

Predict future values based on historical trends:

```python
from cortex_client import CortexClient

client = CortexClient()
prediction = client.predict("amazon.com", "/product/123", dim="price", days_ahead=7)
```

### Watch Alerts

Set up rules that trigger when conditions are met:

```python
# Alert when price drops below $50
client.watch(
    domain="amazon.com",
    feature_dim=48,  # price
    condition="below",
    threshold=50.0,
    notify="webhook"
)
```

## Watch Conditions

| Condition | Description |
|-----------|-------------|
| `ValueAbove(threshold)` | Value rises above threshold |
| `ValueBelow(threshold)` | Value drops below threshold |
| `ChangeByPercent(pct)` | Value changes by more than percentage |
| `Available` | Item becomes available (0 -> positive) |
| `NewInstance` | New page of watched type appears |

## Data Requirements

| Analysis | Minimum Data Points | Recommended |
|----------|-------------------|-------------|
| Trend detection | 3 | 10+ |
| Predictions | 3 | 20+ |
| Anomaly detection | 5 | 30+ |
| Periodicity | 2 full cycles | 4+ cycles |

## Limitations

- **Sparse data**: With only 2-3 data points, pattern detection is unreliable. Cortex will return `None` rather than guessing.
- **Linear predictions**: The prediction engine uses simple linear regression. It works well for linear trends but cannot capture complex, non-linear patterns.
- **Historical depth**: Temporal data depends on delta history in the registry. If you just started mapping a site, there is no historical data yet.
- **Clock accuracy**: Timestamps come from the local system clock. Clock drift between Cortex instances could affect temporal analysis.
