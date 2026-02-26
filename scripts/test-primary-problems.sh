#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

assert_contains() {
  local text="$1"
  local pattern="$2"
  local label="$3"
  if command -v rg >/dev/null 2>&1; then
    printf '%s' "$text" | rg -q --fixed-strings "$pattern" || fail "${label}: missing '${pattern}'"
  else
    printf '%s' "$text" | grep -q -F -- "$pattern" || fail "${label}: missing '${pattern}'"
  fi
}

run_vision() {
  cargo run --quiet --bin agentic-vision-mcp -- "$@"
}

echo "[1/4] Validate MCP surface for visual-state and non-text-signal tools"
info_out="$(run_vision info)"
assert_contains "$info_out" '"tool_count": 21' "vision info"
assert_contains "$info_out" '"vision_capture"' "vision info"
assert_contains "$info_out" '"vision_query"' "vision info"
assert_contains "$info_out" '"vision_health"' "vision info"
assert_contains "$info_out" '"vision_diff"' "vision info"

echo "[2/4] Validate input safety around visual capture and query surfaces"
cargo test --quiet -p agentic-vision-mcp --test edge_cases test_04_invalid_params_no_source
cargo test --quiet -p agentic-vision-mcp --test edge_cases test_bonus_vision_query_param_validation

echo "[3/4] Validate non-text signal workflows and tracking config guards"
cargo test --quiet -p agentic-vision-mcp --test edge_cases test_bonus_vision_track_param_validation
cargo test --quiet -p agentic-vision-mcp --test edge_cases test_14_empty_description

echo "[4/4] Primary vision problem checks passed (P23,P24)"
