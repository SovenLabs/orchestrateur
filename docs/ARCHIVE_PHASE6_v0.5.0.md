# ARCHIVE — Phase 6 v0.5.0

**Date :** Juin 2026  
**Version Cargo :** 0.5.0  
**Tag suggéré :** `phase6-v0.5.0`

---

## Objectif Phase 6

Compléter l'**Esprit** (skills opérationnelles + bridge v0.5) et les **Peaux** (chat HUD/TUI), préparer le **packaging Windows**.

---

## Livrables

### Bridge v0.5

| Commande | Réponse |
|----------|---------|
| `Chat { message }` | `ChatReply { reply }` |
| `ListSkills` | `SkillList { skills }` |
| `ExecuteSkill { name, context }` | `SkillResult { message }` |

Types : `BridgeSkillContext`, `SkillSummary`.

### Skills opérationnelles

- `list_memories`, `search`, `assimilate`, `noop`
- Enregistrement via `SkillRegistry::with_operational_skills()`
- Exécution via facade et bridge (pas de duplication métier)

### CLI

- `orchestrateur chat` → bridge `Command::Chat`
- `orchestrateur skill list` / `skill run <name>` → bridge

### Peau

- **HUD** : onglet Chat (`HudMainView::Chat`)
- **TUI** : vue Chat (`c`), vues Graph/Audit, événements domaine

### Packaging / installateur Windows

- `scripts/build-installer.ps1` — release + zip + **Setup.exe** (Inno Setup 6)
- `scripts/package-windows.ps1` — zip uniquement
- `installer/orchestrateur.iss` — script Inno (raccourcis, workspace `%APPDATA%`)

---

## Vérification

```powershell
cargo test -p orchestrator --features tui
cargo test -p orchestrateur-cli -p orchestrateur-hud
cargo clippy -p orchestrator --features tui -p orchestrateur-cli -p orchestrateur-hud -- -D warnings
.\scripts\build-installer.ps1 -InstallInno
```

---

## Hors scope (Phase 7+)

- Chargement dynamique de skills (plugins)
- Installeur MSI / code signing
- Thought loop agentic (Grok choisit la skill)

---

*Orchestrateur — Sovën, 2026*