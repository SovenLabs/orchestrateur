# Rapport complet — Phases 7 à 14 (v0.7.0 → v0.14.0)

**Projet :** Orchestrateur (SovënLabs)  
**Période :** Juin 2026  
**Dépôt :** https://github.com/SovenLabs/orchestrateur  
**Tag courant :** `phase14-v0.14.0`

### Mantra d'identité (post Phase 14)

> **Cortex first, agent second, gateway third.**

| Renommage | Ancien | Nouveau |
|-----------|--------|---------|
| Profils outils | `toolsets` / `active_toolset` | `capability-profiles` / `active_capability_profile` |
| Port gateway | `18789` | `28789` (défaut) |

Les **différenciateurs** du produit : auto-assimilation, graphe, honeypots, recherche proactive — pas les 18 canaux messaging.

---

## Vue d'ensemble

| Phase | Version | Tag Git | Thème principal |
|-------|---------|---------|-----------------|
| 7 | 0.7.0 | `phase7-v0.7.0` | Agent loop + outils mémoire + sessions |
| 8 | 0.8.0 | `phase8-v0.8.0` | Gateway WebSocket + canaux P0 |
| 9 | 0.9.0 | `phase9-v0.9.0` | Provider registry + MCP stdio |
| 10 | 0.10.0 | `phase10-v0.10.0` | 18 canaux gateway + profils de capacités + auto-assimilation |
| 11 | 0.11.0 | `phase11-v0.11.0` | Skills hub filesystem + plugins subprocess |
| 12 | 0.12.0 | `phase12-v0.12.0` | Plugins natifs + inbound stub HTTP |
| 13 | 0.13.0 | `phase13-v0.13.0` | Marketplace + intégrité BLAKE3 + skill_suggest |
| 14 | 0.14.0 | `phase14-v0.14.0` | Polling stub + catalogue signé + auto-exécute |

**Archives détaillées :** `docs/ARCHIVE_PHASE{N}_v0.{N}.0.md` pour chaque phase.

---

## Phase 7 — v0.7.0 — Fondations agentiques

### Objectif
Poser la boucle agent (LLM ↔ outils) avec persistance de sessions, sans gateway.

### Livrables majeurs

**Cortex**
- `SessionKey`, `ConversationTurn`, `Session` — domaine sessions
- Port `SessionRepository`

**Orchestrator**
- Module `agent/` : `AgentLoop`, `AgentConfig`, parsing JSON outils
- Module `tools/` : `ToolRegistry`, `memory_search`, `memory_get`, `memory_assimilate`
- `OrchestratorFacade::agent_turn()` — point d'entrée agent
- `AppDependencies.session_repo` — injection sessions

**Infrastructure**
- `SqliteSessionStore` — `workspace/.orchestrateur/sessions.db`

**Bridge / Peaux**
- `Command::Chat` → `agent_turn()` (session `default`) au lieu de chat LLM simple

### Fichiers clés
- `crates/cortex/src/domain/session.rs`
- `crates/cortex/src/ports/session_repository.rs`
- `crates/orchestrator/src/agent/`
- `crates/orchestrator/src/tools/`
- `crates/infrastructure/src/session_store.rs`

---

## Phase 8 — v0.8.0 — Gateway WebSocket

### Objectif
Gateway temps réel (port 28789 depuis identité v0.14+), canaux messaging P0, streaming agent, webhook HTTP.

### Livrables majeurs

**Gateway** (`orchestrator/src/gateway/`)
- `protocol.rs` — WS typé (`connect`, `agent.send`, `agent.stream.*`)
- `server.rs` — Axum `/ws`, `/health`, `/v1/channels/webhook`
- `runtime.rs` — `GatewayRunner` → `AgentLoop` + audit `gateway_inbound`
- `registry.rs` — `ChannelRegistry`, trait `Channel`
- `delivery.rs` — livraison sortante multiplexée
- `channels/` — webchat, webhook, telegram (polling), discord/slack (stubs)

**Agent streaming**
- `AgentStreamEvent`, `AgentStreamSink`
- `AgentLoop::run_turn_with_stream()`
- `OrchestratorFacade::agent_turn_with_stream()`

**Configuration**
- `[gateway]` + sous-sections canaux dans `orchestrator.toml`

**CLI**
```powershell
orchestrateur gateway run
orchestrateur gateway run --port 28789 --bind 127.0.0.1
```

**Protocole WS (résumé)**
1. `connect` + token → `connect_ok`
2. `agent_send` → `agent_stream_delta` / `agent_stream_tool` / `agent_stream_end`

**Webhook**
```http
POST /v1/channels/webhook
X-Orchestrateur-Webhook-Secret: <secret>
{"message":"Hello","session_key":"webhook"}
```

---

## Phase 9 — v0.9.0 — Providers + MCP

### Objectif
Registre typé 12 LLM + 5 embeddings, profils TOML, client MCP stdio, outils `mcp_*`.

### Livrables majeurs

**Provider registry** (`orchestrator/src/providers/`)
- Catalogue statique : xai, ollama, openai, anthropic, groq, openrouter, together, deepseek, mistral, perplexity, lmstudio, azure_openai
- Embeddings : ollama, openai, voyage, huggingface, fastembed
- `ProviderProfiles` — surcharges `[provider_profiles.<id>]`

**Infrastructure**
- `OpenAiCompatibleLlmProvider`, `AnthropicLlmProvider`
- `OpenAiEmbeddingsProvider`
- Résolution via `providers/resolve.rs`

**MCP** (`crates/mcp/`)
- `McpStdioClient` — JSON-RPC 2.0 stdio
- `McpManager` — implémente `McpGateway`
- Wiring dans `infrastructure/wiring.rs`

**Outils agent**
- `mcp_list_tools`, `mcp_call`

**CLI**
```powershell
orchestrateur providers list
orchestrateur providers list --kind llm
orchestrateur providers list --kind embedding
```

---

## Phase 10 — v0.10.0 — Canaux gateway + profils de capacités

### Objectif
18 canaux messaging (gateway optionnel), 7 profils de capacités, auto-assimilation systématique par tour.

### Livrables majeurs

**Catalogue canaux (18)**
webchat, webhook, telegram, discord, slack, whatsapp, signal, matrix, teams, email, irc, google_chat, line, mattermost, rocketchat, bluesky, nostr, twitch

- **Dédiés :** webchat, webhook, telegram, discord, slack
- **Polling actif (Phase 10) :** telegram uniquement
- **Stubs :** les 13 autres

**Profils de capacités** (`tools/capability_profiles.rs`)

| ID | Outils |
|----|--------|
| `memory` | memory_search, memory_get, memory_assimilate |
| `mcp` | mcp_list_tools, mcp_call |
| `agent` | memory + mcp + skills (défaut) |
| `research` | memory_search, memory_get |
| `ingest` | memory_assimilate, memory_search |
| `full` | tous les outils |

**Auto-assimilation**
- `[agent] auto_assimilate_turn = true` (défaut)
- Chaque tour appelle `memory_assimilate` en fin de boucle
- `AgentTurnResult.tools_invoked`, `auto_assimilated`

**Bridge enrichi**
- `ChatReply.tools_invoked`, `ChatReply.auto_assimilated`

**CLI**
```powershell
orchestrateur channels list
orchestrateur capability-profiles list
```

---

## Phase 11 — v0.11.0 — Skills hub dynamiques

### Objectif
Hub filesystem, plugins subprocess, chargement auto, CLI skills-hub.

### Livrables majeurs

**Skills hub** (`orchestrator/src/skills/`)
- `hub.rs` — scan `workspace/skills/*/skill.toml` + entrées inline TOML
- `manifest.rs` — parse `skill.toml` (`kind = subprocess`)
- `plugin.rs` — `SubprocessPluginSkill` async + timeout
- `registry.rs` — `with_operational_skills_and_hub()`, `reload_hub()`

**Format manifeste**
```toml
[skill]
id = "pong"
description = "Plugin démo"
version = "0.1.0"
enabled = true

[subprocess]
command = "cmd"
args = ["/c", "echo", "pong"]
stdin_json = false
timeout_secs = 5
```

**Bridge**
- `SkillSummary.source` : `builtin` | `hub`
- `SkillSummary.version` optionnel

**CLI**
```powershell
orchestrateur skills-hub list
orchestrateur skills-hub path
orchestrateur skill list
orchestrateur skill run pong
```

**Démo :** `workspace/skills/pong/skill.toml`

---

## Phase 12 — v0.12.0 — Plugins natifs + inbound stub

### Objectif
Bibliothèques `.dll`/`.so`, inbound HTTP générique pour stubs, outils agent `skill_*`.

### Livrables majeurs

**Plugins natifs** (`feature plugins-native`)
- `skills/native.rs` — ABI FFI + libloading
- `plugins/pong-native/` — crate `cdylib` démo
- ABI : `orchestrateur_skill_execute`, `orchestrateur_skill_free`

**Inbound stub HTTP**
```http
POST /v1/channels/{channel_id}/inbound
X-Orchestrateur-Channel-Token: <token>
{"message":"Hello","session_key":"whatsapp:1"}
```

**Outils agent agentic**
- `skill_list`, `skill_execute`
- Profil `skills` (7 profils) inclus dans `agent`
- `[agent] skill_tools_enabled = true`

**Vérification**
```powershell
cargo build --release -p orchestrateur-plugin-pong-native
cargo test -p orchestrator --features gateway,plugins-native
```

---

## Phase 13 — v0.13.0 — Marketplace + intégrité

### Objectif
Catalogue skills local/distant, BLAKE3 manifestes, routage agent assisté.

### Livrables majeurs

**Marketplace** (`skills/marketplace.rs`)
- `MarketplaceCatalog`, `SkillsMarketplace::sync_to_hub()`
- `SkillsMarketplace::verify_hub_integrity()`
- Feature `skills-marketplace` — fetch distant `reqwest`
- Catalogue : `workspace/skills/marketplace/catalog.json`

**Intégrité manifestes**
- Champ `integrity_hash` BLAKE3 dans `[skill]`
- `compute_integrity_hash()` / `verify_integrity_hash()`
- Vérification au `load_manifest()`

**Agent assisté**
- Outil `skill_suggest`
- `skill_auto_suggest` — catalogue + top 3 dans le prompt
- `suggest_skills()` — scoring textuel partagé

**CLI**
```powershell
orchestrateur skills-hub marketplace
orchestrateur skills-hub sync
orchestrateur skills-hub verify
```

**Config**
```toml
[skills_hub]
marketplace_enabled = true
marketplace_catalog = "skills/marketplace/catalog.json"
# marketplace_url = "https://example.com/catalog.json"

[agent]
skill_auto_suggest = true
```

---

## Phase 14 — v0.14.0 — Polling + signature + auto-exécute

### Objectif
Polling HTTP stub, catalogue signé BLAKE3, exécution skill proactive, bridge HUD marketplace.

### Livrables majeurs

**Polling HTTP canaux stub**
- `GatewayChannelConfig.poll_url`, `poll_interval_secs`
- Boucle GET périodique dans `stub.rs`
- Parse : message unique, tableau, ou `{messages:[]}`
- Header `X-Orchestrateur-Channel-Token` optionnel

**Catalogue signé**
- Champ `catalog_hash` BLAKE3 sur `MarketplaceCatalog`
- `compute_catalog_hash()`, `verify_catalog_hash()`
- `marketplace_require_signature` (défaut `false`)

**Agent `skill_auto_execute`**
- `skill_auto_execute` (défaut `false`)
- `skill_auto_execute_threshold` (défaut `10`)
- `best_skill_match()` — exécution avant LLM si score ≥ seuil
- `AgentTurnResult.auto_executed_skills`

**Bridge HUD**
| Commande | Réponse |
|----------|---------|
| `SkillsMarketplaceList` | `MarketplaceList` |
| `SkillsHubVerify` | `HubIntegrityReport` |

`ChatReply.auto_executed_skills` exposé au CLI/HUD/TUI.

**Config exemple**
```toml
[agent]
skill_auto_execute = false
skill_auto_execute_threshold = 10

[gateway.channels.whatsapp]
poll_url = "http://127.0.0.1:8080/whatsapp/poll"
poll_interval_secs = 30

[skills_hub]
marketplace_require_signature = false
```

---

## Architecture actuelle (v0.14.0)

```
┌─────────────────────────────────────────────────────────────┐
│  Peaux : HUD (egui) · TUI (ratatui) · CLI · Gateway WS      │
└──────────────────────────┬──────────────────────────────────┘
                           │ Bridge Command/Response
┌──────────────────────────▼──────────────────────────────────┐
│  OrchestratorFacade                                         │
│  ├── AgentLoop (LLM + tools + skills auto-exécute/suggest)  │
│  ├── SkillRegistry (builtin + hub + native)                 │
│  ├── GatewayRunner (18 canaux, polling stub/telegram)       │
│  └── Use cases Cortex (assimilate, search, list, …)         │
└──────────────────────────┬──────────────────────────────────┘
                           │ Ports
┌──────────────────────────▼──────────────────────────────────┐
│  Cortex (domaine) · Infrastructure (LanceDB, LLM, MCP)      │
└─────────────────────────────────────────────────────────────┘
```

| Zone | Chemin |
|------|--------|
| Agent | `crates/orchestrator/src/agent/` |
| Tools / capability_profiles | `crates/orchestrator/src/tools/` |
| Gateway | `crates/orchestrator/src/gateway/` |
| Skills | `crates/orchestrator/src/skills/` |
| Providers | `crates/orchestrator/src/providers/` |
| MCP | `crates/mcp/` |
| Config | `workspace/config/orchestrator.toml.example` |
| Skills workspace | `workspace/skills/` |

---

## Features Cargo (orchestrator)

| Feature | Rôle |
|---------|------|
| `gateway` | Axum WS + canaux + reqwest polling |
| `plugins-native` | libloading `.dll`/`.so` |
| `skills-marketplace` | Fetch catalogue distant |
| `cli` / `tui` / `http` | Peaux respectives |

---

## Commandes CLI utiles (cumul phases 7–14)

```powershell
# Agent / mémoire
orchestrateur chat "Bonjour"
orchestrateur search "Rust"
orchestrateur assimilate "Texte à mémoriser"

# Gateway
orchestrateur gateway run
orchestrateur channels list

# Providers & profils de capacités
orchestrateur providers list
orchestrateur capability-profiles list

# Skills
orchestrateur skill list
orchestrateur skill run pong
orchestrateur skills-hub list
orchestrateur skills-hub marketplace
orchestrateur skills-hub sync
orchestrateur skills-hub verify
```

---

## Vérification complète (machine avec Rust)

```powershell
cargo test -p cortex
cargo test -p orchestrator
cargo test -p orchestrator --features gateway
cargo test -p orchestrator --features gateway,plugins-native,skills-marketplace
cargo test -p infrastructure
cargo test -p mcp
cargo clippy -p orchestrator --features gateway,plugins-native,skills-marketplace -- -D warnings
```

---

## Variables d'environnement (référence)

| Variable | Phase | Usage |
|----------|-------|-------|
| `XAI_API_KEY` | 3+ | LLM principal |
| `ORCHESTRATEUR_GATEWAY_TOKEN` | 8+ | Auth WebSocket |
| `ORCHESTRATEUR_WEBHOOK_SECRET` | 8+ | Webhook HTTP |
| `TELEGRAM_BOT_TOKEN` | 8+ | Polling + livraison Telegram |
| `DISCORD_BOT_TOKEN` / `DISCORD_WEBHOOK_URL` | 8+ | Discord |
| `SLACK_BOT_TOKEN` / `SLACK_CHANNEL_ID` | 8+ | Slack |
| `WHATSAPP_TOKEN`, `MATRIX_ACCESS_TOKEN`, … | 10+ | Canaux stub |
| `OPENAI_API_KEY`, `GROQ_API_KEY`, … | 9+ | Provider profiles |

---

## Hors scope cumulé (Phase 15+)

- SDK WhatsApp / Matrix natifs (hors polling HTTP générique)
- Signature GPG / notarisation tierce des plugins
- UI HUD marketplace graphique dédiée
- Polling WebSocket / SSE
- Pairing DM avancé + routing multi-tenant

---

## Tags Git recommandés

```text
phase7-v0.7.0
phase8-v0.8.0
phase9-v0.9.0
phase10-v0.10.0
phase11-v0.11.0
phase12-v0.12.0
phase13-v0.13.0
phase14-v0.14.0   ← release courante
```

---

*Rapport généré — Orchestrateur SovënLabs, Juin 2026*