## Résumé

<!-- Que change cette PR ? -->

## Zone

- [ ] **P6 — Skills** (`workspace/skills/`, `plugins/`, `docs/skills*.md`)
- [ ] Noyau Rust (P0–P5) — *réservé aux correctifs sécurité critique post-gel*

## Checklist skills (P6)

- [ ] Pas d'écriture directe dans le vault mémoire — outils Cortex uniquement
- [ ] `skill.toml` conforme au schéma v1.0 ([`docs/skills-schema.md`](../docs/skills-schema.md))
- [ ] Si catalogue : `catalog.json` avec `"version": 1`
- [ ] `orch skill list` / `orch skill run <id>` testés localement