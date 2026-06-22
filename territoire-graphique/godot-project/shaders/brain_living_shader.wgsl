// brain_living_shader.wgsl - Phase 16
// Version améliorée avec plus d'effets

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
};

@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> activity: f32;

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    
    // Déformation plus prononcée
    let wave = sin(time * 2.0 + position.x * 4.0 + position.y * 3.0) * activity * 0.12;
    let deform = normalize(position) * wave;
    
    let deformed = position + deform;
    
    output.position = vec4<f32>(deformed, 1.0);
    output.uv = uv;
    output.world_pos = deformed;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;
    let pos = input.world_pos;
    
    // Bruit pour effet pixels
    let n1 = fract(sin(dot(uv, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let n2 = fract(sin(dot(pos.xy, vec2<f32>(78.233, 12.9898))) * 43758.5453);
    
    let pulse = sin(time * 3.0) * 0.5 + 0.5;
    let activity_pulse = activity * pulse;
    
    // Couleur de base + glow
    var color = vec3<f32>(0.12, 0.48, 0.92);
    
    // Ajout de glow sur les bords
    let edge = 1.0 - length(uv - 0.5) * 1.8;
    color += vec3<f32>(0.3, 0.5, 1.0) * edge * activity_pulse * 0.6;
    
    // Effet pixels + variation
    let pixel = mix(0.9, 1.15, n1);
    color *= pixel;
    
    // Légère variation selon la position
    color += vec3<f32>(n2 * 0.08);
    
    return vec4<f32>(color, 1.0);
}