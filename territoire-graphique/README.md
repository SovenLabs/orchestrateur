# Territoire Graphique — Phase 15 PoC

Client Godot 4 **Boule de Pixels Vivante** connecté au daemon Rust via WebSocket (Option B).

## Lancement rapide

```powershell
# 1. Daemon
$env:ORCHESTRATEUR_DAEMON_TOKEN = "dev"
.\target\release\orchestrateur.exe daemon run --workspace workspace

# 2. Godot 4.3+ — ouvrir godot-project/ et Play (F5)
```

## Contenu Phase 15

| Composant | Chemin |
|-----------|--------|
| Shader vivant | `godot-project/shaders/brain_living_shader.gdshader` |
| Boule 3D | `godot-project/scenes/brain_sphere.tscn` |
| WebSocket client | `godot-project/scripts/daemon_client.gd` |
| Monitoring | `godot-project/scenes/monitoring_panel.tscn` |
| Protocole | [`communication.md`](communication.md) |
| Archive phase | [`../docs/Phase_15_Territoire_Graphique_PoC.md`](../docs/Phase_15_Territoire_Graphique_PoC.md) |

## Token

Par défaut le client utilise le token `dev`. En production, définir la même valeur des deux côtés :

```powershell
$env:ORCHESTRATEUR_DAEMON_TOKEN = "votre-secret"
```

## GDExtension (optionnel)

```powershell
cargo build -p territoire-gdextension --release
```

Puis activer `territoire_gdextension.gdextension` dans le projet Godot.