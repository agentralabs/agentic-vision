# Implementation Delta (Vision) - 2026-02-23

## Purpose

This file records implemented work that is active but not explicitly represented in the current planning-doc baseline.

Baseline planning docs reviewed:
- `planning-docs/CANONICAL_SISTER_KIT.md`
- `planning-docs/SDK_READINESS.md`

## Implemented Post-Plan Items

### 1. Installer and runtime hardening

Implemented:
- Install profiles for desktop/terminal/server.
- `jq` merge with `python3` fallback for MCP config updates.
- Server auth gating (`AGENTIC_TOKEN` in server mode).
- Wrapper root resolution + local `.avis` auto-detect startup guidance.

Evidence:
- `scripts/install.sh:11`
- `scripts/install.sh:191`
- `scripts/install.sh:324`
- `scripts/install.sh:427`
- `scripts/install.sh:776`

### 2. MCP robustness and validation

Implemented:
- Strict argument validation for `vision_query`, `vision_similar`, `vision_track`.
- Stdio shutdown exits immediately after successful `shutdown`.
- Framing hardening for Content-Length style clients.

Evidence:
- `CHANGELOG.md:11-13`
- `CHANGELOG.md:18-19`
- `CHANGELOG.md:29`

### 3. Release workflow hardening

Implemented:
- Duplicate publish handling expanded to include crates.io `already exists on crates.io index` wording.

Evidence:
- `.github/workflows/release.yml:99`
- `.github/workflows/release.yml:118`

### 4. Social workflow operational changes

Implemented:
- Secret-check logic fixed to avoid invalid workflow evaluation.
- Manual `workflow_dispatch` support added.
- X bridge integration currently removed.

Evidence:
- `.github/workflows/social-release-broadcast.yml`

## Planning Drift Note

`CHANGELOG.md` currently has release entries through `0.1.6`, while current crate release state includes `0.1.7` (publish/release workflow activity). Add changelog parity entry in future planned updates.

Cross-sister reference:
- `agentic-memory/planning-docs/CONSISTENCY-VALIDATION-ACROSS-SISTERS-2026-02-23.md`
