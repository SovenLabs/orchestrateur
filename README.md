# Orchestrateur — Second cerveau local souverain

**Version Cargo workspace : 0.3.0** · **Rust 1.80+** · **Juin 2026**

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

## 4. Compilation et binaires

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
```

CLI sans TUI (binaire plus léger) :

```powershell
cargo build --release -p orchestrateur-cli --no-default-features
```

---

## 5. Mode local sans IA (dégradé)

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

## 6. Tests

```powershell
cargo test -p cortex
cargo test -p orchestrator
cargo test -p orchestrator --features tui
cargo test -p infrastructure
```

---

## 7. Documentation

| Document | Contenu |
|----------|---------|
| [`docs/prompt/PROMPT_MAITRE.md`](docs/prompt/PROMPT_MAITRE.md) | Source de vérité architecte |
| [`docs/prompt/PROMPT_VEILLE.md`](docs/prompt/PROMPT_VEILLE.md) | Protocole veille technologique |
| [`docs/ARCHIVE_PHASE1.md`](docs/ARCHIVE_PHASE1.md) | Clôture Phase 1 |
| [`docs/ARCHIVE_PHASE2_v0.1.0.md`](docs/ARCHIVE_PHASE2_v0.1.0.md) | Clôture Phase 2 |
| [`docs/ARCHIVE_PHASE3_v0.1.0.md`](docs/ARCHIVE_PHASE3_v0.1.0.md) | Clôture Phase 3 |

Tags Git : `phase1-closed`, `phase2-closed`, `phase3-v0.1.0`, `phase4-v0.3.0`.

---

## 8. Phases livrées (résumé)

| Phase | Livrable |
|-------|----------|
| 1 | Cortex domaine + ports |
| 2 | Orchestrator facade + use cases + mocks |
| 3 | Infrastructure LanceDB/Ollama/xAI + sécurité |
| 4 | Bridge HUD, CLI enrichi, egui virtualisé |
| 5 | TUI ratatui intégré au cœur, mode dégradé sans IA |
| 6 | Skills opérationnelles, bridge chat/skills, HUD/TUI chat, packaging Windows |

**Packaging Windows** : `.\scripts\package-windows.ps1`

---

*Orchestrateur — Cortex first, séparation stricte des Peaux, contrôle total.*