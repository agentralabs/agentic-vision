# Interactive Sessions: Multi-Step Flows with ACT

Cortex supports persistent sessions for multi-step workflows
like login, add-to-cart, and checkout.

## How Sessions Work

Many actions (add-to-cart, form submission, search) are executed via HTTP POST without a browser. For complex interactions (drag-drop, canvas, multi-step wizards), Cortex falls back to a browser context. Sessions hold cookies and state across multiple actions regardless of execution method.

### HTTP-First Authentication

Password-based login can be done entirely via HTTP:

```python
import cortex_client

# Login via HTTP (no browser needed for standard login forms)
session = cortex_client.login("shop.example.com", username="user@example.com", password="pw")

# Map with authenticated session
site = cortex_client.map("shop.example.com", session=session)
```

For OAuth login, a brief browser session is needed for the consent screen:

```python
session = cortex_client.login_oauth("shop.example.com", provider="google")
```

## Python Example: Login Flow

```python
import cortex_client

# Map the site first
site = cortex_client.map("shop.example.com")

# Find the login page
login_pages = site.filter(page_type=8)  # PageType::Login = 0x08
login_node = login_pages[0].index

# Start a session and log in
result = site.act(
    node=login_node,
    opcode=(0x03, 0x00),  # Form: fill input
    params={"selector": "#email", "value": "user@example.com"},
    session_id="my-session",
)

# Submit the login form
result = site.act(
    node=login_node,
    opcode=(0x04, 0x00),  # Auth: login
    session_id="my-session",
)

# Navigate to products (still logged in)
products = site.filter(page_type=4)
```

## OpCode Reference

| Category | Action | OpCode | Description |
|----------|--------|--------|-------------|
| Navigation | Click | (0x01, 0x00) | Click a link |
| Commerce | Add to Cart | (0x02, 0x00) | Add item to cart |
| Form | Fill Input | (0x03, 0x00) | Fill a form field |
| Form | Submit | (0x03, 0x05) | Submit a form |
| Auth | Login | (0x04, 0x00) | Click login button |

## Session Timeout

Sessions expire after 1 hour of inactivity by default.

## Next Steps

After mapping and acting on sites, consider:

- [Compiling the map](web-compiler.md) into a typed API for programmatic access
- [Querying with WQL](wql.md) to filter and compare data across sites
- [Tracking changes](temporal.md) to detect price drops, availability, and trends
