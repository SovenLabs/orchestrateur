# ARCHIVE — Phase 10 v0.10.0

**Date :** Juin 2026  
**Version Cargo :** 0.10.0  
**Tag suggéré :** `phase10-v0.10.0`

---

## Objectif Phase 10

18 canaux messaging (≥ 15 requis), 6 toolsets agent groupés, auto-assimilation systématique par tour, bridge `ChatReply` enrichi, configuration `[agent]` + `[gateway.channels.*]`.

---

## Livrables

### Catalogue canaux (`orchestrator/src/gateway/channels/`)

| Module | Rôle |
|--------|------|
| `catalog.rs` | `ChannelCatalog` — 18 descripteurs statiques |
| `stub.rs` | Canaux stub inbound (Phase 11+ pour implémentations réelles) |
| `mod.rs` | Enregistrement via `CHANNEL_DESCRIPTORS` dans `build_gateway_stack` |

**Canaux enregistrés (18) :** webchat, webhook, telegram, discord, slack, whatsapp, signal, matrix, teams, email, irc, google_chat, line, mattermost, rocketchat, bluesky, nostr, twitch

**Implémentations dédiées :** webchat, webhook, telegram, discord, slack  
**Polling actif :** telegram uniquement  
**Stubs :** whatsapp, signal, matrix, teams, email, irc, google_chat, line, mattermost, rocketchat, bluesky, nostr, twitch

### Résolution configuration canaux

| Fonction | Rôle |
|----------|------|
| `resolve_channel_config()` | `gateway/mod.rs` — profils legacy + `extra_channels` HashMap |
| `GatewayConfig::extra_channels` | Surcharges `[gateway.channels.<id>]` depuis TOML |

### Toolsets (`orchestrator/src/tools/toolsets.rs`)

| ID | Outils |
|----|--------|
| `memory` | memory_search, memory_get, memory_assimilate |
| `mcp` | mcp_list_tools, mcp_call |
| `agent` | memory + mcp (défaut) |
| `research` | memory_search, memory_get |
| `ingest` | memory_assimilate, memory_search |
| `full` | tous les outils enregistrés |

`ToolRegistry::build_for_deps()` filtre selon `active_toolset` + présence MCP.

### Agent — auto-assimilation systématique

| Élément | Chemin |
|---------|--------|
| `AgentSettingsConfig` | `orchestrator/src/config.rs` — `[agent]` TOML |
| `auto_assimilate_turn: true` | Défaut — chaque tour appelle `memory_assimilate` |
| `AgentConfig::from_settings()` | `orchestrator/src/agent/config.rs` |
| `AgentTurnResult.tools_invoked` | `orchestrator/src/agent/loop_impl.rs` |

### Bridge enrichi

| Champ `ChatReply` | Rôle |
|-------------------|------|
| `tools_invoked` | Noms des outils exécutés pendant le tour |
| `auto_assimilated` | Résumé d'assimilation auto (si activée) |

### CLI

```powershell
orchestrateur channels list
orchestrateur toolsets list
```

### Configuration TOML

```toml
[agent]
active_toolset = "agent"
auto_assimilate_turn = true
max_tool_iterations = 3
graph_context_enabled = true

[gateway.channels.whatsapp]
enabled = false
token_env = "WHATSAPP_TOKEN"

[gateway.channels.matrix]
enabled = false
token_env = "MATRIX_ACCESS_TOKEN"
```

---

## Vérification

```powershell
cargo test -p orchestrator --features gateway gateway_channels
cargo test -p orchestrator --features gateway gateway_integration
cargo test -p orchestrator config::tests::agent_defaults_auto_assimilate_turn_true
cargo test -p orchestrator toolsets
cargo clippy -p orchestrator --features gateway -- -D warnings
```

---

## Hors scope (Phase 11+)

- Plugins dynamiques
- Skills hub
- Inbound réel pour canaux stub (WhatsApp, Matrix, …)
- Livraison sortante étendue au-delà de telegram/discord/slack

---

*Orchestrateur — Sovën, 2026*