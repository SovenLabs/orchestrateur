# Guide Développeur — Orchestrateur

**Version workspace :** 0.28.0 · **Rust :** 1.80+

## Prise en main

```powershell
git clone https://github.com/SovenLabs/orchestrateur.git
cd orchestrateur
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Dev
cargo test -p orchestrator --test phase7_agent_loop
```

> Le one-liner `irm … | iex` cible les **releases packagées** GitHub. Tant qu’aucune release n’est publiée, le mode `-Dev` depuis le clone est la voie officielle.

| Commande | Usage |
|----------|-------|
| `just daemon` | Daemon WS local `:28790` |
| `just desktop-dev` | Tauri + Godot |
| `orch doctor --workspace workspace` | Diagnostic |

## Structure du dépôt

```
crates/
  cortex/           # Domaine pur — pas de I/O
  infrastructure/   # Adapters (LanceDB, Ollama, cache embeddings)
  orchestrator/   # Application (facade, agent, skills, B212)
  cli/              # Binaire orch
apps/tauri-desktop/
territoire-graphique/
plugins/
```

**Règle d'or :** le domaine `cortex` ne dépend d'aucun adapter. Les use cases vivent dans `orchestrator`, les implémentations dans `infrastructure`.

## Ajouter un outil agent

1. Créer `crates/orchestrator/src/tools/mon_outil.rs` implémentant le trait `Tool`
2. Enregistrer dans `tools/registry.rs` ou `tools/extended/`
3. Ajouter au profil dans `tools/capability_profiles.rs` si besoin
4. Tester : `crates/orchestrator/tests/phase7_agent_loop.rs`

Voir [`agent-tools.md`](agent-tools.md).

## Ajouter une skill (Phase 6)

```powershell
orch skill create mon-skill --workspace workspace
```

Fichiers générés :

```
workspace/skills/mon-skill/
  skill.toml    # metadata, dependencies, skill_type
  SKILL.md      # documentation
  run.ps1       # ou plugin natif
```

Types : `cortex`, `agent`, `b212`, `communication`, `generic`.

| Composant | Fichier Rust |
|-----------|--------------|
| Trait | `orchestrator/src/skills/trait.rs` |
| Hub | `orchestrator/src/skills/hub.rs` |
| Loader | `orchestrator/src/skills/loader.rs` |
| Injection agent | `orchestrator/src/persistent/skill_injection.rs` |

Tests : `cargo test -p orchestrator --test phase6_skills`

## Ajouter un agent persistant

Via code : `AgentManager::create_agent(id, name, role, model)`.

Structure créée :

```
workspace/agents/<id>/
  personality.md
  heartbeat.md
  config.toml
  tasks/
  memories/
  messages/inbox|outbox
  skills/          # skills dédiées
```

Tests : `phase2_agents_e2e.rs`, `integration_multi_agents_b212.rs`

## Tests

| Suite | Commande |
|-------|----------|
| Rapide (CI) | `cargo test --workspace` |
| Phase 7 agent | `cargo test -p orchestrator --test phase7_agent_loop` |
| Intégration | `cargo test -p orchestrator --test integration_multi_agents_b212` |
| Charge | `cargo test -p orchestrator --test load_workspace_scale -- --ignored` |
| Property | `cargo test -p cortex --test property` |

Helpers de test : `orchestrator::testing` (`MockBundle`, `ToolScriptLlmProvider`, `build_test_facade`).

## Performance

- **Graphe** : insertions incrémentales `KnowledgeGraph::insert_memory` — éviter `from_memories` répété sur 10k+ nœuds
- **Embeddings** : cache LRU `infrastructure/src/embedding/cache.rs` (4096 entrées par défaut)
- **Profiling** : `scripts/profile.sh`

## Release

```bash
./scripts/release.sh 0.29.0
```

Puis tag `v0.29.0` → workflow GitHub `release.yml` publie l'installateur Windows.

## Contribuer

1. Branche feature depuis `main`
2. Tests + `cargo clippy -p orchestrator -- -D warnings`
3. PR avec description des crates touchés
4. Noyau P0–P5 gelé sauf correctifs — extensions via skills P6