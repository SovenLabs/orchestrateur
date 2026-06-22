# ARCHIVE — Phase 8 v0.8.0

**Date :** Juin 2026  
**Version Cargo :** 0.8.0  
**Tag suggéré :** `phase8-v0.8.0`

---

## Objectif Phase 8

Gateway WebSocket (port 18789), canaux P0, streaming `agent.stream.*`, CLI `gateway run`, audit par message entrant.

---

## Livrables

### Gateway (`orchestrator/src/gateway/`)

| Module | Rôle |
|--------|------|
| `protocol.rs` | Messages WS typés (`connect`, `agent.send`, `agent.stream.*`) |
| `server.rs` | Axum — `/ws`, `/health`, `/v1/channels/webhook` |
| `runtime.rs` | `GatewayRunner` → `AgentLoop` + audit `gateway_inbound` |
| `registry.rs` | `ChannelRegistry`, trait `Channel` |
| `delivery.rs` | Livraison sortante multiplexée |
| `channels/` | webchat, webhook, telegram (polling), discord/slack (stubs + livraison) |

### Agent — streaming

| Élément | Chemin |
|---------|--------|
| `AgentStreamEvent`, `AgentStreamSink` | `orchestrator/src/agent/stream.rs` |
| `AgentLoop::run_turn_with_stream()` | `orchestrator/src/agent/loop_impl.rs` |
| `OrchestratorFacade::agent_turn_with_stream()` | `orchestrator/src/facade.rs` |

### Configuration

| Section TOML | Fichier |
|--------------|---------|
| `[gateway]` + `[gateway.*]` | `workspace/config/orchestrator.toml.example` |
| `GatewayConfig` | `orchestrator/src/config.rs` |

### CLI

```powershell
$env:ORCHESTRATEUR_GATEWAY_TOKEN = "secret"
orchestrateur gateway run
# ou
orchestrateur gateway run --port 18789 --bind 127.0.0.1
```

Feature : `gateway` (activée par défaut sur `orchestrateur-cli`).

### Protocole WebSocket (résumé)

1. Client → `{"type":"connect","token":"…","session_key":"default"}`
2. Serveur → `connect_ok`
3. Client → `{"type":"agent_send","request_id":"r1","session_key":"default","message":"Salut"}`
4. Serveur → `agent_stream_delta` / `agent_stream_tool` / `agent_stream_end`

### Webhook HTTP

```http
POST /v1/channels/webhook
X-Orchestrateur-Webhook-Secret: <ORCHESTRATEUR_WEBHOOK_SECRET>
Content-Type: application/json

{"message":"Hello","session_key":"webhook"}
```

---

## Vérification

```powershell
cargo test -p orchestrator --features gateway
cargo test -p orchestrator --features gateway gateway_integration
cargo test -p cortex -p infrastructure
cargo clippy -p orchestrator --features gateway -- -D warnings
```

Variables d'environnement utiles :

| Variable | Usage |
|----------|-------|
| `ORCHESTRATEUR_GATEWAY_TOKEN` | Auth WebSocket |
| `ORCHESTRATEUR_WEBHOOK_SECRET` | Auth webhook HTTP |
| `TELEGRAM_BOT_TOKEN` | Polling Telegram + livraison |
| `DISCORD_WEBHOOK_URL` | Livraison Discord (optionnel) |
| `SLACK_BOT_TOKEN` + `SLACK_CHANNEL_ID` | Livraison Slack (optionnel) |

---

## Hors scope (Phase 9+)

- Provider registry extensible
- Discord Gateway bot inbound complet
- Slack Events API inbound
- Pairing DM + routing avancé
- Plugins dynamiques

---

*Orchestrateur — Sovën, 2026*