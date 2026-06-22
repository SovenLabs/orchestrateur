shader_type spatial;
render_mode blend_add, unshaded, depth_draw_never, cull_disabled;

uniform float time : hint_range(0.0, 100000.0) = 0.0;
uniform float network_activity : hint_range(0.0, 2.0) = 1.0;
uniform float flow_speed : hint_range(0.0, 8.0) = 2.4;
uniform vec3 cyan_color = vec3(0.0, 0.87, 1.0);
uniform vec3 orange_color = vec3(1.0, 0.73, 0.27);

varying vec4 v_custom;
varying float v_along;

void vertex() {
	v_custom = INSTANCE_CUSTOM;
	v_along = (VERTEX.z + 0.5);
}

void fragment() {
	float along = clamp(v_along, 0.0, 1.0);
	float flow = fract(along * 3.0 - time * flow_speed * 0.15);
	float packet = smoothstep(0.88, 0.92, flow) * smoothstep(0.02, 0.0, abs(flow - 0.95));
	float edge_fade = 1.0 - abs(VERTEX.y) * 2.0;

	float spike = v_custom.a;
	float impulse = smoothstep(0.0, 0.1, spike) * sin((along - spike) * 20.0 - time * 10.0);
	impulse = max(impulse, 0.0);

	vec3 col = mix(cyan_color, orange_color, packet * 0.7 + impulse * 0.35);
	float alpha = edge_fade * (0.3 + network_activity * 0.5);
	alpha += packet * 0.9 + impulse * 1.1 + spike * 0.7;

	ALBEDO = col;
	ALPHA = clamp(alpha, 0.0, 2.0);
	EMISSION = col * (1.3 + packet * 2.2 + impulse * 2.8 + spike * 2.0) * network_activity;
}