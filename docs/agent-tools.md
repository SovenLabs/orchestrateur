# Outils agent étendus

**Implémentation :** `crates/orchestrator/src/tools/extended/`

Outils supplémentaires de la boucle agent, filtrés par **profil de capacités**.

## Profils

| Profil | Outils étendus |
|--------|----------------|
| `agent` | skills_list/view/manage, session_search, todo, memory, read_file, clarify |
| `full` | Tous les outils enregistrés (shell, fichiers, web stubs, …) |

Configuration dans `orchestrator.toml` :

```toml
[agent]
active_capability_profile = "agent"   # ou "full" pour tout
```

## Catalogue

| Outil | Description |
|-------|-------------|
| `skills_list` | Liste skills Markdown (tier 1) |
| `skill_view` | Contenu SKILL.md ou fichier lié |
| `skill_manage` | CRUD skills P6 |
| `session_search` | Browse / recherche / scroll sessions |
| `todo` | Liste de tâches par session |
| `memory` | MEMORY.md / USER.md agent (≠ Cortex) |
| `read_file` | Lecture paginée workspace |
| `write_file` | Écriture workspace (`full`) |
| `patch` | Modification ciblée (`full`) |
| `search_files` | Ripgrep / walkdir (`full`) |
| `terminal` | Commande shell (`full`) |
| `execute_code` | Python subprocess (`full`) |
| `clarify` | Question utilisateur structurée |
| `delegate_task` | File délégations JSON (`full`) |
| `cronjob` | CRUD jobs planifiés (`full`) |
| `web_search` | Stub — provider requis (`full`) |
| `browser_navigate` | Stub — provider requis (`full`) |
| `open_page` | Stub — alias navigateur (`full`) |
| `image_generate` | Stub (`full`) |
| `text_to_speech` | Stub (`full`) |
| `vision_analyze` | Stub (`full`) |

Les outils Cortex (`memory_search`, `memory_assimilate`, …) restent dans tous les profils mémoire/agent.

## Persistance état agent

```
workspace/.orchestrateur/
├── agent/
│   ├── MEMORY.md
│   ├── USER.md
│   └── todos/<session>.json
├── cron/jobs.json
└── delegations/deleg-*.json
```