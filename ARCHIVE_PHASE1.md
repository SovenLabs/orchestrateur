# ARCHIVE PHASE 1 — Cortex certifié
## Version : v0.2.0-final (document) | Cargo : **0.1.0** | Date : 20 juin 2026
## Statut : **Phase 1 CLOSED** (après Phase 1bis — réconciliation)

---

## 1. Objectif Phase 1bis

Fusionner l'archive stratégique (sandbox) avec le dépôt local `C:\Users\cyril\Dev\Orchestre`, une seule vérité code + prompt.

---

## 2. Correctifs intégrés

| Correctif | Avant | Après |
|-----------|-------|-------|
| Markdown parser | `splitn(3, "---")` fragile | `markdown_parser.rs` — délimiteurs explicites, CRLF, corps avec `---` |
| Frontmatter | champs inconnus ignorés | `#[serde(deny_unknown_fields)]` |
| Tags | pas de limite | max **64** chars → `CortexError::TagTooLong` |
| Memory backlinks | `set_backlinks` seul | `add_or_update_backlink` + dédup par score |
| KnowledgeGraph | réinsertion naïve | `HashSet` nœuds + dédup arêtes à l'insertion |
| Lints | absents | `unsafe_code=forbid`, clippy `unwrap_used/expect_used/panic=deny` |
| Version Cargo | 0.0.2 | **0.1.0** |

## 3. Conservé du repo local (mieux que l'archive)

- `Memory::new` → `Result` + validation titre/contenu
- `CortexError` unifié (pas de `CortexTagError` séparé)
- Structure `domain/` / `ports/` / `services/` (pas de `error` top-level)
- `MemoryDraft` reste dans `orchestrator` — non modifié

---

## 4. Tests

```
cargo test -p cortex → 37 tests OK (Rust stable)
cargo llvm-cov -p cortex → 91.51 % lignes | 92.64 % régions (seuil > 85 % ✅)
```

Nouveaux tests : `deny_unknown_fields`, corps avec `---`, CRLF, tag 64, backlink merge, graphe idempotent.

Couverture (`cargo llvm-cov -p cortex`) : **91.51 %** lignes | **92.64 %** régions.

---

## 5. Checklist clôture

- [x] Correctifs archive portés
- [x] `cargo test -p cortex` vert (stable)
- [x] `ARCHIVE_PHASE1.md` dans le repo
- [x] Cargo **0.1.0**
- [x] Tag git `phase1-closed`

---

## 6. Prochaine étape

**Phase 2 GO** — crate `orchestrator` : facade, use cases, Skill Registry skeleton (sans Grok réel).

*Fin archive Phase 1 — une seule vérité.*