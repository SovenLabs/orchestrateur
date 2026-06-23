# Tests de charge — Phase 7

| Fichier | Seuil | Commande |
|---------|-------|----------|
| `crates/orchestrator/tests/load_workspace_scale.rs` | 10 000 mémoires, 100 agents | voir ci-dessous |
| `crates/cortex/tests/scalability.rs` | 5 000 nœuds graphe < 2 s | `cargo test -p cortex scalability -- --ignored` |
| `crates/infrastructure/tests/hardcore_perf.rs` | 2 000 mémoires concurrentes | `cargo test -p infrastructure hardcore_perf -- --ignored` |

## Exécution locale

```powershell
cargo test -p orchestrator --test load_workspace_scale -- --ignored
cargo test -p cortex --test scalability -- --ignored
```

Ces tests sont exclus de la CI rapide et exécutés dans le job `test-heavy` via `cargo test --workspace -- --ignored`.