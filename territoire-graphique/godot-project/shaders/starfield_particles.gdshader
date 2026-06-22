shader_type spatial;
render_mode blend_add, unshaded, depth_draw_never, cull_disabled;

uniform float time : hint_range(0.0, 100000.0) = 0.0;

void fragment() {
	vec2 uv = UV * 2.0 - 1.0;
	float dist = length(uv);
	float core = smoothstep(1.0, 0.0, dist);
	float tw = 0.55 + 0.45 * sin(time * 1.8 + (UV.x + UV.y) * 12.0);
	vec3 col = vec3(0.55, 0.78, 1.0) * core * tw;
	ALBEDO = col;
	EMISSION = col * 1.4;
}