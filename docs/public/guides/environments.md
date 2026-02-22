# Environment Setup Guide

## Docker

```dockerfile
FROM rust:1.82 AS builder
WORKDIR /app
COPY runtime/ .
RUN cargo build --release

FROM debian:bookworm-slim
# Chromium is optional — only needed for browser fallback and ACT operations
# Mapping works without it via HTTP-first layered acquisition
RUN apt-get update && apt-get install -y chromium && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/cortex /usr/local/bin/
ENV CORTEX_CHROMIUM_PATH=/usr/bin/chromium
CMD ["cortex", "start"]
```

## CI/CD

```yaml
- name: Install Cortex
  run: cargo install --path runtime/

- name: Install Chromium
  run: cortex install

- name: Run tests
  run: |
    cortex start &
    sleep 2
    cortex doctor
    cortex stop
```

## ARM (Apple Silicon / ARM64 Linux)

Cortex supports ARM natively. Chrome for Testing provides ARM64 builds:

```bash
cortex install  # Auto-detects architecture
cortex doctor   # Verify setup
```

## Air-Gapped Environments

1. Download Chromium separately and place in `~/.cortex/chromium/`
2. Set `CORTEX_CHROMIUM_PATH` to the binary path
3. Build Cortex from source: `cargo build --release`

## Serverless

Cortex runs as a local daemon and is designed for persistent
environments. For serverless, pre-map sites and bundle the `.ctx`
files with your deployment.
