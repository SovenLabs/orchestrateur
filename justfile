# Orchestrateur v2 — commandes de build (just)

set shell := ["powershell.exe", "-NoProfile", "-Command"]

default:
    @just --list

# Compile tout le workspace Rust
build:
    cargo build --workspace

# Tests workspace (hors heavy-tests)
test:
    cargo test --workspace

# Tests daemon WS + protocole
test-ws:
    cargo test -p orchestrator --features websocket-server
    cargo test -p shared-types

# Tests client TypeScript
desktop-test:
    cd apps/tauri-desktop; npm test

# Exporte les types TypeScript vers le frontend Tauri
export-types:
    cargo run -p shared-types --bin export-ts

# Lance le daemon territorial (token dev)
daemon:
    $env:ORCHESTRATEUR_DAEMON_TOKEN = "dev"; cargo run --bin orchestrateur -- daemon run --workspace workspace

# Installe les dépendances npm du desktop
desktop-install:
    cd apps/tauri-desktop; npm install

# Dev Tauri + Svelte (daemon requis dans un autre terminal)
desktop-dev:
    cd apps/tauri-desktop; npm run tauri dev

# Build release desktop
desktop-build:
    cd apps/tauri-desktop; npm run tauri build

# Pipeline Phase 21 : types + build Rust + install frontend
phase21-setup: export-types build desktop-install

# Export standalone sphère Godot (nécessite Godot 4.7 dans PATH)
export-sphere:
    powershell -NoProfile -ExecutionPolicy Bypass -File territoire-graphique/scripts/export-sphere.ps1

# Pipeline Phase 25 : types + hub + desktop
phase25-setup: export-types build desktop-install

# Build release hybride v2 (CLI + Tauri + sphère optionnelle) — Phase 26
release-v26:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-release-v2.ps1

# Release CLI seul (zip classique)
release-cli:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-installer.ps1 -ZipOnly

# Tag Git phase26-v0.26.0
tag-phase26:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/tag-phase-release.ps1 -Phase 26

# Tag Git phase27-v0.27.0
tag-phase27:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/tag-phase-release.ps1 -Phase 27

# Pipeline Phase 26 : setup + tests
phase26-validate: export-types test-ws desktop-test

# Pipeline Phase 27 : types + tests desktop cosmiques
phase27-validate: export-types desktop-test