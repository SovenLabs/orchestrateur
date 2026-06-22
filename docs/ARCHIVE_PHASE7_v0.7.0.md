# ARCHIVE — Phase 7 v0.7.0

**Date :** Juin 2026  
**Version Cargo :** 0.7.0  
**Tag suggéré :** `phase7-v0.7.0`

---

## Objectif Phase 7

Poser les **fondations agentiques** : sessions, boucle agent, outils mémoire Cortex, contexte graphe — sans gateway (Phase 8).

---

## Livrables

### Cortex — sessions

| Élément | Chemin |
|---------|--------|
| `SessionKey`, `ConversationTurn`, `Session` | `crates/cortex/src/domain/session.rs` |
| Port `SessionRepository` | `crates/cortex/src/ports/session_repository.rs` |

### Orchestrator — agent + tools

| Module | Rôle |
|--------|------|
| `orchestrator/src/agent/` | `AgentLoop`, `AgentConfig`, parsing tool JSON |
| `orchestrator/src/tools/` | `ToolRegistry` + `memory_search`, `memory_get`, `memory_assimilate` |
| `OrchestratorFacade::agent_turn()` | Point d'entrée agent |
| `AppDependencies.session_repo` | Injection sessions |

### Infrastructure

| Adapter | Chemin |
|---------|--------|
| `SqliteSessionStore` | `crates/infrastructure/src/session_store.rs` |
| DB | `workspace/.orchestrateur/sessions.db` |

### Bridge / Peaux

- `Command::Chat` → `agent_turn()` (session `default`) au lieu de `chat()` simple

---

## Vérification

```powershell
cargo test -p cortex
cargo test -p orchestrator
cargo test -p infrastructure
cargo clippy -p orchestrator -- -D warnings
```

---

## Hors scope (Phase 8)

- Gateway WebSocket + canaux Telegram/Discord
- Streaming agent events
- Plugins dynamiques

---

*Orchestrateur — Sovën, 2026*