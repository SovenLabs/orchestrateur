# ARCHIVE — Phase 9 v0.9.0

**Date :** Juin 2026  
**Version Cargo :** 0.9.0  
**Tag suggéré :** `phase9-v0.9.0`

---

## Objectif Phase 9

Provider registry typé (12 LLM + 5 embeddings), résolution TOML, client MCP stdio, outils agent `mcp_*`.

---

## Livrables

### Provider registry (`orchestrator/src/providers/`)

| Module | Rôle |
|--------|------|
| `descriptor.rs` | Catalogue statique 12 LLM + 5 embeddings |
| `registry.rs` | `ProviderRegistry` — lookup typé |
| `profile.rs` | `ProviderProfiles` — surcharges `[provider_profiles.<id>]` |

**LLM enregistrés :** xai, ollama, openai, anthropic, groq, openrouter, together, deepseek, mistral, perplexity, lmstudio, azure_openai

**Embeddings :** ollama, openai, voyage, huggingface, fastembed

### Infrastructure

| Adapter | Chemin |
|---------|--------|
| `OpenAiCompatibleLlmProvider` | `infrastructure/src/llm/openai_compatible.rs` |
| `AnthropicLlmProvider` | `infrastructure/src/llm/anthropic.rs` |
| `OpenAiEmbeddingsProvider` | `infrastructure/src/embedding/openai_embeddings.rs` |
| Résolution registre | `infrastructure/src/providers/resolve.rs` |

### MCP (`crates/mcp/`)

| Composant | Rôle |
|-----------|------|
| `McpStdioClient` | JSON-RPC 2.0 stdio, `initialize`, `tools/list`, `tools/call` |
| `McpManager` | Implémente `orchestrator::McpGateway` |
| Wiring | `build_mcp_gateway()` dans `infrastructure/wiring.rs` |

### Outils agent

| Outil | Rôle |
|-------|------|
| `mcp_list_tools` | Liste outils MCP |
| `mcp_call` | Appelle `server` + `tool` + `arguments` |

`AgentLoop` charge ces outils si `AppDependencies.mcp` est présent.

### CLI

```powershell
orchestrateur providers list
orchestrateur providers list --kind llm
orchestrateur providers list --kind embedding
```

### Configuration TOML

```toml
[provider_profiles.openai]
model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"

[mcp]
enabled = true

[[mcp.servers]]
name = "filesystem"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "."]
```

---

## Vérification

```powershell
cargo test -p orchestrator provider_registry
cargo test -p orchestrator
cargo test -p mcp
cargo test -p infrastructure
cargo clippy -p orchestrator -p infrastructure -p mcp -- -D warnings
```

---

## Hors scope (Phase 10+)

- 15+ canaux messaging
- Toolsets groupés
- Auto-assimilation systématique
- Plugins dynamiques

---

*Orchestrateur — Sovën, 2026*