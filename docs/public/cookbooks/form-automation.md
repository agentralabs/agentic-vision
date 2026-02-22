# Cookbook: Multi-Step Form Automation

Automate a multi-step form submission using Cortex sessions.

## Scenario

Fill out a 3-step registration form:
1. Enter personal information
2. Select preferences
3. Submit and confirm

## Python

```python
import cortex_client

# Map the site
site = cortex_client.map("app.example.com", max_render=20)

# Find the registration form page
forms = site.filter(page_type=7)  # FormPage
reg_form = forms[0]

# Step 1: Fill personal info
session_id = "reg-session-1"

site.act(
    node=reg_form.index,
    opcode=(0x03, 0x00),  # Fill input
    params={"selector": "#name", "value": "Jane Doe"},
    session_id=session_id,
)

site.act(
    node=reg_form.index,
    opcode=(0x03, 0x00),
    params={"selector": "#email", "value": "jane@example.com"},
    session_id=session_id,
)

# Click "Next"
site.act(
    node=reg_form.index,
    opcode=(0x01, 0x00),  # Click
    params={"selector": "button.next"},
    session_id=session_id,
)

# Step 2: Select preferences
site.act(
    node=reg_form.index,
    opcode=(0x03, 0x02),  # Select option
    params={"selector": "#plan", "value": "premium"},
    session_id=session_id,
)

site.act(
    node=reg_form.index,
    opcode=(0x01, 0x00),
    params={"selector": "button.next"},
    session_id=session_id,
)

# Step 3: Submit
result = site.act(
    node=reg_form.index,
    opcode=(0x03, 0x05),  # Form submit
    session_id=session_id,
)

if result.success:
    print(f"Registration complete! Redirected to: {result.new_url}")
```

## Tips

- Always use a `session_id` to maintain cookies across steps
- Sessions expire after 1 hour of inactivity
- Use `site.refresh()` to update the map if the form adds new pages
