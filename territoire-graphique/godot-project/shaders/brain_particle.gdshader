shader_type spatial;
render_mode blend_add, unshaded, cull_disabled, depth_draw_never, transparency_alpha;

uniform vec4 particle_color : source_color = vec4(0.35, 0.72, 1.0, 0.85);
uniform float softness : hint_range(0.0, 1.0) = 0.65;

// Particules additives douces (Godotneers-style glow particles)
void fragment() {
	vec2 uv = UV * 2.0 - 1.0;
	float dist = length(uv);
	float core = smoothstep(1.0, 0.0, dist);
	float halo = smoothstep(1.0, softness, dist);
	float alpha = core * halo * particle_color.a;
	vec3 col = particle_color.rgb * (core + halo * 0.35);
	ALBEDO = col;
	ALPHA = alpha;
	EMISSION = col * 1.6;
}