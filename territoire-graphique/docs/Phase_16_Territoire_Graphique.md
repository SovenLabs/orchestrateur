# Phase 16 — Amélioration Shaders, Particules et WebSocket

**Date** : 22 juin 2026
**Branche** : feature/phase-15-territoire-graphique

## Objectif
Améliorer significativement la qualité visuelle de la Boule et poser les bases du WebSocket côté Rust.

## Réalisations

### 1. Shader avancé (v2)
- Déformation plus organique et réactive
- Meilleur effet de glow sur les bords
- Variation de couleur selon l'activité
- Effet pixels plus prononcé

### 2. Particules améliorées
- Nombre de particules et émission entièrement dynamiques
- Meilleure réactivité visuelle

### 3. WebSocket côté Rust (structure)
- Ajout de dépendances tokio + tokio-tungstenite dans le crate
- Structure de base pour un serveur WebSocket
- Méthode `start_websocket_server()` exposée

### 4. Amélioration du script
- Meilleure gestion de l'intensité
- Mise à jour automatique du panneau Monitoring

## Prochaines étapes (Phase 17)
- Finaliser le serveur WebSocket fonctionnel
- Ajouter d'autres panneaux (Chat, Memory List)
- Améliorer les effets visuels globaux

**Objectif Phase 20** : Visuel impressionnant et réactif du Territoire Graphique.
