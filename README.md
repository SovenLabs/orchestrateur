# Orchestrateur — Second cerveau local souverain

**Version Cargo workspace : 0.28.0** · **Protocole WS : 1.2.0** · **Rust 1.80+** · **Juin 2026**

> Hiérarchie : [`docs/project-hierarchy.md`](docs/project-hierarchy.md) · Architecture : [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) · Protocole : [`docs/protocol-ws.md`](docs/protocol-ws.md) · Harness : [`docs/harness-integral.md`](docs/harness-integral.md)
>
> **Phase 7** : [`docs/USER_GUIDE.md`](docs/USER_GUIDE.md) · [`docs/DEVELOPER_GUIDE.md`](docs/DEVELOPER_GUIDE.md) · [`docs/B212_INTEGRATION.md`](docs/B212_INTEGRATION.md) · [`docs/agent-tools.md`](docs/agent-tools.md)

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

| Capacité | Détail |
|----------|--------|
| **Auto-assimilation** | Chaque tour agent peut persister un résumé en Cortex (`auto_assimilate_turn = true` par défaut) |
| **Graphe de connaissances** | Hubs, backlinks, contexte graphe injecté dans le prompt agent |
| **Recherche proactive** | Souvenirs pertinents chargés *avant* l’appel LLM |
| **Honeypots & intégrité** | Mémoires canari, empreinte BLAKE3 config, audit chaîné tamper-evident |
| **Profils de capacités** | Groupes d’outils Cortex (`memory`, `agent`, `research`, …) — pas un catalogue de plugins distant |

**Skills** : extensions optionnelles de l’Esprit (subprocess, natif, marketplace local) — jamais le substitut du Cortex. Schémas : [`docs/skills-schema.md`](docs/skills-schema.md).

---

## 2. v1.2 — Kinds, drafts, watcher, prétraitement

| Fonctionnalité | Résumé |
|----------------|--------|
| **Kinds** | Chaque mémoire porte un type sémantique (`decision`, `dead_end`, `pattern`, `context`, `progress`, `business`) — frontmatter Markdown + couleurs UI |
| **Drafts** | File de revue : brouillons générés (watcher ou agent) → publier / rejeter via daemon WS (`draft_created`, `draft_published`, `draft_discarded`) |
| **Watcher** | `[watcher] enabled = true` — surveille session/git, produit des drafts LLM ; CLI `orchestrateur watcher status` / `draft publish` |
| **Message preprocess** | `[agent] message_preprocess = true` — messages courts/vagues enrichis via graphe + recherche proactive avant le LLM ; événements `MessageExpanded` / `MessageCompressed` |

Desktop Tauri **1.2.0** : onglet Drafts, badges kind cosmiques, contrôle watcher dans le drawer `]`.

---

## 3. Architecture du dépôt

Carte spatiale et priorités P0–P6 : **[`docs/project-hierarchy.md`](docs/project-hierarchy.md)**.

```
orchestrateur/
├── crates/
│   ├── shared-types/     # Protocole WS + export TypeScript
│   ├── cortex/           # P0 — domaine + ports
│   ├── orchestrator/     # P2/P3 — facade, agent, daemon, gateway
│   ├── infrastructure/   # P1 — LanceDB, Ollama, xAI, filesystem
│   ├── mcp/              # P4 — transport MCP stdio
│   ├── cli/              # P4 — harness CLI (cible : orch.exe seul)
│   └── client/           # Bridge embarqué
├── apps/
│   └── tauri-desktop/    # P5 — client WS (cible : apps/desktop-tauri)
├── territoire-graphique/ # P5 — client Godot (cible : apps/godot-territoire)
├── plugins/              # P6 — skills natives
├── workspace/            # Données utilisateur dev (≠ Cargo workspace)
├── scripts/              # install.ps1, release
└── Cargo.toml            # Workspace Rust
```

| Crate | Priorité | Responsabilité |
|-------|----------|----------------|
| `cortex` | P0 | Entités, ports, services purs |
| `infrastructure` | P1 | Adapters mémoire / LLM |
| `orchestrator` | P2–P3 | Facade, bridge, daemon, gateway |
| `mcp` | P4 | Client/serveur MCP |
| `cli` | P4 | Harness headless (`orch` installé ; `orchestre` / `orchestrateur` = alias clap) |
| `apps/tauri-desktop` | P5 | Desktop — client WS du daemon |

Après validation du noyau, **seules les skills (P6)** évoluent librement. Voir [`docs/project-hierarchy.md`](docs/project-hierarchy.md).

Voir aussi [`docs/architecture.md`](docs/architecture.md) et [`territoire-graphique/communication.md`](territoire-graphique/communication.md).

### Lancement développement

```powershell
# Terminal 1 — daemon
just daemon

# Terminal 2 — interface de commandement
just desktop-dev
```

| Commande | Rôle |
|----------|------|
| `just export-types` | Types TS depuis `shared-types` |
| `just test-ws` | Tests protocole daemon |
| `just desktop-test` | Tests Vitest frontend |

---

## 4. Workspace utilisateur

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

Configurer `type = "lancedb"` dans `[vector_store]`. Les providers IA (xAI, Ollama) sont **optionnels** — voir §6.

---

## 5. Installateur unique (`install.ps1`)

Un seul script — deux modes. Binaires release sur **[GitHub Releases](https://github.com/SovenLabs/orchestrateur/releases)**.

> **État Juin 2026 :** le dépôt est en **0.28.0** mais **aucune release GitHub n’est encore publiée** (`Orchestrateur-v*-Setup-win64.exe`).  
> En attendant, utilisez le **mode dev** depuis un clone. Le one-liner `irm | iex` fonctionnera dès qu’une release sera publiée via `scripts/publish-github-release.ps1`.

### Windows — politique d’exécution

Si PowerShell **refuse** `irm | iex` (« exécution de scripts désactivée »), utilisez l’une de ces formes :

```powershell
# Recommandé (bypass + évite le cache irm)
$i = Join-Path $env:TEMP "orchestrateur-install.ps1"
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1" -OutFile $i -UseBasicParsing
powershell -NoProfile -ExecutionPolicy Bypass -File $i

# One-liner (si cache CDN : ajoutez ?t=1 à l'URL)
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm 'https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1?t=3' | iex"

# Ou depuis un clone
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Dev
```

### Modes d’installation

| Mode | Commande | Quand l’utiliser |
|------|----------|------------------|
| **Release** | `powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1 \| iex"` | Après publication d’une release GitHub |
| **Release + daemon auto** | `$env:ORCHESTRATEUR_INSTALL_DAEMON='1';` puis commande release ci-dessus | Post-install avec tâche planifiée daemon |
| **Dev (recommandé aujourd’hui)** | `git clone …` puis `.\install.ps1 -Dev` avec `-ExecutionPolicy Bypass` | Contributeurs, version courante non packagée |
| **Dev + daemon** | `.\install.ps1 -Dev -InstallDaemon` | Dev + daemon au logon |

Après l’install release, le script exécute **`orch doctor`**, initialise le workspace `%APPDATA%\Orchestrateur\workspace`, génère `ORCHESTRATEUR_DAEMON_TOKEN` si absent, et affiche le snippet MCP.

Version fixe release (quand disponible) : `$env:ORCHESTRATEUR_VERSION = "0.28.0"`

### Dépannage rapide

| Symptôme | Cause probable | Action |
|----------|----------------|--------|
| « exécution de scripts désactivée » | `ExecutionPolicy` Windows | `-ExecutionPolicy Bypass` (voir ci-dessus) |
| « Aucune release GitHub publiée » | 0 release sur le dépôt | `git clone` + `.\install.ps1 -Dev` |
| `orch` / `just` introuvable | PATH ou mauvais répertoire | Nouveau terminal après install ; `cd` vers le clone |
| `npm` / `npm.ps1` refusé | Même politique PowerShell | `npm.cmd` au lieu de `npm` |

---

## 6. Compilation et binaires

### Prérequis

- Rust stable **1.80+** (`rust-toolchain.toml`) — [rustup.rs](https://rustup.rs/)
- `protoc` : fourni via `.tools/` (voir `.cargo/config.toml`)
- `just` (optionnel, recettes dev) — `cargo install just` ou `winget install Casey.Just`

### CLI harness

**Installé :** un seul binaire `orch.exe` dans PATH.

**Alias clap** (même programme, pas de `.exe` séparés) : `orchestre`, `orchestrateur` — visibles dans `orch --help`.

Le mode dev (`.\install.ps1 -Dev`) copie `orch.exe` dans `%USERPROFILE%\.cargo\bin` (ou `C:\Program Files\Orchestrateur\` avec `-AllUsers`). **Ouvrez un nouveau terminal** après l’installation.

### Build

```powershell
cd C:\GitDev\Projet\orchestrateur
cargo build -p orchestrateur-cli --bin orch
cargo build --release -p orchestrateur-cli --bin orch --features gateway,websocket-server
```

### Lancement (exemples)

```powershell
cd C:\GitDev\Projet\orchestrateur
$env:ORCHESTRATEUR_DAEMON_TOKEN = "secret"

orch daemon run --workspace C:\GitDev\Projet\orchestrateur\workspace
orch list --workspace workspace
orch watcher status --workspace workspace

# Sans PATH :
.\target\release\orch.exe daemon run --workspace workspace
```

> **Erreur « n'est pas reconnu »** : vous êtes probablement dans `C:\Windows\System32` sans Rust/just dans le PATH. `cd` vers le dépôt, installez Rust si besoin, puis `.\install.ps1 -Dev`.

---

## 7. Mode local sans IA (dégradé)

L’application **démarre toujours** même si xAI/Ollama sont absents ou hors ligne.

| Composant | Comportement si indisponible |
|-----------|------------------------------|
| Liste / détail / filtre mémoires | ✅ Fonctionne (filesystem + LanceDB) |
| Recherche sémantique | ⚠ Désactivée |
| Assimilation LLM | ⚠ Désactivée |

---

## 8. Tests (Phase 7)

```powershell
cargo test -p orchestrator --test phase7_agent_loop
cargo test -p orchestrator --test integration_multi_agents_b212
cargo test -p orchestrator --test phase6_skills
cargo test -p cortex --test property
cargo test -p orchestrator --features gateway
cargo test -p orchestrator --features websocket-server
```

| Suite | Emplacement |
|-------|-------------|
| Intégration multi-agents + B212 | `crates/orchestrator/tests/integration_multi_agents_b212.rs` |
| Charge (10k mémoires, 100 agents) | `crates/orchestrator/tests/load_workspace_scale.rs` — voir [`tests/load/README.md`](tests/load/README.md) |

Les tests **sécurité** et **charge** sont marqués `#[ignore]` — `cargo test -- --ignored` pour la suite complète (job CI `test-heavy`).

---

## 9. Documentation

| Document | Contenu |
|----------|---------|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Vue d'ensemble système (Phase 7) |
| [`docs/DEVELOPER_GUIDE.md`](docs/DEVELOPER_GUIDE.md) | Contribuer, skills, agents, tests |
| [`docs/USER_GUIDE.md`](docs/USER_GUIDE.md) | CLI, dashboard, usage quotidien |
| [`docs/B212_INTEGRATION.md`](docs/B212_INTEGRATION.md) | Framework trading desk |
| [`docs/architecture.md`](docs/architecture.md) | Historique phases 21–27 |
| [`docs/protocol-ws.md`](docs/protocol-ws.md) | Protocole 1.2.0, handshake, événements drafts |
| [`docs/agent-tools.md`](docs/agent-tools.md) | Outils agent et profils de capacités |
| [`docs/UI_EVENT_HORIZON.md`](docs/UI_EVENT_HORIZON.md) | Design System Event Horizon (UI + Godot) |
| [`apps/tauri-desktop/README.md`](apps/tauri-desktop/README.md) | Desktop Tauri + Svelte |
| [`territoire-graphique/README.md`](territoire-graphique/README.md) | Client Godot |

**Ops :** `orch backup create` · `scripts/release.sh` · `scripts/profile.sh`

---

*Orchestrateur — Cortex first, séparation stricte des Peaux, contrôle total.*