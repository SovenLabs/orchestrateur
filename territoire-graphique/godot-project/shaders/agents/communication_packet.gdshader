shader_type spatial;
render_mode blend_add, unshaded, depth_draw_never, cull_disabled;

uniform float time : hint_range(0.0, 100000.0) = 0.0;
uniform float alpha_mul : hint_range(0.0, 1.0) = 1.0;
uniform float tail_stretch : hint_range(0.0, 2.0) = 1.0;
uniform vec3 core_color = vec3(1.0, 0.92, 0.55);
uniform vec3 halo_color = vec3(0.35, 0.88, 1.0);

void fragment() {
	vec2 uv = UV - vec2(0.5);
	uv.x *= 1.0 + tail_stretch * 0.35;
	float d = length(uv);

	float core = smoothstep(0.48, 0.0, d);
	float ring = smoothstep(0.42, 0.36, d) * smoothstep(0.3, 0.36, d);
	float halo = smoothstep(0.5, 0.1, d) * 0.35;
	float flicker = 0.85 + 0.15 * sin(time * 18.0 + d * 12.0);

	vec3 col = mix(halo_color, core_color, core + ring * 0.6);
	float alpha = (core * 1.1 + ring * 0.7 + halo) * alpha_mul * flicker;

	ALBEDO = col;
	ALPHA = clamp(alpha, 0.0, 1.5);
	EMISSION = col * (2.5 + core * 5.0 + ring * 2.0);
}