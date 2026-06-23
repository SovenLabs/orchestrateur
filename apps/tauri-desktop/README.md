# Orchestrateur Desktop — Tauri 2 + Svelte 5

Application desktop hybride (Phases 21–26). Interface J.A.R.V.I.S. branchée sur le daemon `ws://127.0.0.1:28790/ws`, avec lancement Godot intégré.

**Design system :** [`../../docs/UI_EVENT_HORIZON.md`](../../docs/UI_EVENT_HORIZON.md)  
**Architecture v2 :** [`../../docs/architecture.md`](../../docs/architecture.md)

## Prérequis

- Rust 1.80+ (workspace racine)
- Node.js 20+
- Daemon actif : `just daemon` ou `orch daemon run --workspace workspace`
- Godot 4.7 (optionnel) — pour **Ouvrir Sphère** / **Territoire Godot**

## Développement

```powershell
cp .env.example .env   # optionnel
npm install
npm run tauri dev
```

## Commandes Tauri (Phase 25+)

| Commande Rust | Rôle |
|---------------|------|
| `launch_sphere_window` | Ouvre `SphereDedicated.tscn` ou export |
| `launch_territory_window` | Ouvre `MainTerritory.tscn` |
| `get_territory_launch_status` | PID processus Godot spawnés |

Raccourcis UI : **⌘K** → palette (inclut lancement Sphère / Territoire).

## Types partagés

```powershell
npm run export-types
# équivalent : cargo run -p shared-types --bin export-ts
```

## Build release

```powershell
# Depuis la racine du repo
just release-v26
# ou : npm run tauri build (desktop seul)
```

Installateurs : `src-tauri/target/release/bundle/nsis/` ou `msi/`.

## Tests

```powershell
npm test
```