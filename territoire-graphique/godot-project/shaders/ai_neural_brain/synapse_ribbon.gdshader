shader_type spatial;
render_mode blend_add, unshaded, depth_draw_never, cull_disabled;

uniform float time : hint_range(0.0, 100000.0) = 0.0;
uniform float network_activity : hint_range(0.0, 2.0) = 1.0;
uniform float flow_speed : hint_range(0.0, 8.0) = 2.4;
uniform vec3 cyan_color = vec3(0.0, 0.87, 1.0);
uniform vec3 orange_color = vec3(1.0, 0.73, 0.27);

void fragment() {
	// UV.x = 0..1 le long de la connexion, UV.y = bord du ruban
	float along = UV.x;
	float flow = fract(along * 3.0 - time * flow_speed * 0.15);
	float packet = smoothstep(0.88, 0.92, flow) * smoothstep(0.02, 0.0, abs(flow - 0.95));
	float base = 0.25 + 0.75 * (1.0 - abs(UV.y - 0.5) * 2.0);

	float spike = COLOR.a;
	float impulse = smoothstep(0.0, 0.08, spike) * sin((along - spike) * 24.0 - time * 12.0);
	impulse = max(impulse, 0.0) * step(0.0, along - spike + 0.05);

	vec3 col = mix(cyan_color, orange_color, packet * 0.65 + impulse * 0.4);
	float alpha = base * (0.35 + network_activity * 0.55);
	alpha += packet * 0.8 + impulse * 1.2 + spike * 0.6;

	ALBEDO = col;
	ALPHA = clamp(alpha, 0.0, 2.0);
	EMISSION = col * (1.2 + packet * 2.5 + impulse * 3.0 + spike * 2.0) * network_activity;
}