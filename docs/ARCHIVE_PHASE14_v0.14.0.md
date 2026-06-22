# ARCHIVE — Phase 14 v0.14.0

**Date :** Juin 2026  
**Version Cargo :** 0.14.0  
**Tag suggéré :** `phase14-v0.14.0`

---

## Objectif Phase 14

Polling HTTP générique pour canaux stub, catalogue marketplace signé BLAKE3, auto-exécution skill agent (`skill_auto_execute`), commandes bridge HUD marketplace.

---

## Livrables

### Polling HTTP canaux stub (`gateway/channels/stub.rs`)

| Composant | Rôle |
|-----------|------|
| `poll_url` | URL GET périodique (TOML `[gateway.channels.*]`) |
| `poll_interval_secs` | Intervalle minimal 5 s (défaut 30) |
| `parse_poll_payloads()` | Parse tableau, `{messages:[]}` ou message unique |

Réponse poll attendue (même format que webhook) :

```json
{"message":"Bonjour","session_key":"whatsapp:user1","external_id":"user1"}
```

ou `{"messages":[...]}`.

Header optionnel : `X-Orchestrateur-Channel-Token` si `token_env` est défini.

### Catalogue marketplace signé (`skills/marketplace.rs`)

| Composant | Rôle |
|-----------|------|
| `catalog_hash` | BLAKE3 hex du JSON sans ce champ |
| `compute_catalog_hash()` | Génère l'empreinte |
| `verify_catalog_hash()` | Vérifie à l'import |
| `marketplace_require_signature` | Rejette les catalogues non signés |

```json
{
  "version": 1,
  "catalog_hash": "abc123…",
  "skills": [ … ]
}
```

Générer le hash (machine avec Rust) :

```powershell
cargo test -p orchestrator skills::marketplace::tests::catalog_hash_roundtrip -- --nocapture
# ou charger le catalogue et appeler MarketplaceCatalog::compute_catalog_hash()
```

### Agent `skill_auto_execute`

| Élément | Rôle |
|---------|------|
| `skill_auto_execute` | Active l'exécution proactive (défaut `false`) |
| `skill_auto_execute_threshold` | Score minimal (défaut `10`) |
| `best_skill_match()` | Scoring partagé avec `skill_suggest` |
| `AgentTurnResult.auto_executed_skills` | Trace des skills auto-exécutées |

### Bridge HUD (Phase 14)

| Commande | Réponse |
|----------|---------|
| `SkillsMarketplaceList` | `MarketplaceList { version, catalog_hash, entries }` |
| `SkillsHubVerify` | `HubIntegrityReport { report }` |

`ChatReply` inclut `auto_executed_skills`.

### Configuration TOML

```toml
[agent]
skill_auto_execute = false
skill_auto_execute_threshold = 10

[gateway.channels.whatsapp]
enabled = true
token_env = "WHATSAPP_TOKEN"
poll_url = "http://127.0.0.1:8080/whatsapp/poll"
poll_interval_secs = 30

[skills_hub]
marketplace_require_signature = false
```

---

## Vérification

```powershell
cargo test -p orchestrator --features gateway,plugins-native,skills-marketplace
cargo test -p orchestrator skills::marketplace
cargo test -p orchestrator gateway_stub_inbound --features gateway
cargo clippy -p orchestrator --features gateway,plugins-native,skills-marketplace -- -D warnings
```

---

## Hors scope (Phase 15+)

- SDK WhatsApp / Matrix natifs
- Signature GPG / notarisation tierce
- UI HUD marketplace dédiée (panneau graphique)
- Polling WebSocket / SSE

---

*Orchestrateur — Sovën, 2026*