# ARCHIVE PHASE 5 — v0.3.0
## 20 juin 2026 | TUI intégré + mode dégradé + unification bridge

**Statut :** Phase 5 clôturée. CLI, TUI et HUD partagent le contrat bridge ; démarrage unifié via `bootstrap_workspace`.

---

## Livrables

| Étape | Livrable | Statut |
|-------|----------|--------|
| 1 | TUI ratatui dans `orchestrator/src/tui/` (feature `tui`) | ✅ |
| 2 | `orchestrateur` sans args → TUI si TTY | ✅ |
| 3 | Mode dégradé (`unavailable` LLM/embedding, health `degraded`) | ✅ |
| 4 | HUD/TUI signalent services off (barre rouge, boutons désactivés) | ✅ |
| 5 | `infrastructure::bootstrap_workspace` (wiring partagé) | ✅ |
| 6 | `orchestrator::execute_command` (CLI headless = bridge) | ✅ |
| 7 | `bridge::ui_common` (état UI partagé HUD/TUI) | ✅ |
| 8 | Default config `vector_store.type = lancedb` | ✅ |
| 9 | Nettoyage `presentation-egui`, docs, tag `phase5-v0.3.0` | ✅ |

---

## Architecture Phase 5

- **Bridge** : `Command` / `Response` — seule voie pour HUD, TUI et commandes CLI (`health`, `list`, `get`, `search`, `assimilate`)
- **Bootstrap** : `infrastructure::bootstrap_workspace` — HUD, CLI, TUI
- **TUI** : `orchestrator` feature `tui`, binaire `orchestrateur`
- **HUD** : crate `hud/`, binaire `orchestrateur-hud`
- **CLI direct** : `graph`, `chat` uniquement (pas encore dans le bridge)

---

## Tests

```
cargo test -p orchestrator --features tui
cargo test -p orchestrateur-cli -p orchestrateur-hud
cargo clippy -p orchestrator --features tui -p orchestrateur-cli -p orchestrateur-hud -- -D warnings
```

---

## Tag

```bash
git tag -a phase5-v0.3.0 -m "Phase 5: TUI, mode dégradé, bootstrap + bridge unifié"
```