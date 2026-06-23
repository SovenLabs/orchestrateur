shader_type spatial;
render_mode blend_add, unshaded, depth_draw_never, cull_disabled;

uniform float alpha_mul : hint_range(0.0, 1.0) = 1.0;
uniform float ring_progress : hint_range(0.0, 1.0) = 0.0;
uniform vec3 burst_color = vec3(0.0, 0.87, 1.0);

void fragment() {
	vec2 uv = UV - vec2(0.5);
	float d = length(uv);
	float radius = 0.12 + ring_progress * 0.42;
	float ring = smoothstep(radius + 0.06, radius, d) * smoothstep(radius - 0.04, radius, d);
	float fill = smoothstep(radius, 0.0, d) * (1.0 - ring_progress) * 0.25;

	vec3 col = burst_color;
	float alpha = (ring + fill) * alpha_mul * (1.0 - ring_progress * 0.85);

	ALBEDO = col;
	ALPHA = clamp(alpha, 0.0, 1.2);
	EMISSION = col * (1.5 + ring * 3.0);
}