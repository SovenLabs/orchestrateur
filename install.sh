#!/usr/bin/env sh
# Orchestrateur — script d'installation (one-liner)
#   curl -fsSL https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.sh | sh
# Version fixe:
#   ORCHESTRATEUR_VERSION=0.5.0 curl -fsSL ... | sh
#
# Releases binaires: Windows x64 uniquement (Setup.exe).
# Linux / macOS: compilation depuis les sources (voir README).

set -eu

REPO="SovenLabs/orchestrateur"
RAW_BASE="https://raw.githubusercontent.com/${REPO}/main"

os="$(uname -s 2>/dev/null || echo unknown)"

case "$os" in
    MINGW*|MSYS*|CYGWIN*|Windows*)
        if command -v powershell.exe >/dev/null 2>&1; then
            echo "Windows detecte — delegation vers install.ps1"
            exec powershell.exe -NoProfile -ExecutionPolicy Bypass -Command \
                "irm '${RAW_BASE}/install.ps1' | iex"
        fi
        echo "Erreur: PowerShell requis sous Windows." >&2
        exit 1
        ;;
    Linux|Darwin)
        cat <<EOF
Orchestrateur — pas de binaire pre-compile pour ${os} pour l'instant.

Installation depuis les sources:
  git clone https://github.com/${REPO}.git
  cd orchestrateur
  cargo build --release -p orchestrateur-cli --features tui -p orchestrateur-hud

Windows (Setup.exe):
  curl -fsSL ${RAW_BASE}/install.sh | sh
  # ou PowerShell:
  irm ${RAW_BASE}/install.ps1 | iex

Releases Windows:
  https://github.com/${REPO}/releases
EOF
        exit 0
        ;;
    *)
        echo "OS non supporte: ${os}" >&2
        exit 1
        ;;
esac