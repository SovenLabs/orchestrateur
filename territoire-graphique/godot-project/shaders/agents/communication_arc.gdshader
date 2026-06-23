shader_type spatial;
render_mode blend_add, unshaded, depth_draw_never, cull_disabled;

uniform float time : hint_range(0.0, 100000.0) = 0.0;
uniform float pulse_pos : hint_range(0.0, 1.0) = 0.0;
uniform float alpha_mul : hint_range(0.0, 1.0) = 1.0;
uniform float arc_seed : hint_range(0.0, 100.0) = 0.0;
uniform float arrival_flash : hint_range(0.0, 1.0) = 0.0;
uniform vec3 cyan_color = vec3(0.0, 0.87, 1.0);
uniform vec3 violet_color = vec3(0.73, 0.53, 1.0);
uniform vec3 pulse_color = vec3(1.0, 0.73, 0.27);

void fragment() {
	float along = UV.x;
	float across = 1.0 - abs(UV.y - 0.5) * 2.0;

	// Double ruban : coeur fin + halo large
	float core = smoothstep(0.55, 0.0, across);
	float halo = smoothstep(0.0, 0.35, across) * (1.0 - core) * 0.55;
	float ribbon = core + halo;

	// Flux synaptique continu (paquets multiples)
	float flow_phase = arc_seed * 0.17;
	float flow_a = fract(along * 4.5 - time * 2.8 + flow_phase);
	float flow_b = fract(along * 3.0 - time * 1.6 + flow_phase * 1.3);
	float packet_a = smoothstep(0.9, 0.94, flow_a) * smoothstep(0.02, 0.0, abs(flow_a - 0.97));
	float packet_b = smoothstep(0.88, 0.91, flow_b) * smoothstep(0.02, 0.0, abs(flow_b - 0.93)) * 0.65;
	float flow_packets = packet_a + packet_b;

	// Impulsion principale voyageant le long de l'arc
	float dist = along - pulse_pos;
	float pulse = smoothstep(0.16, 0.0, abs(dist));
	float trail = smoothstep(0.42, 0.0, max(dist, 0.0)) * 0.55;
	float wake = smoothstep(0.08, 0.0, abs(dist)) * sin(dist * 48.0 - time * 14.0);
	wake = max(wake, 0.0) * 0.45;

	// Onde d'impulsion synaptique
	float impulse = smoothstep(0.0, 0.12, pulse_pos) * sin((along - pulse_pos) * 28.0 - time * 11.0);
	impulse = max(impulse, 0.0) * smoothstep(0.0, 0.2, ribbon);

	// Shimmer + données binaires
	float shimmer = 0.12 + 0.08 * sin(along * 22.0 - time * 7.0 + arc_seed);
	float data_bits = step(0.55, fract(along * 36.0 - time * 9.0 + arc_seed * 0.31));

	vec3 base_col = mix(cyan_color, violet_color, sin(along * 3.14159 + arc_seed) * 0.5 + 0.5);
	vec3 col = mix(base_col, pulse_color, (pulse * 0.8 + flow_packets * 0.5 + impulse * 0.35));
	col += pulse_color * arrival_flash * smoothstep(0.92, 1.0, along) * 0.6;

	float energy = pulse + trail + flow_packets + impulse + wake;
	float alpha = ribbon * (0.18 + shimmer + data_bits * 0.06) * alpha_mul;
	alpha += energy * ribbon * alpha_mul * 0.85;
	alpha += arrival_flash * smoothstep(0.85, 1.0, along) * ribbon * 0.7;

	ALBEDO = col;
	ALPHA = clamp(alpha, 0.0, 1.8);
	EMISSION = col * (0.9 + energy * 3.2 + arrival_flash * 2.5) * (0.85 + core * 0.4);
}