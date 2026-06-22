# Phase 16 — Amélioration Shaders, Particules et WebSocket

**Date :** 22 juin 2026  
**Branche :** `main`  
**Version :** 0.17.0  
**Statut :** Terminé

---

## Objectif

Améliorer la **Boule de Pixels Vivante** et poser les fondations WebSocket côté Rust (GDExtension, Option B).

---

## Réalisations

### 1. Shader avancé (v2)

| Fichier | Rôle |
|---------|------|
| `shaders/brain_living_shader.gdshader` | Shader actif Godot 4 |
| `shaders/brain_living_shader.wgsl` | Référence spec WGSL |

Améliorations :
- Déformation organique triple bruit
- Glow fresnel sur les bords
- Double bruit procédural « pixels vivants »
- Gradient couleur cold → warm → hot selon `activity`

### 2. Particules v2 (`brain_sphere.gd`)

- `amount` dynamique 8 → 280
- `emitting` activé/désactivé selon seuil (`particle_activity_threshold = 0.15`)
- Lissage d'intensité (`smooth_speed`)
- Échelle et vitesse particules liées à l'activité

### 3. WebSocket Rust (`rust-gdextension`)

Dépendances : `tokio`, `tokio-tungstenite`, `futures-util`

```rust
pub fn start_websocket_server() -> Result<WebSocketServer, WsError>
server.spawn_daemon_client(url, token)?;  // client vers daemon :28790
```

Module `websocket.rs` — fondation Phase 17 (bidirectionnel complet).

### 4. Script principal (`main.gd`)

- Hub central `activity_changed` → boule + monitoring
- Clamping via `ActivityMapper.clamp_intensity()`
- Monitoring lissé + `set_connection_status()`

---

## Structure mise à jour

```
territoire-graphique/godot-project/
├── shaders/
│   ├── brain_living_shader.gdshader   # v2
│   └── brain_living_shader.wgsl       # référence
├── scripts/
│   ├── main.gd                        # hub Phase 16
│   ├── brain_sphere.gd                # particules v2
│   ├── monitoring_panel.gd            # barre lissée
│   └── daemon_client.gd
└── scenes/ ...

territoire-graphique/rust-gdextension/
├── src/
│   ├── lib.rs                         # start_websocket_server()
│   ├── websocket.rs                   # client daemon
│   └── activity.rs
└── Cargo.toml
```

---

## Lancement

```powershell
$env:ORCHESTRATEUR_DAEMON_TOKEN = "dev"
.\orchestrateur.exe daemon run --workspace workspace
# Godot 4.3+ → godot-project → F5
```

---

## Phase 17 (prochaine)

- GDExtension godot-rust : exposer `TerritoireBridge` à Godot
- Commandes bidirectionnelles (Chat, Search) depuis le client natif
- Territoire multi-fenêtres

---

**Fin Phase 16**