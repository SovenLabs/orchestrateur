shader_type spatial;
render_mode blend_add, unshaded, depth_draw_never, cull_disabled;

uniform float time : hint_range(0.0, 100000.0) = 0.0;
uniform float network_activity : hint_range(0.0, 2.0) = 1.0;
uniform vec3 cyan_color = vec3(0.0, 0.87, 1.0);
uniform vec3 orange_color = vec3(1.0, 0.73, 0.27);
uniform vec3 magenta_color = vec3(0.73, 0.53, 1.0);

varying vec4 v_custom;

void vertex() {
	// INSTANCE_CUSTOM: phase (r), size_scale (g), accent_id (b), spike (a)
	v_custom = INSTANCE_CUSTOM;
	float phase = v_custom.r * TAU;
	float size_scale = 0.75 + v_custom.g * 0.5;
	float spike = v_custom.a;
	float pulse = 1.0 + 0.35 * sin(time * 3.2 + phase) * network_activity;
	pulse += spike * 1.8;
	VERTEX *= size_scale * pulse;
}

void fragment() {
	float phase = v_custom.r * TAU;
	float accent = v_custom.b;
	float spike = v_custom.a;

	vec3 col = cyan_color;
	if (accent > 0.66) {
		col = mix(col, orange_color, (accent - 0.66) / 0.34);
	} else if (accent > 0.33) {
		col = mix(col, magenta_color, (accent - 0.33) / 0.33);
	}

	float flicker = 0.65 + 0.35 * sin(time * 4.5 + phase);
	float alpha = (0.55 + flicker * 0.45) * network_activity;
	alpha = clamp(alpha + spike * 0.8, 0.0, 1.5);

	ALBEDO = col * (1.0 + spike * 2.0);
	ALPHA = alpha;
	EMISSION = col * (1.8 + spike * 3.0) * network_activity;
}