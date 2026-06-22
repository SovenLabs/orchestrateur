# Référence Godot — Territoire Graphique

**Moteur cible :** Godot **4.7.stable** (Forward+)  
**Documentation officielle :** [https://docs.godotengine.org/en/stable/](https://docs.godotengine.org/en/stable/)  
**Branche doc :** `stable` (pas `latest`, pas `4.4` isolé) — toujours vérifier l’URL `/en/stable/`.

> Règle projet : avant d’introduire une API, un type packed ou un `render_mode` shader, croiser avec la doc **stable** et tester dans l’éditeur 4.7.

---

## 1. Index doc stable → usage Territoire Graphique

| Domaine | Page stable | Utilisation projet |
|---------|-------------|------------------|
| **GDScript** | [GDScript reference](https://docs.godotengine.org/en/stable/classes/class_@gdscript.html) | Scripts `scripts/`, `@tool`, typage |
| **Types packed** | [PackedVector3Array](https://docs.godotengine.org/en/stable/classes/class_packedvector3array.html) | Positions neurones, meshes |
| | [PackedVector2Array](https://docs.godotengine.org/en/stable/classes/class_packedvector2array.html) | **Float** — pas pour paires d’indices entiers |
| | [Vector2i](https://docs.godotengine.org/en/stable/classes/class_vector2i.html) | Arêtes graphe dans `Array` |
| **Scènes / nodes** | [Node](https://docs.godotengine.org/en/stable/classes/class_node.html) | Autoloads, hiérarchie |
| | [Window](https://docs.godotengine.org/en/stable/classes/class_window.html) | Multi-fenêtrage `WindowManager` |
| **UI responsive** | [Control anchors](https://docs.godotengine.org/en/stable/tutorials/ui/size_and_anchors.html) | Colonnes dock gauche/droite |
| | [SplitContainer](https://docs.godotengine.org/en/stable/classes/class_splitcontainer.html) | `VSplitContainer` panneaux |
| | [Viewport scaling](https://docs.godotengine.org/en/stable/tutorials/rendering/multiple_resolutions.html) | 1920×1080, `stretch/aspect=expand` |
| **Réseau** | [WebSocketPeer](https://docs.godotengine.org/en/stable/classes/class_websocketpeer.html) | `TerritoryDaemonClient` |
| | [HTTPRequest](https://docs.godotengine.org/en/stable/classes/class_httprequest.html) | Fallback health HTTP |
| **3D rendu** | [MeshInstance3D](https://docs.godotengine.org/en/stable/classes/class_meshinstance3d.html) | Noyau plasma |
| | [MultiMeshInstance3D](https://docs.godotengine.org/en/stable/classes/class_multimeshinstance3d.html) | Neurones + synapses AI Brain |
| | [Camera3D](https://docs.godotengine.org/en/stable/classes/class_camera3d.html) | Orbite `NeuralBrainCamera` |
| | [WorldEnvironment](https://docs.godotengine.org/en/stable/classes/class_worldenvironment.html) | Post-FX glow |
| | [Environment](https://docs.godotengine.org/en/stable/classes/class_environment.html) | Bloom, tonemap, sky |
| **Particules** | [GPUParticles3D](https://docs.godotengine.org/en/stable/classes/class_gpuparticles3d.html) | `BrainSphere` idle/assim/tool |
| | [ParticleProcessMaterial](https://docs.godotengine.org/en/stable/classes/class_particleprocessmaterial.html) | `radial_accel`, `orbit_velocity` |
| | [3D particles tutorial](https://docs.godotengine.org/en/stable/tutorials/3d/particles/index.html) | Bonnes pratiques perf |
| **Shaders** | [Shading language](https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/index.html) | Tous `.gdshader` |
| | [Spatial shaders](https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/spatial_shader.html) | Boule, particules, plasma |
| | [Sky shaders](https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/sky_shader.html) | `starfield_background.gdshader` |
| | [Shader functions](https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/shader_functions.html) | `mix`, `smoothstep`, `fract` (**GLSL**, pas GDScript) |
| | [Render modes](https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/spatial_shader.html#render-modes) | `blend_add`, `unshaded`, etc. |
| **MultiMesh** | [MultiMesh](https://docs.godotengine.org/en/stable/classes/class_multimesh.html) | `use_custom_data`, `INSTANCE_CUSTOM` |
| **Input** | [InputEvent](https://docs.godotengine.org/en/stable/classes/class_inputevent.html) | Caméra orbite souris |

---

## 2. Pièges validés sur Godot 4.7.stable (projet réel)

### GDScript

| ❌ Éviter | ✅ Utiliser (stable / 4.7) |
|----------|---------------------------|
| `PackedVector2iArray` | `Array` de `Vector2i` — pas de packed dédié documenté pour Vector2i |
| `fract(x)` en GDScript | `x - floor(x)` ou `fmodf(x, 1.0)` |
| `env.glow_levels/1 = true` | `env.set("glow_levels/1", true)` — slash réservé à l’inspecteur |
| `class_name` = nom autoload | Éviter conflit (`VisualEventMapper` autoload sans `class_name`) |
| Inférence `var e := _edges[i]` sur `Array` | `var e: Vector2i = _edges[i]` |

### Shaders (`spatial`)

| ❌ Éviter | ✅ Utiliser |
|----------|-------------|
| `render_mode … transparency_alpha` (4.7) | `blend_add` + `ALPHA` dans le fragment si besoin |
| `fract()` en GDScript | En **GLSL shader** : `fract()` est valide ([shader functions](https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/shader_functions.html)) |

### GPUParticles3D

| Règle doc / runtime | Application |
|---------------------|-------------|
| `amount >= 1` ([GPUParticles3D](https://docs.godotengine.org/en/stable/classes/class_gpuparticles3d.html)) | Ne jamais mettre `amount = 0` — couper avec `emitting = false` |
| `GPUParticlesAttractor3D` | Non instanciable sur certaines builds → préférer `radial_accel` négatif |

### Environment / Glow

Réglages utilisés (`neural_brain_environment.gd`, `brain_visual_fx.gd`) — voir [Environment glow](https://docs.godotengine.org/en/stable/classes/class_environment.html#class-environment-property-glow-enabled) :

```gdscript
environment.glow_enabled = true
environment.glow_intensity = 0.82      # 0.7–0.85 cinématique
environment.glow_strength = 1.15
environment.glow_bloom = 0.38
environment.glow_blend_mode = Environment.GLOW_BLEND_MODE_SCREEN
environment.glow_hdr_threshold = 0.55  # threshold bas = plus de bloom
environment.glow_hdr_scale = 2.4
environment.set("glow_levels/1", true)   # … /2, /3, /4
```

### Display (project.godot)

Aligné [Multiple resolutions](https://docs.godotengine.org/en/stable/tutorials/rendering/multiple_resolutions.html) :

```ini
window/size/viewport_width=1920
window/size/viewport_height=1080
window/stretch/mode="canvas_items"
window/stretch/aspect="expand"
```

---

## 3. Fichiers projet ↔ APIs doc

| Module | Fichiers | Classes doc stable |
|--------|----------|-------------------|
| Scène principale | `main_scene.gd` | `MainScene`, `Environment`, starfield |
| Client WS | `daemon_client.gd` | `WebSocketPeer`, `HTTPRequest` |
| Layout | `dock_layout.gd` | `Control`, `SplitContainer`, `Viewport` |
| Boule Phase 20 | `brain_sphere.gd`, `brain_*.gdshader` | `GPUParticles3D`, `ShaderMaterial` |
| AI Neural Brain | `ai_neural_brain/*.gd`, `shaders/ai_neural_brain/*` | `MultiMeshInstance3D`, `Sky`, `Environment` |
| Post-FX | `brain_visual_fx.gd`, `neural_brain_environment.gd` | `WorldEnvironment`, `Environment` |

---

## 4. Workflow développeur / IA

1. **Lire** la page `stable` de la classe concernée (lien ci-dessus).
2. **Vérifier** les propriétés dans l’inspecteur Godot 4.7 (certaines pages stable couvrent 4.x générique).
3. **Ne pas copier** des snippets Godot 3.x (`spatial` legacy, `yield`, `export(int)` sans typage).
4. **Tester** F5 après tout changement shader — erreurs `render_mode` apparaissent au chargement.
5. **Préférer** `MultiMesh` pour >1000 instances ([Using MultiMesh](https://docs.godotengine.org/en/stable/tutorials/performance/using_multimesh.html)).

---

## 5. Versions doc Godot (ne pas mélanger)

| URL | Quand l’utiliser |
|-----|------------------|
| `/en/stable/` | **Référence projet** — Godot 4.x stable actuel |
| `/en/latest/` | Branche dev — peut diverger de 4.7.stable |
| `/en/4.4/` | Archive version — utile si régression spécifique 4.4 |

**Notre binaire :** `4.7.stable.official` → doc **`stable`** + tests locaux font foi en cas d’écart.

---

## 6. Changelog compatibilité (sessions Territoire Graphique)

| Date | Problème | Résolution | Doc |
|------|----------|------------|-----|
| 2026-06 | `amount = 0` particules | `MIN_GPU_AMOUNT = 1` + `emitting=false` | [GPUParticles3D](https://docs.godotengine.org/en/stable/classes/class_gpuparticles3d.html) |
| 2026-06 | `glow_levels/1 =` parse error | `set("glow_levels/1", true)` | [Environment](https://docs.godotengine.org/en/stable/classes/class_environment.html) |
| 2026-06 | `GPUParticlesAttractor3D` null | Retiré, `radial_accel` | [ParticleProcessMaterial](https://docs.godotengine.org/en/stable/classes/class_particleprocessmaterial.html) |
| 2026-06 | `transparency_alpha` shader | Retiré du `render_mode` | [Spatial render modes](https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/spatial_shader.html#render-modes) |
| 2026-06 | `PackedVector2iArray` | `Array` + `Vector2i` | [Vector2i](https://docs.godotengine.org/en/stable/classes/class_vector2i.html) |
| 2026-06 | `fract()` GDScript | `v - floor(v)` | [GDScript built-ins](https://docs.godotengine.org/en/stable/classes/class_@gdscript.html) |

---

*Maintenir ce fichier à chaque correction liée à une incompatibilité Godot 4.7 / doc stable.*