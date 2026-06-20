# SPEC — Cerveau Interne Souverain v0.4.0

**Projet :** Orchestrateur  
**Version Cargo :** 0.4.0  
**Date :** Juin 2026  
**Statut :** Phase 6 — Bridge étendu, observabilité, import, daemon HTTP

---

## 1. Vision

Le **Cerveau Interne Souverain** est un second cerveau **local**, **durable** (7–10 ans) et **contrôlé** par l'utilisateur. Il repose sur trois couches non négociables :

| Couche | Crate | Rôle |
|--------|-------|------|
| **Squelette** | `cortex` | Mémoires Markdown, graphe, ports hexagonaux, recherche |
| **Esprit** | `orchestrator` | Use cases, facade, bridge, sécurité, skills |
| **Peau** | `hud` / `cli` | Interfaces optionnelles et remplaçables |

**Principe directeur :** la solidité du Cortex prime sur tout rendu visuel. La Peau ne touche jamais aux ports Cortex — uniquement au **bridge** `Command` / `Response`.

---

## 2. Objectifs v0.4.0

1. **Bridge v0.4** — contrat JSON typé (`serde` adjacently tagged) pour CLI, HUD, TUI et daemon HTTP.
2. **Observabilité** — graphe de connaissances et journal d'audit exposés via le bridge.
3. **Mode dégradé explicite** — recherche bloquée si embeddings indisponibles ; bannière HUD permanente.
4. **Import souverain** — récupération de mémoires Markdown externes sans doublon d'ID.
5. **Daemon HTTP local** — `POST /v1/execute` avec Bearer token (feature `http`).
6. **Validation adversariale** — `MemoryDraftValidator` reste dans `cortex` uniquement.

---

## 3. Bridge v0.4 — Contrat

### 3.1 Sérialisation

```rust
#[serde(tag = "command", content = "payload")]
pub enum Command { ... }

#[serde(tag = "response", content = "payload")]
pub enum Response { ... }
```

### 3.2 Commandes

| Commande | Payload | Description |
|----------|---------|-------------|
| `Assimilate` | `{ text, tags }` | Assimilation LLM |
| `Search` | `{ query, limit }` | Recherche vectorielle |
| `List` | `{ filter, offset, limit }` | Liste paginée |
| `GetMemory` | `{ id }` | Détail mémoire |
| `HealthCheck` | — | Sonde providers |
| `SubscribeToEvents` | — | Accusé abonnement événements |
| `Graph` | — | Statistiques graphe |
| `Audit` | `{ limit }` | Journal d'audit récent |

### 3.3 Réponses

| Réponse | Payload | Description |
|---------|---------|-------------|
| `MemoryList` | `{ items, total }` | Résumés paginés |
| `MemoryDetail` | `{ memory }` | Entité complète |
| `SearchResults` | `{ items }` | Hits vectoriels |
| `Assimilated` | `{ memory_id, title }` | Accusé assimilation (détail via événement) |
| `GraphSummary` | `{ node_count, edge_count, hubs }` | Vue graphe |
| `AuditLog` | `{ entries, chain_intact }` | Entrées audit + intégrité chaîne |
| `Health` | `{ status, version, llm_available, embedding_available }` | Santé service |
| `Error` | `AppError` | Erreur sérialisable |
| `Success` | `{ message }` | Accusé simple |
| `Event` | `DomainEvent` | Événement domaine |

### 3.4 Mode dégradé — Search

Si `embedding_available == false`, `Command::Search` retourne :

```json
{
  "response": "Error",
  "payload": {
    "kind": "degraded",
    "message": "recherche indisponible — provider embeddings hors ligne"
  }
}
```

---

## 4. Journal d'audit (Couche 4)

- Fichier append-only JSONL, chaînage **BLAKE3**.
- `AuditLog::read_recent(limit)` — entrées les plus récentes, ordre chronologique.
- `AuditLog::verify_chain()` — vérifie hash et `previous_hash` sur le fichier complet.
- Types : `orchestrator::security::AuditEvent`.

---

## 5. Import de mémoires

**Use case :** `ImportMemories::execute(source_dir)`

1. Scan récursif `*.md`
2. Parse via `cortex::parse_memory_markdown`
3. Skip si `MemoryId` déjà présent
4. `SaveMemory` pour les nouvelles entrées
5. Rapport : `{ imported, skipped, errors }`

**Facade :** `OrchestratorFacade::import_from_directory(path)`

**CLI :** `orchestrateur import --source PATH`

---

## 6. Daemon HTTP (feature `http`)

```powershell
$env:ORCHESTRATEUR_DAEMON_TOKEN = "secret"
orchestrateur serve --port 17489 --bind 127.0.0.1
```

| Endpoint | Méthode | Auth | Corps |
|----------|---------|------|-------|
| `/v1/execute` | POST | `Authorization: Bearer <token>` | `Command` JSON |

Réponse : `Response` JSON (200) ou 401 si token invalide.

Dépendances workspace : `axum`, `tower`, `tower-http`.

---

## 7. HUD v0.4

### 7.1 Vues (`crates/hud/src/views/`)

| Module | Rôle |
|--------|------|
| `health_banner.rs` | Bannière permanente mode dégradé |
| `graph_view.rs` | Nœuds, arêtes, hubs cliquables |
| `audit_view.rs` | Entrées audit + statut chaîne |

### 7.2 Navigation

`HudMainView` : **Explorateur** | **Graphe** | **Audit**

- Changement d'onglet → `Command::Graph` ou `Command::Audit { limit: 100 }`
- Clic hub → retour Explorateur + `GetMemory`

### 7.3 Bannière dégradée

Affichée tant que `!embedding_available` ou `!llm_available`.

---

## 8. CLI v0.4

- Version affichée : **0.4.0**
- Nouvelles sous-commandes : `import`, `serve` (feature `http`)
- `assimilate` gère `Response::Assimilated`
- `graph` via bridge `Command::Graph`

---

## 9. Sécurité — périmètre inchangé

- **Ne pas toucher** : tri-stack, Hermes, OpenClaw
- **Validator** : `cortex::MemoryDraftValidator` uniquement
- Audit, intégrité config, honeypots, garde comportemental : inchangés en comportement

---

## 10. Tests requis

```powershell
cargo test -p cortex -p orchestrator -p orchestrateur-cli -p orchestrateur-hud
cargo clippy -p orchestrator --features tui -p orchestrateur-cli -p orchestrateur-hud -- -D warnings
```

---

## 11. Versioning

| Axe | Valeur |
|-----|--------|
| Cargo workspace | **0.4.0** |
| Bridge | **v0.4** |
| Document | **SPEC_CERVEAU_v0.4.0** |

Bump **0.5.0** lors de la prochaine phase (Skills opérationnelles, packaging Windows, etc.).

---

*Orchestrateur — Cerveau Interne Souverain — Sovën, 2026*