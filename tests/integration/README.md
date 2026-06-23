# Tests d'intégration — Phase 7

Les scénarios multi-agents et B212 sont implémentés dans le crate `orchestrator` :

| Fichier | Scénario |
|---------|----------|
| `crates/orchestrator/tests/integration_multi_agents_b212.rs` | Messagerie inter-agents + workflow B212 |
| `crates/orchestrator/tests/phase3_b212_workflow.rs` | Workflow desk B212 (6 étapes) |
| `crates/orchestrator/tests/phase2_agents_e2e.rs` | Cycle de vie agents persistants |

## Exécution

```powershell
cd C:\GitDev\Projet\orchestrateur
cargo test -p orchestrator --test integration_multi_agents_b212
cargo test -p orchestrator --test phase3_b212_workflow
cargo test -p orchestrator --test phase2_agents_e2e
```

Suite complète (incluant tests `#[ignore]`) :

```powershell
cargo test -p orchestrator -- --ignored
```