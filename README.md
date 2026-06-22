# Orchestrateur — Second cerveau local souverain

**Version Cargo workspace : 0.13.0** · **Rust 1.80+** · **Juin 2026**

> Documentation architecte : [`docs/prompt/PROMPT_MAITRE.md`](docs/prompt/PROMPT_MAITRE.md) · Archives phases : [`docs/`](docs/)

---

## 1. Identité et philosophie

Le projet s’appelle **Orchestrateur**. Il est conçu pour durer **7 à 10 ans**, en **local et souverain**, avec un **contrôle total** à chaque niveau.

| Couche | Rôle | Priorité |
|--------|------|----------|
| **Cortex** | Le **Squelette** — domaine, ports hexagonaux, mémoires Markdown, graphe, recherche | **n°1** |
| **Orchestrateur** | L’**Esprit** — use cases, facade, bridge, sécurité, skills | **n°2** |
| **Peau** | Interface optionnelle et remplaçable (HUD egui, TUI ratatui, CLI) | Optionnelle |

**Hiérarchie non négociable** : Cortex → Orchestrateur → Peau. La solidité du squelette prime sur tout rendu visuel.

**Skills** (futur) : capacités ajoutées à l’Esprit (voix, images, trading, etc.).

---

## 2. Architecture du dépôt

```
Orchestre/
├── crates/
│   ├── cortex/           # Domaine + ports (MemoryRepository, VectorStore, EmbeddingProvider)
│   ├── orchestrator/     # Application (facade, use cases, bridge, security, tui/)
│   ├── infrastructure/   # Adapters (LanceDB, Ollama, xAI, filesystem)
│   ├── cli/              # Binaire orchestrateur.exe (CLI + TUI)
│   └── hud/              # Binaire orchestrateur-hud.exe (egui)
├── workspace/            # Données utilisateur (hors code source)
│   ├── config/
│   ├── memories/
│   ├── logs/
│   └── .orchestrateur/   # LanceDB (généré au runtime)
├── docs/
│   ├── ARCHIVE_PHASE*.md
│   └── prompt/
└── Cargo.toml            # Workspace Rust
```

### Couches et règles d’or

| Crate | Responsabilité | Règle |
|-------|----------------|-------|
| `cortex` | Entités, value objects, ports, services purs | Zéro dépendance infra / orchestrator |
| `orchestrator` | Use cases, facade, bridge HUD/TUI, TUI (feature `tui`) | Ne connaît que les ports Cortex |
| `infrastructure` | Implémentations concrètes des ports | Injectée au démarrage des binaires |
| `cli` | Point d’entrée cœur (terminal) | CLI pur ou TUI selon contexte |
| `hud` | Peau graphique egui | Consomme uniquement le bridge |

Le **TUI** (ratatui) vit dans `orchestrator/src/tui/`, compilé via la feature `tui` du binaire `orchestrateur` — **pas** dans `hud/`.

---

## 3. Workspace utilisateur

Tous les binaires prennent `--workspace <chemin>` (défaut : `./workspace`).

```
workspace/
├── config/
│   └── orchestrator.toml.example   # Copier vers orchestrator.toml
├── memories/                       # Fichiers Markdown (<uuid-v7>.md)
├── logs/
└── .orchestrateur/
    └── lancedb/                    # Index vectoriel local
```

**Première utilisation :**

```powershell
Copy-Item workspace\config\orchestrator.toml.example workspace\config\orchestrator.toml
```

Configurer `type = "lancedb"` dans `[vector_store]`. Les providers IA (xAI, Ollama) sont **optionnels** : voir §5.

---

## 4. Installation utilisateur (one-liner)

Les binaires Windows sont publiés sur **[GitHub Releases](https://github.com/SovenLabs/orchestrateur/releases)**.  
Les scripts d’installation sont servis depuis la branche `main` (raw GitHub).

### Windows — PowerShell

```powershell
irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1 | iex
```

Version fixe, installation silencieuse :

```powershell
$env:ORCHESTRATEUR_VERSION = "0.5.0"
$env:ORCHESTRATEUR_SILENT = "1"
irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1 | iex
```

Alternative (télécharger puis exécuter, plus lisible) :

```powershell
irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1 -OutFile install.ps1
.\install.ps1
```

### Git Bash / WSL sous Windows

```bash
curl -fsSL https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.sh | sh
```

### Linux / macOS

Pas de binaire précompilé pour l’instant — `install.sh` affiche les instructions de compilation depuis les sources.

### Où c’est hébergé ?

| Élément | URL |
|---------|-----|
| Scripts `install.ps1` / `install.sh` | `raw.githubusercontent.com/SovenLabs/orchestrateur/main/...` |
| Setup.exe + zip | `github.com/SovenLabs/orchestrateur/releases/download/vX.Y.Z/...` |

### Publier une release (mainteneurs)

```powershell
.\scripts\build-installer.ps1 -InstallInno
.\scripts\publish-github-release.ps1          # necessite `gh auth login`
# ou: git tag v0.5.0 && git push origin v0.5.0   # declenche .github/workflows/release.yml
```

---

## 5. Compilation et binaires

### Prérequis

- Rust stable **1.80+** (`rust-toolchain.toml`)
- `protoc` : fourni via `.tools/` (voir `.cargo/config.toml`)

### Build développement

```powershell
cargo build -p orchestrateur-cli
cargo build -p orchestrateur-hud
```

### Build release (distribution)

```powershell
cargo build --release -p orchestrateur-cli
cargo build --release -p orchestrateur-hud
```

Exécutables produits :

| Fichier | Rôle |
|---------|------|
| `target\release\orchestrateur.exe` | Cœur — CLI, TUI (défaut si terminal interactif) |
| `target\release\orchestrateur-hud.exe` | Peau graphique egui |

### Lancement (exemples)

```powershell
# HUD graphique
.\target\release\orchestrateur-hud.exe --workspace workspace

# TUI (terminal interactif, sans argument)
.\target\release\orchestrateur.exe --workspace workspace

# CLI pur
.\target\release\orchestrateur.exe list --workspace workspace
.\target\release\orchestrateur.exe get <uuid> --workspace workspace

# Gateway WebSocket Phase 8 (port 18789)
$env:ORCHESTRATEUR_GATEWAY_TOKEN = "secret"
.\target\release\orchestrateur.exe gateway run --workspace workspace

# Catalogue canaux (18) et toolsets agent (Phase 10)
.\target\release\orchestrateur.exe channels list
.\target\release\orchestrateur.exe toolsets list

# Hub skills + plugins dynamiques (Phase 11)
.\target\release\orchestrateur.exe skills-hub list
.\target\release\orchestrateur.exe skill run pong
```

CLI sans TUI (binaire plus léger) :

```powershell
cargo build --release -p orchestrateur-cli --no-default-features
```

---

## 6. Mode local sans IA (dégradé)

L’application **démarre toujours** même si xAI/Ollama sont absents ou hors ligne.

| Composant | Comportement si indisponible |
|-----------|------------------------------|
| Liste / détail / filtre mémoires | ✅ Fonctionne (filesystem + LanceDB) |
| Recherche sémantique | ⚠ Désactivée — barre rouge (HUD) ou message (TUI) |
| Assimilation LLM | ⚠ Désactivée — bouton grisé |

Le health check bridge remonte `status: degraded` avec `llm_available` et `embedding_available`.

Variable xAI (optionnelle) :

```powershell
$env:XAI_API_KEY = "sk-..."
```

---

## 7. Tests

### Boucle développeur (rapide, < 15 s)

```powershell
cargo test -p cortex
cargo test -p orchestrator
cargo test -p orchestrator --features gateway
cargo test -p mcp
```

Lister les providers, canaux et toolsets :

```powershell
orchestrateur providers list
orchestrateur providers list --kind llm
orchestrateur channels list
orchestrateur toolsets list
orchestrateur skills-hub list
orchestrateur skills-hub marketplace
orchestrateur skills-hub sync
orchestrateur skills-hub verify
orchestrateur toolsets list   # inclut toolset `skills`
```

Les tests **sécurité**, **charge** et **intégration lourde** sont marqués `#[ignore]` — ils ne tournent pas par défaut.

### Suite complète (sécurité + scalabilité)

```powershell
cargo test -p cortex -- --ignored
cargo test -p orchestrator -- --ignored
cargo test --workspace -- --ignored
```

### Tests très coûteux (feature `heavy-tests`)

```powershell
cargo test -p cortex --features heavy-tests
cargo test -p orchestrator --features heavy-tests
```

### Cibles ciblées

```powershell
cargo test -p cortex --test adversarial_validation -- --ignored
cargo test -p cortex --test scalability -- --ignored
cargo test -p orchestrator --features tui
cargo test -p infrastructure
```

La CI (`.github/workflows/ci.yml`) exécute les tests rapides sur chaque PR, et la suite `--ignored` sur `main`.

---

## 8. Documentation

| Document | Contenu |
|----------|---------|
| [`docs/prompt/PROMPT_MAITRE.md`](docs/prompt/PROMPT_MAITRE.md) | Source de vérité architecte |
| [`docs/prompt/PROMPT_VEILLE.md`](docs/prompt/PROMPT_VEILLE.md) | Protocole veille technologique |
| [`docs/ARCHIVE_PHASE1.md`](docs/ARCHIVE_PHASE1.md) | Clôture Phase 1 |
| [`docs/ARCHIVE_PHASE2_v0.1.0.md`](docs/ARCHIVE_PHASE2_v0.1.0.md) | Clôture Phase 2 |
| [`docs/ARCHIVE_PHASE3_v0.1.0.md`](docs/ARCHIVE_PHASE3_v0.1.0.md) | Clôture Phase 3 |

Tags Git : `phase1-closed`, `phase2-closed`, `phase3-v0.1.0`, `phase4-v0.3.0`.

---

## 9. Phases livrées (résumé)

| Phase | Livrable |
|-------|----------|
| 1 | Cortex domaine + ports |
| 2 | Orchestrator facade + use cases + mocks |
| 3 | Infrastructure LanceDB/Ollama/xAI + sécurité |
| 4 | Bridge HUD, CLI enrichi, egui virtualisé |
| 5 | TUI ratatui intégré au cœur, mode dégradé sans IA |
| 6 | Skills opérationnelles, bridge chat/skills, HUD/TUI chat, packaging Windows |
| 7–9 | Agent loop, gateway WS, provider registry (12 LLM + 5 embeddings), MCP stdio |
| 10 | 18 canaux, 6 toolsets, auto-assimilation par tour, `[agent]` TOML |
| 11 | Skills hub filesystem, plugins subprocess, `[skills_hub]` TOML |
| 12 | Plugins natifs, inbound stub HTTP, outils agent `skill_*` |
| 13 | Marketplace skills, intégrité BLAKE3, `skill_suggest` + auto-suggest |
| 14 | Polling HTTP stub, catalogue signé BLAKE3, `skill_auto_execute`, bridge marketplace |

**Packaging Windows** :
```powershell
.\scripts\build-installer.ps1 -InstallInno   # Setup.exe + zip
.\scripts\package-windows.ps1                # zip uniquement
```
Artefacts dans `dist/` : `Orchestrateur-v0.5.0-Setup-win64.exe`, `Orchestrateur-v0.5.0-win64.zip`

---

*Orchestrateur — Cortex first, séparation stricte des Peaux, contrôle total.*