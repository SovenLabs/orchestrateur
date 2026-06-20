# ARCHIVE PHASE 3 — v0.1.0
## 20 juin 2026 | Orchestrateur opérationnel

**Statut :** Phase 3 clôturée. Système opérationnel avec persistance réelle et intégration IA multi-provider.

---

## Livrables (9 étapes)

| Étape | Livrable | Statut |
|-------|----------|--------|
| 1 | Traits `EmbeddingProvider` + `LlmProvider` | ✅ |
| 2 | `LancedbVectorStore` | ✅ |
| 3 | Factory VectorStore + config TOML | ✅ |
| 4 | `OllamaEmbeddingProvider` | ✅ |
| 5 | Factory EmbeddingProvider + chaîne fallback | ✅ |
| 6 | `XaiGrokProvider` + `OllamaLlmProvider` | ✅ |
| 7 | Flux `assimilate` (LLM → draft → Cortex) | ✅ |
| 8 | Tests intégration + CLI `clap` | ✅ |
| 9 | Archive + tag `phase3-v0.1.0` | ✅ |

---

## Architecture Phase 3

- **Cortex** : `Embedding`, `EmbeddingProvider`, `VectorStore`, `MemoryRepository`
- **Orchestrator** : `LlmProvider`, `MemoryDraft`, `AssimilateFromText`, config multi-provider
- **Infrastructure** : `LancedbVectorStore`, `OllamaEmbeddingProvider`, `XaiGrokProvider`, factories, `build_app_dependencies`
- **CLI** : `list`, `search`, `assimilate`, `graph`, `chat`

---

## Tests

```
cargo test -p cortex -p orchestrator -p infrastructure
→ 86 tests (42 + 36 + 5 + 3 intégration), 1 ignored (Ollama E2E)

cargo clippy -p cortex -p orchestrator -p infrastructure -p orchestrateur-cli -- -D warnings
→ 0 warning
```

---

## Configuration

Voir `workspace/config/orchestrator.toml.example` — sections `[providers]`, `[xai]`, `[ollama]`, `[vector_store]`, `[lancedb]`.

Variables d'environnement : `XAI_API_KEY` (ou `api_key_env` configurable).

---

## Prérequis build

- Rust stable
- `protoc` : fourni dans `.tools/protoc/` (configuré via `.cargo/config.toml`)

---

## Prochaine phase

Phase 4 : HUD egui minimal + Skills avancées.