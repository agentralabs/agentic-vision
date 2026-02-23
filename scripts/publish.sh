#!/bin/bash
# Pre-publish checks and dry-run for crates.io
set -euo pipefail

echo "Running pre-publish checks..."
echo ""

echo "1. Running tests..."
cargo test --workspace
echo ""

echo "2. Checking formatting..."
cargo fmt --all -- --check
echo ""

echo "3. Running clippy..."
cargo clippy --workspace -- -D warnings
echo ""

echo "4. Dry-run publish (paired crates: core library)..."
cargo publish -p agentic-vision --dry-run
echo ""

echo "5. Dry-run publish (paired crates: MCP server)..."
# Core crate may not be indexed yet for brand-new versions, so use --no-verify
# for MCP dry-run packaging validation at this stage.
cargo publish -p agentic-vision-mcp --dry-run --no-verify
echo ""

echo "All checks passed!"
echo ""
echo "To publish:"
echo "  cargo publish -p agentic-vision"
echo "  # Wait for it to be available on crates.io"
echo "  cargo publish -p agentic-vision-mcp"
