# Guide Utilisateur — Orchestrateur

**CLI :** `orch` (alias `orchestre`, `orchestrateur`) · **Version :** 0.28.0

## Installation

Installateur **style Hermes** : étapes automatisées (Git, clone, release ou build, PATH, workspace).

```powershell
# Windows — one-liner (recommandé)
iex (irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/scripts/install.ps1)

# Version fixe
$env:ORCHESTRATEUR_VERSION = "0.28.0"
iex (irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/scripts/install.ps1)

# Daemon auto au logon
$env:ORCHESTRATEUR_INSTALL_DAEMON = "1"
iex (irm https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/scripts/install.ps1)
```

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/SovenLabs/orchestrateur/main/install.sh | bash
```

```powershell
# Développement (depuis clone)
git clone https://github.com/SovenLabs/orchestrateur.git
cd orchestrateur
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Dev
```

Si aucune release GitHub n’est publiée, le one-liner **compile automatiquement** depuis les sources (comme Hermes installe Python/uv). Si PowerShell refuse l’exécution, utilisez `-ExecutionPolicy Bypass`.

Première configuration :

```powershell
orch onboard
orch doctor
```

Le workspace par défaut est `./workspace` (ou `%APPDATA%\Orchestrateur\workspace` après install release).

## Démarrage quotidien

```powershell
# Terminal 1 — daemon (clients desktop / Godot)
$env:ORCHESTRATEUR_DAEMON_TOKEN = "dev"
orch daemon run

# Terminal 2 — desktop (optionnel)
just desktop-dev
```

Santé : `http://127.0.0.1:28790/health`

## Cortex — mémoires

| Commande | Action |
|----------|--------|
| `orch memory list` | Liste les souvenirs Markdown |
| `orch memory show <id>` | Détail d'une mémoire |
| `orch memory search "requête"` | Recherche sémantique |
| `orch chat "question"` | Tour agent libre |

Kinds de mémoire : `decision`, `pattern`, `progress`, `context`, `business`, `dead_end`.

## Agents persistants

| Commande | Action |
|----------|--------|
| `orch agent list` | Agents enregistrés |
| `orch agent create <id> <nom> <rôle>` | Nouvel agent |
| `orch agent wake <id>` | Réveille l'agent |
| `orch agent send <from> <to> "message"` | Messagerie inter-agents |
| `orch agent turn <id> "message"` | Tour de conversation |

Les arcs visuels Godot s'activent sur `agent send` et `agent_wake`.

## Skills

| Commande | Action |
|----------|--------|
| `orch skill list` | Skills du hub |
| `orch skill create <id>` | Scaffolding skill |
| `orch skill install <id>` | Depuis marketplace locale |
| `orch skill run <id>` | Exécution manuelle |

## B212 (trading desk)

```powershell
orch b212 init-agents
orch b212 analyze BTCUSDT --session london --lookback 24
orch b212 proposals list
```

Voir [`B212_INTEGRATION.md`](B212_INTEGRATION.md).

## Watcher et brouillons

```powershell
orch watcher status
orch draft list
orch draft publish <id>
```

Le watcher surveille les sessions Markdown et propose des brouillons LLM à publier.

## Sauvegarde

```powershell
orch backup plan
orch backup create
orch backup restore .\workspace\backups\orchestrateur-backup-20260623T120000Z
```

Sauvegarde : mémoires, LanceDB, sessions SQLite, agents, audit, skills, B212.

## Mode dégradé (sans IA)

L'application démarre même si Ollama/xAI sont absents :

- Liste / recherche fichier : OK
- Recherche sémantique / chat : désactivés
- `orch doctor` indique l'état des providers

## Interfaces

| Interface | Rôle |
|-----------|------|
| **CLI** | Scripts, automation, `daemon run` |
| **Tauri Desktop** | Chat cosmique, drafts, watcher |
| **Godot Territoire** | Visualisation 3D, monitoring agents |

## Aide rapide

```powershell
orch --help
orch agent --help
orch skill --help
orch backup --help
```