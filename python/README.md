# agentic-vision

Binary web cartography for AI agents.

## Installation

```bash
pip install agentic-vision
```

## Quick Start

```python
import agentic_vision

print(agentic_vision.__version__)
```

## Development

```bash
# Build the native library
cargo build --release

# Install in dev mode
pip install -e "python/[dev]"

# Run tests
pytest python/tests/ -v
```

## License

MIT
