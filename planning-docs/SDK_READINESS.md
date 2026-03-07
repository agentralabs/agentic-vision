# SDK Readiness - AgenticVision

Internal-only tracking board for distribution-channel maturity.

## Scope

This board tracks whether AgenticVision package channels are mature enough for long-term public SDK promises.

## Channel State

| Channel | Package(s) | State | Notes |
|---|---|---|---|
| crates.io | `agentic-vision`, `agentic-vision-mcp` | Ready | Paired crates published and documented. |
| GitHub installer | `scripts/install.sh` | Ready | One-line install with MCP merge and legacy asset fallback. |
| PyPI SDK | none (canonical) | In progress | Python clients exist in repo, but no canonical publish contract yet. |
| npm SDK | none (canonical) | In progress | Packaging assets exist, but no confirmed canonical npm release policy yet. |

## SDK Gate Checklist

| Gate | Requirement | Status | Evidence / Follow-up |
|---|---|---|---|
| G1 | Public SDK API contract documented | Needs Work | Define official Python and/or npm API boundaries and stability guarantees. |
| G2 | SemVer + compatibility policy clear per channel | Needs Work | Add per-channel support matrix for crates/PyPI/npm. |
| G3 | Release automation present per channel | Partial | Rust release automation exists; add canonical PyPI/npm release workflows if those channels become official. |
| G4 | Channel docs and examples complete | Partial | Rust/MCP docs are strong; PyPI/npm docs need canonical install+usage docs. |
| G5 | Cross-language parity defined | Needs Work | Decide parity targets: wrapper-only vs full SDK surfaces. |
| G6 | Support boundary stated | Partial | Clarify which non-Rust channels are official vs experimental. |

## Decision

AgenticVision is maintained-SDK ready for Rust+MCP channels. PyPI/npm should ship as official channels only after G1-G6 pass.

## Exit Criteria (for official PyPI/npm)

1. Canonical package names finalized.
2. API surface documented and semver-guarded.
3. CI publish pipeline with smoke tests added.
4. README/INSTALL updated with official support statement.

## Review Cadence

- Update on every tagged release.
- Re-check this board before enabling any new public channel.

Last updated: 2026-02-21
