# Orchestrateur — Second cerveau local souverain

**Version Cargo workspace : 0.28.0** · **Protocole WS : 1.2.0** · **Rust 1.80+** · **Juin 2026**

> Architecture : [`docs/architecture.md`](docs/architecture.md) · Protocole : [`docs/protocol-ws.md`](docs/protocol-ws.md)

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

**Skills** : extensions optionnelles de l’Esprit (subprocess, natif, marketplace local) — jamais le substitut du Cortex.

---

## 2. v1.2 — Kinds, drafts, watcher, prétraitement

| Fonctionnalité | Résumé |
|----------------|--------|
| **Kinds** | Chaque mémoire porte un type sémantique (`decision`, `dead_end`, `pattern`, `context`, `progress`, `business`) — frontmatter Markdown + couleurs UI |
| **Drafts** | File de revue : brouillons générés (watcher ou agent) → publier / rejeter via daemon WS (`draft_created`, `draft_published`, `draft_discarded`) |
| **Watcher** | `[watcher] enabled = true` — surveille session/git, produit des drafts LLM ; CLI `watcher status` / `draft publish` |
| **Message preprocess** | `[agent] message_preprocess = true` — messages courts/vagues enrichis via graphe + recherche proactive avant le LLM ; événements `MessageExpanded` / `MessageCompressed` |

Desktop Tauri **1.2.0** : onglet Drafts, badges kind cosmiques, contrôle watcher dans le drawer `]`.

---

## 3. Architecture du dépôt

```
Orchestre/
├── crates/
│   ├── shared-types/     # Types partagés + export TypeScript
│   ├── orchestrator-core/# Placeholder extraction Cortex/AgentLoop
│   ├── cortex/           # Domaine + ports (MemoryRepository, VectorStore, EmbeddingProvider)
│   ├── orchestrator/     # Application (facade, bridge, daemon WS, gateway, watcher, drafts)
│   ├── infrastructure/   # Adapters (LanceDB, Ollama, xAI, filesystem)
│   ├── cli/              # Binaire orchestrateur.exe (CLI + daemon)
│   └── client/           # Client bridge embarqué
├── apps/
│   └── tauri-desktop/    # Desktop Tauri 2 + Svelte 5
├── territoire-graphique/ # Client Godot 4
├── workspace/            # Données utilisateur (hors code source)
│   ├── config/
│   ├── memories/
│   ├── logs/
│   └── .orchestrateur/   # LanceDB (généré au runtime)
├── docs/
│   ├── architecture.md
│   └── protocol-ws.md
└── Cargo.toml            # Workspace Rust
```

| Crate | Responsabilité |
|-------|----------------|
| `cortex` | Entités, value objects, ports, services purs — zéro dépendance infra |
| `orchestrator` | Use cases, facade, bridge, daemon WS (`websocket-server`) |
| `infrastructure` | Implémentations concrètes des ports |
| `apps/tauri-desktop` | Interface desktop — client WS du daemon |

Voir [`docs/architecture.md`](docs/architecture.md) et [`territoire-graphique/communication.md`](territoire-graphique/communication.md).

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

## 5. Installation utilisateur (one-liner)

Les binaires Windows sont publiés sur **[GitHub Releases](https://github.com/SovenLabs/orchestrateur/releases)**.

```powershell
irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1 | iex
```

Version fixe : `$env:ORCHESTRATEUR_VERSION = "0.28.0"`

---

## 6. Compilation et binaires

### Prérequis

- Rust stable **1.80+** (`rust-toolchain.toml`)
- `protoc` : fourni via `.tools/` (voir `.cargo/config.toml`)

### Build

```powershell
cargo build -p orchestrateur-cli
cargo build --release -p orchestrateur-cli --features gateway,websocket-server
```

### Lancement (exemples)

```powershell
$env:ORCHESTRATEUR_DAEMON_TOKEN = "secret"
.\target\release\orchestrateur.exe daemon run --workspace workspace
.\target\release\orchestrateur.exe list --workspace workspace
.\target\release\orchestrateur.exe watcher status --workspace workspace
```

---

## 7. Mode local sans IA (dégradé)

L’application **démarre toujours** même si xAI/Ollama sont absents ou hors ligne.

| Composant | Comportement si indisponible |
|-----------|------------------------------|
| Liste / détail / filtre mémoires | ✅ Fonctionne (filesystem + LanceDB) |
| Recherche sémantique | ⚠ Désactivée |
| Assimilation LLM | ⚠ Désactivée |

---

## 8. Tests

```powershell
cargo test -p cortex
cargo test -p orchestrator --lib
cargo test -p orchestrator --features gateway
cargo test -p orchestrator --features websocket-server
```

Les tests **sécurité** et **charge** sont marqués `#[ignore]` — `cargo test -- --ignored` pour la suite complète.

---

## 9. Documentation

| Document | Contenu |
|----------|---------|
| [`docs/architecture.md`](docs/architecture.md) | Architecture v2, fenêtres WS, daemon |
| [`docs/protocol-ws.md`](docs/protocol-ws.md) | Protocole 1.2.0, handshake, événements drafts |
| [`apps/tauri-desktop/README.md`](apps/tauri-desktop/README.md) | Desktop Tauri + Svelte |
| [`territoire-graphique/README.md`](territoire-graphique/README.md) | Client Godot |

---

*Orchestrateur — Cortex first, séparation stricte des Peaux, contrôle total.*