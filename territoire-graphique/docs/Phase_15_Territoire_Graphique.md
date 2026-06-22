# Phase 15 — Territoire Graphique PoC (Avancée)

**Date :** 22 juin 2026
**Branche :** feature/phase-15-territoire-graphique

## Réalisations de la Phase 15

### 1. Shader amélioré (glow + déformation + effet pixels vivants)
- Shader WGSL complet avec :
  - Déformation de la sphère basée sur l'activité
  - Effet de glow dynamique
  - Bruit pour simulation d'effet "pixels vivants"
  - Pulsation temporelle

### 2. Particules connectées à l'activité
- Le nombre de particules et leur émission sont maintenant contrôlés dynamiquement par `activity_intensity` dans `brain_sphere.gd`.

### 3. Communication WebSocket (Option B)
- Ajout d'un client WebSocket dans Godot (`brain_sphere.gd`).
- Connexion prévue sur `ws://127.0.0.1:28790` (port du daemon Rust).
- Fallback sur simulation GDExtension si la connexion échoue.

### 4. Premier panneau dockable : Monitoring
- Panneau `MonitoringPanel` avec ProgressBar et label d'intensité.
- Mise à jour en temps réel via la fonction `update_activity()`.

## Structure actuelle

```
territoire-graphique/
├── godot-project/
│   ├── scenes/
│   │   ├── MainTerritory.tscn
│   │   ├── BrainSphere.tscn
│   │   └── MonitoringPanel.tscn
│   ├── scripts/
│   │   ├── brain_sphere.gd
│   │   └── monitoring_panel.gd
│   └── shaders/
│       └── brain_living_shader.wgsl
├── rust-gdextension/
└── docs/
    └── Phase_15_Territoire_Graphique.md
```

## Prochaines étapes (Phase 16)
- Amélioration des shaders et particules
- Intégration complète WebSocket côté Rust (serveur)
- Ajout d'autres panneaux (Chat, Memory List)

**Objectif Phase 20** : Visuel impressionnant et fonctionnel de la Boule de Pixels Vivante + Territoire Graphique.
