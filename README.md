# Orchestrateur — Second cerveau local souverain

**Version Cargo workspace : 0.18.0** · **Rust 1.80+** · **Juin 2026**

> Documentation architecte : [`docs/prompt/PROMPT_MAITRE.md`](docs/prompt/PROMPT_MAITRE.md) · Archives phases : [`docs/`](docs/)

---

## 1. Identité et philosophie

Le projet s’appelle **Orchestrateur**. Il est conçu pour durer **7 à 10 ans**, en **local et souverain**, avec un **contrôle total** à chaque niveau.

### Mantra d’architecture

> **Cortex first, agent second, gateway third.**

| Priorité | Couche | Rôle |
|----------|--------|------|
| **1** | **Cortex** | Le **Squelette** — mémoires Markdown, graphe, LanceDB, ports hexagonaux |
| **2** | **Agent** | L’**Esprit** — LLM + outils mémoire natifs, au service du Cortex |
| **3** | **Gateway** | **Canal d’entrée optionnel** — WS, webhooks, messagerie (pas le cœur du produit) |
| — | **Territoire Graphique** | Godot 4 + daemon WS — client visuel remplaçable |
| — | **CLI** | Headless, scripts, `daemon run` |

Le **cerveau et l’IA sont fusionnés** : l’agent n’est pas un chatbot autonome, c’est l’opérateur du Cortex. Chaque tour peut assimiler, rechercher et enrichir le graphe — pas seulement répondre.

**Hiérarchie non négociable** : Cortex → Orchestrateur (agent) → Peau / gateway. La solidité du squelette prime sur tout rendu visuel ou connecteur messaging.

### Différenciateurs (cœur du produit)

Ces capacités définissent Orchestrateur — pas la liste des canaux messaging :

| Capacité | Détail |
|----------|--------|
| **Auto-assimilation** | Chaque tour agent peut persister un résumé en Cortex (`auto_assimilate_turn = true` par défaut) |
| **Graphe de connaissances** | Hubs, backlinks, contexte graphe injecté dans le prompt agent |
| **Recherche proactive** | Souvenirs pertinents chargés *avant* l’appel LLM |
| **Honeypots & intégrité** | Mémoires canari, empreinte BLAKE3 config, audit chaîné tamper-evident |
| **Profils de capacités** | Groupes d’outils Cortex (`memory`, `agent`, `research`, …) — pas un catalogue de plugins distant |

**Skills** : extensions optionnelles de l’Esprit (subprocess, natif, marketplace local) — jamais le substitut du Cortex.

---

## 2. Architecture du dépôt

```
Orchestre/
├── crates/
│   ├── cortex/           # Domaine + ports (MemoryRepository, VectorStore, EmbeddingProvider)
│   ├── orchestrator/     # Application (facade, bridge, daemon WS, gateway)
│   ├── infrastructure/   # Adapters (LanceDB, Ollama, xAI, filesystem)
│   ├── cli/              # Binaire orchestrateur.exe (CLI + daemon)
│   └── client/           # Client bridge embarqué
├── territoire-graphique/ # Client Godot 4 (Phase 15+)
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
| `orchestrator` | Use cases, facade, bridge, daemon WS (`websocket-server`) | Ne connaît que les ports Cortex |
| `infrastructure` | Implémentations concrètes des ports | Injectée au démarrage des binaires |
| `cli` | Point d’entrée cœur (terminal) | CLI pur ou TUI selon contexte |
| `territoire-graphique` | Rendu visuel Godot 4 | Client WS — remplaçable |

Voir [`territoire-graphique/communication.md`](territoire-graphique/communication.md) pour le protocole WS Option B (daemon port 28790).

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
cargo build -p territoire-gdextension
```

### Build release (distribution)

```powershell
cargo build --release -p orchestrateur-cli --features gateway,websocket-server
```

Exécutable produit :

| Fichier | Rôle |
|---------|------|
| `target\release\orchestrateur.exe` | CLI headless + daemon WS + gateway |

### Lancement (exemples)

```powershell
# Daemon Territoire Graphique (port 28790 — client Godot)
$env:ORCHESTRATEUR_DAEMON_TOKEN = "secret"
.\target\release\orchestrateur.exe daemon run --workspace workspace

# CLI pur
.\target\release\orchestrateur.exe list --workspace workspace
.\target\release\orchestrateur.exe get <uuid> --workspace workspace

# Gateway WebSocket (port 28789 — canaux messaging, optionnel)
$env:ORCHESTRATEUR_GATEWAY_TOKEN = "secret"
.\target\release\orchestrateur.exe gateway run --workspace workspace

# Canaux gateway + profils de capacités agent
.\target\release\orchestrateur.exe channels list
.\target\release\orchestrateur.exe capability-profiles list

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
| Recherche sémantique | ⚠ Désactivée — erreur bridge / daemon |
| Assimilation LLM | ⚠ Désactivée — erreur bridge / daemon |

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
cargo test -p orchestrator --features websocket-server
cargo test -p mcp
```

Lister les providers, profils de capacités et hub skills :

```powershell
orchestrateur providers list
orchestrateur providers list --kind llm
orchestrateur capability-profiles list
orchestrateur skills-hub list
orchestrateur skills-hub marketplace
orchestrateur skills-hub sync
orchestrateur skills-hub verify
orchestrateur channels list    # gateway optionnel (18 canaux)
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
| 10 | 18 canaux gateway, profils de capacités, auto-assimilation par tour, `[agent]` TOML |
| 11 | Skills hub filesystem, plugins subprocess, `[skills_hub]` TOML |
| 12 | Plugins natifs, inbound stub HTTP, outils agent `skill_*` |
| 13 | Marketplace skills, intégrité BLAKE3, `skill_suggest` + auto-suggest |
| 14 | Polling HTTP stub, catalogue signé BLAKE3, `skill_auto_execute`, bridge marketplace |
| 14 bis | Suppression egui/ratatui, daemon WS :28790, `territoire-graphique/` Godot |
| 15 | Boule de Pixels Vivante Godot, shader, particules, monitoring, client WS |
| 16 | Shader v2, particules seuillées, WebSocket GDExtension (tokio-tungstenite) |

**Packaging Windows** :
```powershell
.\scripts\build-installer.ps1 -InstallInno   # Setup.exe + zip
.\scripts\package-windows.ps1                # zip uniquement
```
Artefacts dans `dist/` : `Orchestrateur-v0.5.0-Setup-win64.exe`, `Orchestrateur-v0.5.0-win64.zip`

---

*Orchestrateur — Cortex first, séparation stricte des Peaux, contrôle total.*