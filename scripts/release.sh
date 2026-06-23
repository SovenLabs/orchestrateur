#!/usr/bin/env bash
# Orchestrateur — processus de release sémantique
# Usage : ./scripts/release.sh [version]
# Exemple : ./scripts/release.sh 0.29.0

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  VERSION="$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')"
  echo "Version courante : $VERSION"
  read -r -p "Nouvelle version (Entrée = inchangé) : " NEW
  if [[ -n "${NEW:-}" ]]; then
    VERSION="$NEW"
  fi
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Version invalide (attendu MAJOR.MINOR.PATCH) : $VERSION" >&2
  exit 1
fi

echo "==> Mise à jour Cargo.toml workspace → $VERSION"
if command -v sed >/dev/null 2>&1; then
  sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
else
  echo "sed requis" >&2
  exit 1
fi

echo "==> cargo check --workspace"
cargo check --workspace

echo "==> Tests Phase 7"
cargo test -p orchestrator --test phase7_agent_loop
cargo test -p orchestrator --test integration_multi_agents_b212
cargo test -p orchestrator --test phase6_skills

echo "==> Build CLI release"
cargo build -p orchestrateur-cli --bin orch --features gateway,websocket-server --release

echo "==> Prêt pour tag v$VERSION"
echo "    git add Cargo.toml"
echo "    git commit -m \"chore: release v$VERSION\""
echo "    git tag v$VERSION"
echo "    git push && git push origin v$VERSION"