# Protocole WebSocket — Orchestrateur v2 (Phase 23)

**Version protocole :** `1.2.0` · **Transport :** JSON texte · **Endpoint :** `ws://127.0.0.1:28790/ws`

## Versioning

| Constante Rust | Valeur | Rôle |
|----------------|--------|------|
| `PROTOCOL_VERSION` | `1.2.0` | Version négociée dans `connect_ok` |
| `PROTOCOL_MIN_CLIENT` | `1.0.0` | Minimum supporté (warning si inférieur) |

### Règles de compatibilité

- **MAJOR** : rupture wire format (renommage `type`, suppression champ obligatoire)
- **MINOR** : extensions rétrocompatibles (`protocol_version` dans `connect`, `/metrics`, nouveaux events)
- **PATCH** : documentation uniquement

Clients `1.0.x` restent supportés : `protocol_version` dans `connect` est optionnel (défaut `1.0.0`).

## Handshake (v1.1)

```json
{
  "type": "connect",
  "token": "dev",
  "protocol_version": "1.2.0",
  "client": {
    "window_kind": "desktop",
    "window_id": "tauri-main",
    "panels": ["dashboard", "memory"],
    "subscriptions": ["brain_pulse", "memories"]
  }
}
```

Réponse :

```json
{
  "type": "connect_ok",
  "version": "0.23.0",
  "protocol_version": "1.2.0",
  "session_id": "uuid",
  "territory_session_id": "uuid"
}
```

## Événements UI typés (`BackendEvent`)

| Broadcast daemon | `BackendEvent` |
|------------------|----------------|
| `brain_pulse` | `agent_activity` |
| `memory_assimilated` | `memory_assimilated` |
| `thought_propagation` | `thought_propagation` |
| `neuron_stimulated` | `neuron_stimulated` |
| `degraded_mode` | `system_status: degraded` |

Source de vérité Rust : `crates/shared-types/src/events.rs`.

## Observabilité

### `GET /health`

```json
{
  "status": "ok",
  "version": "0.23.0",
  "protocol_version": "1.2.0",
  "port": 28790,
  "territory_session_id": "uuid",
  "connected_clients": 2,
  "connected_windows": {
    "main": 0,
    "extension": 0,
    "desktop": 1,
    "sphere": 1,
    "total": 2
  },
  "metrics": {
    "messages_received": 120,
    "messages_sent": 118,
    "broadcasts_sent": 15,
    "execute_requests": 42,
    "ping_requests": 30,
    "connections_opened": 5,
    "auth_failures": 0,
    "parse_errors": 1
  }
}
```

### `GET /metrics`

Réponse allégée : `protocol_version`, `connected_clients`, `metrics`.

## Résilience client (Tauri)

| Mécanisme | Implémentation |
|-----------|----------------|
| Reconnexion | Backoff exponentiel 500ms → 30s |
| File d'attente | `OutboundMessageQueue` (max 64) |
| Requêtes corrélées | `PendingRequestRegistry` + `executeAsync` |
| Heartbeat | `ping`/`pong` toutes les 15s |
| Idempotence | Broadcasts = affichage uniquement |

## Tests

```bash
# Rust — protocole + live WS
cargo test -p orchestrator --features websocket-server

# TypeScript — queue, backoff, pending
cd apps/tauri-desktop && npm test
```

Voir aussi [`territoire-graphique/communication.md`](../territoire-graphique/communication.md) pour le contrat bridge complet.