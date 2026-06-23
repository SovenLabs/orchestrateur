# Harness intégral — Esprit + Cortex

**Version :** 0.28.0 · **Protocole WS :** 1.2.0

Orchestrateur est un **produit unique** : le cerveau (Cortex) et l'esprit (agent) ne sont pas des outils séparés.

Hiérarchie du dépôt, priorités et gel noyau : [`project-hierarchy.md`](project-hierarchy.md).

## Démarrage

```powershell
# Terminal 1 — daemon harness
$env:ORCHESTRATEUR_DAEMON_TOKEN = "dev"
just daemon

# Terminal 2 — desktop (peau harness par défaut)
just desktop-dev
```

## CLI headless

**Installé :** un seul binaire **`orch`** dans PATH (cible produit).

**Alias clap** (même binaire) : `orchestre`, `orchestrateur` — équivalents dans l'aide et les sous-commandes, pas trois `.exe` distincts. Voir [`project-hierarchy.md`](project-hierarchy.md) section 6.

```powershell
# Installation (une fois) — installateur unique
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Dev
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Dev -InstallDaemon
# Release (nécessite une GitHub Release publiée) :
# powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.ps1 | iex"

# Menus interactifs (navigation ↑↓, hints entre parenthèses)
orchestrateur setup       # centre de commande harness
orchestrateur settings    # configuration (profil, LLM, canaux, …)
orchestrateur onboard     # assistant premier lancement (sans flags)

# Onboard harness (flags = non interactif)
orch onboard --local-only --install-daemon --workspace workspace
orch configure --profile local_only --llm ollama --workspace workspace

# Mise à jour autonome (exécute install.ps1, pas d'instructions manuelles)
orchestrateur update              # dev si dépôt local détecté, sinon release
orchestrateur update --check      # compare version locale vs GitHub
orchestrateur update --dev        # recompile + réinstalle ~/.orchestrateur/bin
orchestrateur update --release    # installateur release GitHub

# Diagnostic enrichi (Cortex + daemon/gateway HTTP + tokens + egress)
orch doctor --workspace C:\GitDev\Projet\orchestrateur\workspace

# Daemon / gateway
orch daemon install --workspace workspace
orch daemon status --workspace workspace
orch gateway status --workspace workspace
orch harness run --workspace workspace   # superviseur daemon+gateway

# Canaux et providers
orch channels status --workspace workspace
orch channels enable telegram --workspace workspace
orch providers test --kind llm --workspace workspace
orch providers set ollama --workspace workspace

orch harness smoke --workspace workspace
orch watcher status --workspace workspace
orch draft list --workspace workspace
orch assimilate "idée à retenir" --tags insight --workspace workspace

# Skills (P6 — hub + marketplace)
orch skill list --workspace workspace
orch skill run pong --workspace workspace
orch skill install market-echo --workspace workspace
orch skill update --workspace workspace
```

Schémas hub / catalogue : [`skills-schema.md`](skills-schema.md).

## MCP (Claude Code, Cursor, …)

```powershell
orch mcp serve --workspace workspace
```

Outils exposés : `cortex_search`, `cortex_get`, `cortex_graph`, `cortex_assimilate`, `draft_list`, `draft_publish`, `esprit_chat`, `harness_health`.

## Boucle intégrée

1. **Esprit** reçoit un message → preprocess (expand/compress)
2. **Cortex** fournit contexte (search + graphe + kinds)
3. **Esprit** peut assimiler via outils natifs
4. **Watcher** produit des drafts → publication humaine ou CLI
5. Tour suivant : contexte enrichi automatiquement

## Règle skills

Les skills sont des extensions **Esprit** — elles ne stockent jamais la mémoire directement. Toute persistance passe par les outils Cortex (`memory_assimilate`, `draft_publish`, …).

Voir [`workspace/skills/RESOLVER.md`](../workspace/skills/RESOLVER.md).