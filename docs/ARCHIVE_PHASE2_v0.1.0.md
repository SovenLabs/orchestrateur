# ARCHIVE – PHASE 2 – ESprit (Orchestrator)
## Version v0.1.0 | Démarrée le 20 juin 2026 | **CLOSED** le 20 juin 2026
## Cargo workspace : **0.1.0** | Tag git : `phase2-closed`

**Statut officiel :** Phase 2 **clôturée**.

Le crate `orchestrator` (L'Esprit) expose une facade stable, 5 use cases testables en mémoire, un Skill Registry squelette et des mocks in-memory des 3 ports Cortex.

---

## 1. Objectif atteint

Construire la couche **Application / Esprit** 100 % testable sans disque, sans réseau, sans Grok ni Ollama.

**Livrable bloquant n°1 validé :** mocks in-memory (`InMemoryMemoryRepository`, `InMemoryVectorStore`, `InMemoryEmbeddingProvider`).

---

## 2. Structure livrée

```
orchestrator/src/
├── lib.rs
├── error.rs
├── config.rs
├── deps.rs
├── facade.rs
├── memory_draft.rs          # conservé depuis Phase 1 (hors Cortex)
├── use_cases/
│   ├── list_memories.rs
│   ├── get_memory.rs
│   ├── save_memory.rs
│   ├── search_memories.rs
│   └── assimilate_from_draft.rs
├── skills/
│   ├── skill.rs             # name/desc sync, execute async
│   └── registry.rs          # noop incluse
└── testing/
    └── mocks.rs
```

---

## 3. Flux `assimilate_from_draft` (dry-run)

1. `MemoryDraft::into_memory()` — validation domaine
2. Embedding via `EmbeddingProvider`
3. Corpus existant → `BacklinkCalculator::compute_semantic_backlinks`
4. Wikilinks explicites → `merge_backlinks` → `apply_to_memory`
5. `MemoryRepository::save` + `VectorStore::upsert`
6. Retour `(Memory, Vec<DomainEvent>)` avec `MemoryAssimilated`

---

## 4. Tests et qualité

```
cargo test -p orchestrator  → 26 tests OK
cargo test -p cortex        → 37 tests OK (non-régression)
cargo clippy -p orchestrator --no-deps -- -D warnings → OK
cargo llvm-cov -p orchestrator → 93.48 % lignes | 93.00 % régions (seuil ≥ 80 % ✅)
```

---

## 5. Checklist clôture (section 8 — validée)

- [x] `cargo test -p orchestrator` vert à 100 %
- [x] `cargo clippy -p orchestrator --no-deps -- -D warnings` zéro erreur orchestrator
- [x] `cargo test -p cortex` non-régression
- [x] Couverture ≥ 80 % use cases + facade
- [x] 3 mocks fonctionnels, thread-safe, utilisés dans tous les tests
- [x] Flux `assimilate_from_draft` câblé (backlinks + événements)
- [x] `OrchestratorFacade` expose les 5 use cases
- [x] `SkillRegistry` avec skill `noop`
- [x] Kill Criteria Phase 2 respectés
- [x] `MemoryDraft` dans `orchestrator` uniquement (pas dans Cortex)
- [x] `docs/ARCHIVE_PHASE2_v0.1.0.md` présent
- [x] Crate `orchestrator` version **0.1.0** (workspace)
- [x] Tag git `phase2-closed`

---

## 6. Prochaine étape

**Phase 3 GO** — crate `infrastructure` : `FileMemoryRepository`, `LancedbVectorStore`, `OllamaEmbeddingProvider`.

*Fin archive Phase 2 — une seule vérité.*