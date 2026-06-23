# Outils Hermess — port dans Orchestrateur

**Source :** `C:\GitDev\Projet\Hermess` · **Implémentation :** `crates/orchestrator/src/tools/hermes/`

Orchestrateur reprend les outils agent Hermess sous forme de modules Rust, filtrés par **profil de capacités**.

## Profils

| Profil | Contenu |
|--------|---------|
| `agent` | Cortex + skills + session_search + todo + memory + read_file + clarify |
| `hermes` | Tous les outils ci-dessous (y compris shell, fichiers, stubs web) |
| `full` | Tout ce qui est enregistré |

Activer dans `orchestrator.toml` :

```toml
[agent]
active_capability_profile = "hermes"   # ou "agent" pour le sous-ensemble
```

## Mapping Hermess → Orchestrateur

| Hermess | Orchestrateur | Statut |
|---------|---------------|--------|
| `skills_list` | `skills_list` | Implémenté (SKILL.md workspace) |
| `skill_view` | `skill_view` | Implémenté |
| `skill_manage` | `skill_manage` | Implémenté (P6 workspace/skills) |
| `skills` (système) | Hub + prompt skills | Inchangé (P6) |
| `session_search` | `session_search` | Implémenté (SQLite sessions) |
| `todo` | `todo` | Implémenté (`.orchestrateur/agent/todos/`) |
| `memory` | `memory` | Implémenté (MEMORY.md / USER.md agent) |
| `memories` | `memory_search/get/assimilate` | Cortex (existant) |
| `read_file` | `read_file` | Implémenté |
| `write_file` | `write_file` | Profil `hermes` |
| `patch` | `patch` | Profil `hermes` |
| `search_files` | `search_files` | Profil `hermes` (rg ou walkdir) |
| `terminal` | `terminal` | Profil `hermes` |
| `execute_code` | `execute_code` | Profil `hermes` (Python subprocess) |
| `clarify` | `clarify` | Implémenté (réponse pending UI) |
| `delegate_task` | `delegate_task` | File JSON locale (worker à brancher) |
| `cronjob` | `cronjob` | CRUD jobs JSON (scheduler à brancher) |
| `web_search` | `web_search` | Stub — provider requis |
| `browser_navigate` | `browser_navigate` | Stub — provider requis |
| `open_page` | `open_page` | Stub (alias Hermess) |
| `image_generate` | `image_generate` | Stub |
| `text_to_speech` | `text_to_speech` | Stub |
| `vision_analyze` | `vision_analyze` | Stub |
| `workspace` | `--workspace` | Concept (pas un outil) |

## Persistance agent Hermess

```
workspace/.orchestrateur/
├── agent/
│   ├── MEMORY.md      # notes agent (outil memory, target=memory)
│   ├── USER.md        # profil utilisateur (target=user)
│   └── todos/
│       └── <session>.json
├── cron/
│   └── jobs.json
└── delegations/
    └── deleg-*.json
```

## Références Hermess

- Registre : `Hermess/tools/registry.py`
- Exécution : `Hermess/agent/tool_executor.py`
- Fichiers : `Hermess/tools/file_tools.py`
- Skills : `Hermess/tools/skills_tool.py`, `skill_manager_tool.py`