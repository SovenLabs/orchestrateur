// brain_living_shader.wgsl
// Phase 15 - Boule de Pixels Vivante améliorée

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> activity: f32; // 0.0 à 2.0

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    
    // Déformation légère basée sur l'activité
    let deform = sin(time * 3.0 + position.x * 5.0) * activity * 0.08;
    let deformed_pos = position + normalize(position) * deform;
    
    output.position = vec4<f32>(deformed_pos, 1.0);
    output.uv = uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;
    
    // Effet "pixels vivants" (bruit + pulsation)
    let noise = fract(sin(dot(uv, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let pulse = sin(time * 2.5) * 0.5 + 0.5;
    
    // Glow + couleur basée sur l'activité
    let base_color = vec3<f32>(0.15, 0.55, 0.95);
    let glow = activity * pulse * 0.6;
    
    let final_color = base_color + vec3<f32>(glow * 0.4, glow * 0.3, glow * 0.8);
    
    // Ajout de bruit pour effet pixel
    let pixel_effect = mix(0.85, 1.15, noise);
    
    return vec4<f32>(final_color * pixel_effect, 1.0);
}