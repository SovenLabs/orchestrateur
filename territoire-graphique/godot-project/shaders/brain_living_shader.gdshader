shader_type spatial;
render_mode unshaded, cull_disabled;

uniform float time : hint_range(0.0, 100000.0) = 0.0;
uniform float activity : hint_range(0.0, 1.0) = 0.35;

// v2 — déformation organique triple + fresnel + double bruit pixels
void vertex() {
	vec3 p = VERTEX;
	float n1 = sin(p.x * 4.2 + time * 1.1) * cos(p.y * 4.2 + time * 0.85);
	float n2 = sin(p.z * 5.5 + time * 1.4 + activity * 3.5) * sin(p.x * 3.2 + activity * 2.0);
	float n3 = cos(p.y * 6.2 + time * 0.55 + activity * 1.8) * cos(p.z * 2.8 + time);
	float organic = (n1 + n2 + n3) / 3.0;
	VERTEX += NORMAL * organic * activity * 0.12;
}

void fragment() {
	float fresnel = pow(1.0 - abs(dot(NORMAL, VIEW)), 2.8);
	float pulse = 0.5 + 0.5 * sin(time * (3.0 + activity * 5.0));
	float pixel1 = fract(sin(dot(UV * vec2(150.0, 110.0) + time * 0.6, vec2(12.9898, 78.233))) * 43758.5453);
	float pixel2 = fract(sin(dot(UV * vec2(85.0, 65.0) - time * 0.75, vec2(39.3468, 11.135))) * 22578.1459);
	float pixels = (pixel1 + pixel2) * 0.5;

	vec3 cold = vec3(0.03, 0.08, 0.38);
	vec3 warm = vec3(0.28, 0.62, 1.0);
	vec3 hot = vec3(0.55, 0.88, 1.0);
	vec3 base = mix(cold, warm, activity);
	base = mix(base, hot, activity * activity * pulse);

	vec3 edge = vec3(0.25, 0.55, 1.0) * fresnel * (0.65 + activity * 0.9);
	vec3 glow = base + edge + vec3(pixels * 0.28 * activity);
	ALBEDO = glow;
	EMISSION = glow * (1.25 + activity * 1.6 + fresnel * 0.55);
}