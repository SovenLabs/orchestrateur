# Phase 17 — Panneaux Dockables

**Date :** 22 juin 2026  
**Branche :** `main`  
**Version :** 0.18.0  
**Statut :** Terminé

---

## Objectif

Enrichir le **Territoire Graphique** avec des panneaux dockables et interactifs autour de la **Boule de Pixels Vivante**, tout en conservant la métaphore spatiale du territoire.

---

## Réalisations

### 1. Panneau Chat (`ChatPanel.tscn`)

- Saisie multiligne (`TextEdit`) — Entrée envoie, Maj+Entrée nouvelle ligne
- Affichage des réponses agent (`RichTextLabel` BBCode)
- Envoi via autoload `DaemonClient.execute_chat()`
- Pulsation boule à l'envoi et à la réception (`brain_pulse_requested`)

### 2. Panneau Memory List (`MemoryListPanel.tscn`)

- Liste des mémoires avec recherche (`LineEdit` → `Search` / `List`)
- Sélection → détail (`GetMemory`) + signal `memory_selected`
- Rafraîchissement auto à la connexion WS et après assimilation
- Méthode `focus_memory()` pour navigation depuis le graphe

### 3. Panneau Graph (`GraphPanel.tscn`)

- Addon `force_directed_graph` — simulation force-directed simplifiée
- Hubs principaux via `GraphSummary` (`memory_id`, `title`, `inbound_links`)
- Clic nœud → `hub_selected` → focus dans Memory List
- Stats nœuds / arêtes

### 4. Système de docking

| Composant | Rôle |
|-----------|------|
| `DockPanel` | Base titre + bouton détachement (préparation Phase 18) |
| `DockLayout` | Sauvegarde `user://territory_layout.json` (offsets splits) |
| `MainTerritory.tscn` | `HSplitContainer` gauche (mémoires + graphe) · centre transparent · droite (chat + monitoring) |

### 5. Communication centralisée

| Composant | Rôle |
|-----------|------|
| `DaemonClient` (autoload) | Hub WS Option B — `execute_*`, signaux `command_completed`, `brain_pulse_requested`, `activity_changed` |
| `TerritoryManager` | Relie boule ↔ daemon, graphe ↔ mémoires, persistance layout |

### 6. Boule de Pixels Vivante

- `pulse_activity(boost, duration)` — boost particules + intensité shader
- `update_brain_activity()` — lissage activité daemon (Health poll)

---

## Structure

```
territoire-graphique/godot-project/
├── scenes/
│   ├── MainTerritory.tscn      # scène principale
│   ├── BrainSphere.tscn
│   ├── MonitoringPanel.tscn
│   ├── ChatPanel.tscn
│   ├── MemoryListPanel.tscn
│   └── GraphPanel.tscn
├── scripts/
│   ├── daemon_client.gd        # hub WS étendu
│   ├── territory_manager.gd
│   ├── dock_layout.gd
│   ├── dock_panel.gd
│   ├── brain_sphere.gd
│   ├── chat_panel.gd
│   ├── memory_list_panel.gd
│   ├── graph_panel.gd
│   └── monitoring_panel.gd
└── addons/
    └── force_directed_graph/
        ├── plugin.cfg
        ├── plugin.gd
        └── force_graph.gd
```

---

## Lancement

```bash
# Terminal 1 — daemon
orchestrateur daemon run

# Terminal 2 — Godot 4.7+
# Ouvrir territoire-graphique/godot-project → F5
```

Token WS par défaut : `dev` (`ORCHESTRATEUR_DAEMON_TOKEN`).

---

## Prochaines étapes (Phase 18)

- Multi-fenêtrage (détachement panneaux via `detach_requested`)
- Effets visuels globaux du territoire
- Connexion plus profonde avec `AgentLoop` / Cortex
- Tests UX fluidité docking

---

**Fin de la Phase 17**