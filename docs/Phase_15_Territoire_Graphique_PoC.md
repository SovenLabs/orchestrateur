# Phase 15 — Territoire Graphique PoC (Avancée)

**Date :** 22 juin 2026  
**Branche :** `main` (workflow direct)  
**Version :** 0.16.0  
**Objectif :** Poser les bases visuelles et techniques du **Territoire Graphique** avec une Boule de Pixels Vivante.

---

## Résumé

| Élément | Statut | Fichier(s) |
|---------|--------|------------|
| Shader Boule vivante | ✅ | `godot-project/shaders/brain_living_shader.gdshader` |
| Script Boule + particules | ✅ | `godot-project/scripts/brain_sphere.gd` |
| Client WebSocket Option B | ✅ | `godot-project/scripts/daemon_client.gd` (autoload) |
| Fallback HTTP /health | ✅ | `daemon_client.gd` |
| Fallback GDExtension (stub) | ✅ | `rust-gdextension/` + `.gdextension` |
| Panneau Monitoring | ✅ | `godot-project/scenes/monitoring_panel.tscn` |
| Structure PoC | ✅ | `scenes/`, `scripts/`, `shaders/` |

---

## Détails techniques

### 1. Shader (`brain_living_shader.gdshader`)

- Déformation vertex selon `activity`
- Glow pulsé via `time`
- Bruit procédural « pixels vivants »
- Uniforms : `time`, `activity`

### 2. Boule (`brain_sphere.gd`)

- Connexion au signal `DaemonClient.activity_changed`
- `update_brain_activity(intensity)` met à jour shader + `GPUParticles3D`
- Particules : 24 → 220 selon l'intensité

### 3. Monitoring Panel

- Ancré en haut à droite (dockable via `PanelContainer`)
- `update_activity()` — barre + label pourcentage
- Statut connexion daemon (WS ou fallback)

### 4. Communication

**Option B — WebSocket** `ws://127.0.0.1:28790/ws`

1. `connect` + token (`ORCHESTRATEUR_DAEMON_TOKEN`, défaut `dev`)
2. `execute` + `HealthCheck` toutes les 2 s
3. Mapping → intensité via `ActivityMapper`

**Fallbacks :**

- HTTP `GET /health` si WS indisponible
- Pulsation idle si daemon hors ligne
- GDExtension Rust (build optionnel) — mapping partagé dans `activity.rs`

---

## Structure après Phase 15

```
territoire-graphique/
├── communication.md
├── README.md
├── godot-project/
│   ├── project.godot          # autoload DaemonClient
│   ├── main.tscn
│   ├── scenes/
│   │   ├── brain_sphere.tscn
│   │   └── monitoring_panel.tscn
│   ├── scripts/
│   │   ├── daemon_client.gd
│   │   ├── brain_sphere.gd
│   │   ├── monitoring_panel.gd
│   │   ├── activity_mapper.gd
│   │   └── main.gd
│   └── shaders/
│       └── brain_living_shader.gdshader
└── rust-gdextension/
    ├── src/activity.rs
    ├── territoire_gdextension.gdextension
    └── Cargo.toml
```

---

## Lancement

```powershell
# Terminal 1 — daemon Rust
$env:ORCHESTRATEUR_DAEMON_TOKEN = "dev"
.\target\release\orchestrateur.exe daemon run --workspace workspace

# Terminal 2 — Godot 4.3+
# Ouvrir territoire-graphique/godot-project/ et F5
```

---

## Prochaines étapes (Phase 16+)

- Territoire multi-fenêtres
- Chat / assimilation dans l'UI Godot
- Shaders avancés (connexions graphe visuelles)
- GDExtension godot-rust complète

---

**Fin Phase 15**