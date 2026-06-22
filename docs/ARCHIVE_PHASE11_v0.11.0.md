# ARCHIVE — Phase 11 v0.11.0

**Date :** Juin 2026  
**Version Cargo :** 0.11.0  
**Tag suggéré :** `phase11-v0.11.0`

---

## Objectif Phase 11

Hub de skills filesystem + plugins subprocess dynamiques, chargement auto au démarrage, CLI `skills-hub list`, bridge `SkillSummary` enrichi (`source`, `version`).

---

## Livrables

### Skills hub (`orchestrator/src/skills/`)

| Module | Rôle |
|--------|------|
| `hub.rs` | `SkillsHub` — scan `workspace/skills/*/skill.toml` + entrées inline |
| `manifest.rs` | Parse `skill.toml` (`kind = subprocess`) |
| `plugin.rs` | `SubprocessPluginSkill` — exécution async + timeout |
| `registry.rs` | `with_operational_skills_and_hub()`, `reload_hub()` |
| `skill.rs` | `SkillSource`, `SkillEntry`, trait `Skill` → `&str` |

### Format manifeste (`workspace/skills/<id>/skill.toml`)

```toml
[skill]
id = "pong"
description = "Plugin démo"
version = "0.1.0"
enabled = true

[subprocess]
command = "cmd"
args = ["/c", "echo", "pong"]
stdin_json = false
timeout_secs = 5
```

Avec `stdin_json = true` : envoie `SkillContext` JSON sur stdin, attend `{"message":"..."}` sur stdout.

### Configuration TOML

```toml
[skills_hub]
enabled = true
directory = "skills"
auto_load = true

[[skills_hub.entries]]
id = "inline-echo"
description = "Echo inline"
command = "echo"
args = ["inline"]
enabled = true
```

### Bridge / facade

| Champ `SkillSummary` | Rôle |
|----------------------|------|
| `source` | `builtin` ou `hub` |
| `version` | Version plugin (optionnel) |

`OrchestratorFacade::new()` charge automatiquement le hub si `skills_hub.auto_load = true`.

### CLI

```powershell
orchestrateur skills-hub list
orchestrateur skills-hub path
orchestrateur skill list          # affiche [builtin] / [hub]
orchestrateur skill run pong      # exécute plugin hub
```

### Plugin démo

`workspace/skills/pong/skill.toml` — retourne `pong` via subprocess.

---

## Vérification

```powershell
cargo test -p orchestrator skills_hub
cargo test -p orchestrator config::tests::loads_skills_hub_from_toml
cargo test -p orchestrator skills::registry
cargo clippy -p orchestrator -- -D warnings
```

---

## Hors scope (Phase 12+)

- Plugins natifs `.dll` / `.so` (libloading)
- Inbound réel canaux stub (WhatsApp, Matrix, …)
- Marketplace skills distant
- Agent choisit la skill automatiquement (agentic)

---

*Orchestrateur — Sovën, 2026*