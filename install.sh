#!/usr/bin/env bash
# Orchestrateur — installateur Unix (style Hermes)
#
#   curl -fsSL https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.sh | bash
#
# Dev (depuis clone) :
#   ORCHESTRATEUR_DEV=1 ./install.sh
#
# Version release fixe :
#   ORCHESTRATEUR_VERSION=0.28.0 curl -fsSL ... | bash

set -eu

REPO="SovenLabs/orchestrateur"
REPO_URL="https://github.com/${REPO}.git"
RAW_BASE="https://raw.githubusercontent.com/${REPO}/main"

ORCHESTRATEUR_HOME="${ORCHESTRATEUR_HOME:-${HOME}/.local/share/orchestrateur}"
INSTALL_DIR="${ORCHESTRATEUR_HOME}/orchestrateur"
BIN_DIR="${HOME}/.orchestrateur/bin"
WORKSPACE="${XDG_CONFIG_HOME:-${HOME}/.config}/Orchestrateur/workspace"
BRANCH="${ORCHESTRATEUR_BRANCH:-main}"

info()  { printf '-> %s\n' "$1"; }
ok()    { printf '[OK] %s\n' "$1"; }
warn()  { printf '[!] %s\n' "$1" >&2; }
fail()  { printf '[X] %s\n' "$1" >&2; exit 1; }

os="$(uname -s 2>/dev/null || echo unknown)"

case "$os" in
    MINGW*|MSYS*|CYGWIN*|Windows*)
        if command -v powershell.exe >/dev/null 2>&1; then
            info "Windows detecte — delegation vers scripts/install.ps1"
            exec powershell.exe -NoProfile -ExecutionPolicy Bypass -Command \
                "iex (irm '${RAW_BASE}/scripts/install.ps1')"
        fi
        fail "PowerShell requis sous Windows."
        ;;
esac

banner() {
    echo ""
    echo "+---------------------------------------------------------+"
    echo "|           * Orchestrateur Installer                     |"
    echo "+---------------------------------------------------------+"
    echo "|  Second cerveau local souverain — Soven Labs            |"
    echo "+---------------------------------------------------------+"
    echo ""
}

ensure_git() {
    if command -v git >/dev/null 2>&1; then
        ok "Git found ($(git --version))"
        return 0
    fi
    fail "git requis. Installez git puis relancez l'installateur."
}

ensure_rust() {
    if command -v cargo >/dev/null 2>&1; then
        ok "Rust found ($(cargo --version))"
        return 0
    fi
    info "Installation de rustup..."
    curl -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    # shellcheck disable=SC1091
    source "${HOME}/.cargo/env"
    command -v cargo >/dev/null 2>&1 || fail "rustup installation echouee"
    ok "Rust installe"
}

clone_or_update_repo() {
    mkdir -p "$(dirname "$INSTALL_DIR")"
    if [ -d "${INSTALL_DIR}/.git" ]; then
        info "Mise a jour du depot existant..."
        git -C "$INSTALL_DIR" config core.autocrlf false 2>/dev/null || true
        git -C "$INSTALL_DIR" stash push -u -m "orchestrateur-install-autostash" 2>/dev/null || true
        git -C "$INSTALL_DIR" fetch origin "$BRANCH"
        git -C "$INSTALL_DIR" reset --hard "origin/${BRANCH}"
        git -C "$INSTALL_DIR" stash pop 2>/dev/null || true
        ok "Depot mis a jour"
        return 0
    fi
    info "Clone de ${REPO_URL} ..."
    git clone --branch "$BRANCH" "$REPO_URL" "$INSTALL_DIR"
    git -C "$INSTALL_DIR" config core.autocrlf false 2>/dev/null || true
    ok "Depot clone"
}

build_cli() {
    info "Compilation release (orch)..."
    cd "$INSTALL_DIR"
    # shellcheck disable=SC1091
    [ -f "${HOME}/.cargo/env" ] && source "${HOME}/.cargo/env"
    export RUSTFLAGS="-D warnings"
    cargo build --release -p orchestrateur-cli --bin orch --features gateway,websocket-server
    ok "Build termine"
}

install_binaries() {
    mkdir -p "$BIN_DIR"
    cp "${INSTALL_DIR}/target/release/orch" "${BIN_DIR}/orch"
    chmod +x "${BIN_DIR}/orch"
    ln -sf orch "${BIN_DIR}/orchestre"
    ln -sf orch "${BIN_DIR}/orchestrateur"
    ok "Binaires installes dans ${BIN_DIR}"
}

setup_path() {
    case ":${PATH}:" in
        *":${BIN_DIR}:"*) ok "PATH deja configure" ;;
        *)
            export PATH="${BIN_DIR}:${PATH}"
            shell_rc=""
            if [ -n "${ZSH_VERSION:-}" ] && [ -f "${HOME}/.zshrc" ]; then
                shell_rc="${HOME}/.zshrc"
            elif [ -f "${HOME}/.bashrc" ]; then
                shell_rc="${HOME}/.bashrc"
            elif [ -f "${HOME}/.profile" ]; then
                shell_rc="${HOME}/.profile"
            fi
            if [ -n "$shell_rc" ] && ! grep -qF "$BIN_DIR" "$shell_rc" 2>/dev/null; then
                {
                    echo ""
                    echo "# Orchestrateur CLI"
                    echo "export PATH=\"${BIN_DIR}:\$PATH\""
                    echo "export ORCHESTRATEUR_HOME=\"${ORCHESTRATEUR_HOME}\""
                } >> "$shell_rc"
                ok "PATH ajoute a ${shell_rc}"
            else
                warn "Ajoutez manuellement au PATH : export PATH=\"${BIN_DIR}:\$PATH\""
            fi
            ;;
    esac
}

init_workspace() {
    mkdir -p "${WORKSPACE}/config" "${WORKSPACE}/memories" "${WORKSPACE}/logs"
    example="${INSTALL_DIR}/workspace/config/orchestrator.toml.example"
    config="${WORKSPACE}/config/orchestrator.toml"
    if [ ! -f "$config" ] && [ -f "$example" ]; then
        cp "$example" "$config"
        ok "Config creee : $config"
    else
        ok "Workspace pret : $WORKSPACE"
    fi
}

write_marker() {
    mkdir -p "$ORCHESTRATEUR_HOME"
    commit="$(git -C "$INSTALL_DIR" rev-parse HEAD 2>/dev/null || echo "")"
    printf '{"schema_version":1,"install_mode":"build","pinned_branch":"%s","pinned_commit":"%s"}\n' \
        "$BRANCH" "$commit" > "${ORCHESTRATEUR_HOME}/.orchestrateur-bootstrap-complete"
    ok "Marqueur bootstrap ecrit"
}

completion() {
    echo ""
    echo "+---------------------------------------------------------+"
    echo "|              [OK] Installation Complete!                |"
    echo "+---------------------------------------------------------+"
    echo ""
    echo "* Workspace : ${WORKSPACE}"
    echo "* Code      : ${INSTALL_DIR}"
    echo ""
    echo "Commandes :"
    echo "  orch doctor --workspace \"${WORKSPACE}\""
    echo "  orch onboard --workspace \"${WORKSPACE}\""
    echo "  orch daemon run --workspace \"${WORKSPACE}\""
    echo ""
    echo "[*] Ouvrez un nouveau terminal pour le PATH"
    echo ""
}

main() {
    banner
    if [ "${ORCHESTRATEUR_DEV:-}" = "1" ] && [ -f "./Cargo.toml" ]; then
        INSTALL_DIR="$(pwd)"
        info "Mode dev — depot local : ${INSTALL_DIR}"
    fi
    ensure_git
    clone_or_update_repo
    ensure_rust
    build_cli
    install_binaries
    setup_path
    init_workspace
    write_marker
    if command -v "${BIN_DIR}/orch" >/dev/null 2>&1 && [ "${ORCHESTRATEUR_SKIP_DOCTOR:-}" != "1" ]; then
        info "Verification harness (orch doctor)..."
        "${BIN_DIR}/orch" doctor --workspace "$WORKSPACE" || warn "doctor a signale des problemes"
    fi
    completion
}

main "$@"