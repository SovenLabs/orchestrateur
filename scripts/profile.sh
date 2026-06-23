#!/usr/bin/env bash
# Profiling des goulots Orchestrateur (Linux/macOS + cargo-flamegraph optionnel)
# Usage : ./scripts/profile.sh [crate]

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CRATE="${1:-orchestrator}"
PROFILE_DIR="$ROOT/target/profiles"
mkdir -p "$PROFILE_DIR"

echo "==> Benchmarks ignorés (charge)"
cargo test -p cortex --test scalability -- --ignored --nocapture 2>&1 | tee "$PROFILE_DIR/cortex-scalability.log" || true
cargo test -p orchestrator --test load_workspace_scale -- --ignored --nocapture 2>&1 | tee "$PROFILE_DIR/orch-load.log" || true

if command -v cargo-flamegraph >/dev/null 2>&1; then
  echo "==> Flamegraph (nécessite perf sur Linux)"
  CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p "$CRATE" --root -- test phase7_multi_turn --nocapture || true
  mv flamegraph.svg "$PROFILE_DIR/flamegraph-$CRATE.svg" 2>/dev/null || true
else
  echo "cargo-flamegraph non installé — cargo install flamegraph"
fi

echo "==> Timings cargo (référence)"
{ time cargo test -p orchestrator --test phase7_agent_loop; } 2>&1 | tee "$PROFILE_DIR/phase7-timing.log"

echo "Résultats dans $PROFILE_DIR"