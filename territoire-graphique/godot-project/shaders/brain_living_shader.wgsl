// Référence WGSL v2 — miroir conceptuel de brain_living_shader.gdshader (Godot spatial).
// Godot 4 utilise .gdshader ; ce fichier documente la spec shader Phase 16.

struct Uniforms {
    time: f32,
    activity: f32,
}

// Déformation organique : triple bruit sur les normales
// fragment : fresnel edge glow + double hash pixel + gradient cold→hot