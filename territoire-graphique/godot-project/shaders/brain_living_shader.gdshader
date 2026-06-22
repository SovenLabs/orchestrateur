shader_type spatial;
render_mode unshaded, cull_disabled;

uniform float time : hint_range(0.0, 100000.0) = 0.0;
uniform float activity : hint_range(0.0, 1.0) = 0.35;

// Déformation sphère + glow pulsé + bruit "pixels vivants"
void vertex() {
	float n1 = sin(VERTEX.x * 5.0 + time * 1.2) * cos(VERTEX.y * 5.0 + time * 0.9);
	float n2 = sin(VERTEX.z * 6.0 + time * 1.5 + activity * 3.0);
	float deform = (n1 + n2) * 0.5 * activity * 0.1;
	VERTEX += NORMAL * deform;
}

void fragment() {
	float pulse = 0.5 + 0.5 * sin(time * (2.5 + activity * 4.0));
	float pixel = fract(sin(dot(UV * vec2(120.0, 90.0) + time * 0.5, vec2(12.9898, 78.233))) * 43758.5453);
	vec3 deep = vec3(0.05, 0.12, 0.45);
	vec3 bright = vec3(0.25, 0.65, 1.0);
	vec3 base = mix(deep, bright, activity);
	vec3 glow = base + vec3(pulse * 0.35 * activity);
	glow += vec3(pixel * 0.2 * activity);
	ALBEDO = glow;
	EMISSION = glow * (1.0 + activity * 1.2);
}