# ARCHIVE — Phase 13 v0.13.0

**Date :** Juin 2026  
**Version Cargo :** 0.13.0  
**Tag suggéré :** `phase13-v0.13.0`

---

## Objectif Phase 13

Marketplace skills (catalogue local + sync distant), intégrité BLAKE3 des manifestes, routage agent assisté (`skill_suggest` + prompt enrichi).

---

## Livrables

### Marketplace (`orchestrator/src/skills/marketplace.rs`)

| Composant | Rôle |
|-----------|------|
| `MarketplaceCatalog` | Parse `catalog.json` |
| `SkillsMarketplace::sync_to_hub()` | Écrit `workspace/skills/<id>/skill.toml` |
| `SkillsMarketplace::verify_hub_integrity()` | Vérifie les hash BLAKE3 |
| `skills-marketplace` feature | Fetch catalogue distant via `reqwest` |

**Catalogue** : `workspace/skills/marketplace/catalog.json`

```json
{
  "version": 1,
  "skills": [
    {
      "id": "market-echo",
      "name": "Market Echo",
      "description": "...",
      "version": "0.1.0",
      "enabled": true,
      "manifest_toml": "[skill]\n..."
    }
  ]
}
```

### Intégrité manifestes

- Champ optionnel `integrity_hash` dans `[skill]` (BLAKE3 hex)
- `compute_integrity_hash()` / `verify_integrity_hash()` dans `manifest.rs`
- Vérification automatique au `load_manifest()`

### Agent agentic assisté

| Élément | Rôle |
|---------|------|
| `skill_suggest` | Outil — correspondance nom/description |
| `skill_auto_suggest` | Injecte catalogue + top 3 suggestions dans le prompt |
| `suggest_skills()` | Scoring textuel partagé CLI/tests |

### Configuration TOML

```toml
[skills_hub]
marketplace_enabled = true
marketplace_catalog = "skills/marketplace/catalog.json"
# marketplace_url = "https://example.com/catalog.json"

[agent]
skill_tools_enabled = true
skill_auto_suggest = true
```

### CLI

```powershell
orchestrateur skills-hub marketplace
orchestrateur skills-hub sync
orchestrateur skills-hub verify
```

---

## Vérification

```powershell
cargo test -p orchestrator skills::marketplace
cargo test -p orchestrator skills::manifest
cargo test -p orchestrator --features skills-marketplace
cargo clippy -p orchestrator --features skills-marketplace -- -D warnings
```

---

## Hors scope (Phase 14+)

- SDK WhatsApp / Matrix (polling réel)
- Marketplace signé GPG / notarisation tierce
- Auto-exécution skill sans tool call LLM
- UI HUD marketplace

---

*Orchestrateur — Sovën, 2026*