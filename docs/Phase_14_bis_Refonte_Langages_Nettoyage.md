# Phase 14 bis — Refonte des langages + Grand Nettoyage

**Version :** 0.15.0 · Juin 2026  
**Tag Git cible :** `phase14bis-v0.15.0`  
**Objectif :** Nettoyer radicalement les anciens moteurs graphiques et poser les fondations d'une architecture multi-langages cohérente avec la vision **Territoire Graphique**.

---

## 1. Contexte et décision stratégique

Suite à l'analyse complète du dépôt et des discussions sur la vision du **Territoire Graphique** :

- Le **cœur IA** (Cortex + Agent + Bridge + Skills + Tools + Gateway + Security) reste **intégralement en Rust**.
- Les anciens moteurs graphiques (**egui** via `crates/hud` et **ratatui** via `crates/tui`) ont été **supprimés**.
- Le seul changement de langage concerne **la couche de présentation visuelle** (Godot 4).

### Choix validé : Option B – WebSocket local

| Critère | Bénéfice |
|---------|----------|
| Découplage | Godot et Rust évoluent indépendamment |
| Longévité | Le daemon peut tourner sans client visuel |
| Testabilité | Protocole `Command`/`Response` réutilisé |
| Migration | Remplacement Godot → Bevy sans toucher le backend |

**Architecture retenue :**

- Daemon Rust : `orchestrateur daemon run` — WS `ws://127.0.0.1:28790/ws`
- Client Godot : `territoire-graphique/godot-project/`
- Protocole : [`territoire-graphique/communication.md`](../territoire-graphique/communication.md)

---

## 2. Composants — état après refonte

| Composant | Décision | Statut |
|-----------|----------|--------|
| `crates/cortex` | Rester intact | ✅ |
| `crates/orchestrator` | Rust + daemon WS (`websocket-server`) | ✅ |
| `crates/infrastructure` | Rester intact | ✅ |
| `crates/hud` | Supprimé (egui) | ✅ |
| `crates/tui` + `src/tui` | Supprimé (ratatui) | ✅ |
| `crates/cli` | Gardé + `daemon run` | ✅ |
| `crates/client` | Gardé | ✅ |
| `territoire-graphique/` | Créé (Godot 4 + gdextension stub) | ✅ |

---

## 3. Livrables réalisés

### 3.1 Grand nettoyage

- [x] Suppression `crates/hud/`, `crates/tui/`, `orchestrator/src/tui/`
- [x] Retrait egui/ratatui du workspace `Cargo.toml`
- [x] Feature `tui` retirée de `orchestrator`
- [x] Scripts release / installer mis à jour (CLI seul)
- [x] `README.md` mis à jour

### 3.2 Architecture multi-langages

- [x] Dossier `territoire-graphique/` avec Godot 4 vierge
- [x] Crate `territoire-gdextension` (stub Phase 15)
- [x] `communication.md` — protocole WS Option B

### 3.3 Daemon backend

- [x] Feature `websocket-server` dans `orchestrator`
- [x] Module `daemon/` (axum WS, `Command`/`Response` JSON)
- [x] Section `[daemon]` dans `orchestrator.toml` (port 28790)
- [x] Commande CLI `orchestrateur daemon run`
- [x] Tests `daemon_integration`

### 3.4 Documentation

- [x] Ce document
- [x] `docs/README.md` mis à jour
- [x] Archives phases 1–14 conservées (références historiques HUD/TUI)

---

## 4. Lancement

```powershell
# Daemon Territoire Graphique
$env:ORCHESTRATEUR_DAEMON_TOKEN = "secret"
.\target\release\orchestrateur.exe daemon run --workspace workspace

# Santé
curl http://127.0.0.1:28790/health
```

---

## 5. Prochaines étapes — Phase 15

- Setup complet Godot 4 + shaders
- Première Boule de Pixels
- Connexion WebSocket Godot ↔ daemon Rust

---

**Fin du document Phase 14 bis**