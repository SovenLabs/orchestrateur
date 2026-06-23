#!/usr/bin/env sh
# Orchestrateur — installateur unique (délègue vers install.ps1 sous Windows)
#   curl -fsSL https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.sh | sh
# Dev (depuis clone) :
#   ORCHESTRATEUR_DEV=1 ./install.sh
# Version fixe release :
#   ORCHESTRATEUR_VERSION=0.28.0 curl -fsSL ... | sh

set -eu

REPO="SovenLabs/orchestrateur"
RAW_BASE="https://raw.githubusercontent.com/${REPO}/main"

os="$(uname -s 2>/dev/null || echo unknown)"

case "$os" in
    MINGW*|MSYS*|CYGWIN*|Windows*)
        if command -v powershell.exe >/dev/null 2>&1; then
            if [ "${ORCHESTRATEUR_DEV:-}" = "1" ] && [ -f "./install.ps1" ]; then
                echo "Mode dev — install.ps1 -Dev"
                exec powershell.exe -NoProfile -ExecutionPolicy Bypass -File ./install.ps1 -Dev
            fi
            echo "Windows detecte — delegation vers install.ps1 (release)"
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
  cargo build --release -p orchestrateur-cli --bin orch --features gateway,websocket-server
  # Binaire : target/release/orch (alias clap : orchestre, orchestrateur)

Windows (installateur unique):
  irm ${RAW_BASE}/install.ps1 | iex          # release
  .\\install.ps1 -Dev                        # dev (depuis le clone)

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