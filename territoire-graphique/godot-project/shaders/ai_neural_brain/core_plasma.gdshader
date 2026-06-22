shader_type spatial;
render_mode unshaded, cull_disabled;

uniform float time : hint_range(0.0, 100000.0) = 0.0;
uniform float activity : hint_range(0.3, 2.8) = 1.0;
uniform vec3 core_color_a = vec3(0.8, 1.0, 1.0);
uniform vec3 core_color_b = vec3(0.67, 0.87, 1.0);

float hash31(vec3 p) {
	p = fract(p * 0.3183099 + vec3(0.17, 0.11, 0.23));
	p += dot(p, p.yzx + 19.19);
	return fract((p.x + p.y) * p.z);
}

float noise3(vec3 p) {
	vec3 i = floor(p);
	vec3 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	float n000 = hash31(i);
	float n100 = hash31(i + vec3(1.0, 0.0, 0.0));
	float n010 = hash31(i + vec3(0.0, 1.0, 0.0));
	float n110 = hash31(i + vec3(1.0, 1.0, 0.0));
	float n001 = hash31(i + vec3(0.0, 0.0, 1.0));
	float n101 = hash31(i + vec3(1.0, 0.0, 1.0));
	float n011 = hash31(i + vec3(0.0, 1.0, 1.0));
	float n111 = hash31(i + vec3(1.0, 1.0, 1.0));
	float nx00 = mix(n000, n100, f.x);
	float nx10 = mix(n010, n110, f.x);
	float nx01 = mix(n001, n101, f.x);
	float nx11 = mix(n011, n111, f.x);
	float nxy0 = mix(nx00, nx10, f.y);
	float nxy1 = mix(nx01, nx11, f.y);
	return mix(nxy0, nxy1, f.z);
}

float fbm(vec3 p) {
	float v = 0.0;
	float a = 0.5;
	for (int i = 0; i < 5; i++) {
		v += a * noise3(p);
		p *= 2.03;
		a *= 0.5;
	}
	return v;
}

void vertex() {
	vec3 dir = normalize(VERTEX);
	float plasma = fbm(dir * 2.4 + vec3(time * 0.55, time * 0.41, time * 0.36));
	float swirl = sin(time * 1.8 + dir.y * 6.0 + dir.x * 4.0) * 0.035 * activity;
	VERTEX += dir * (plasma * 0.08 * activity + swirl);
}

void fragment() {
	vec3 dir = normalize(VERTEX);
	float t = time;
	float fresnel = pow(1.0 - abs(dot(NORMAL, VIEW)), 3.5);
	float pulse = 0.55 + 0.45 * sin(t * (2.2 + activity * 1.4));

	vec3 flow = dir * 3.0 + vec3(t * 0.4, t * 0.32, t * 0.28);
	float plasma = fbm(flow);
	float vortex = fbm(flow * 1.8 + vec3(0.0, t * 0.6, 0.0));

	vec3 base = mix(core_color_b, core_color_a, plasma);
	base = mix(base, vec3(1.0), vortex * 0.35);
	base *= 0.85 + activity * 0.55;

	vec3 rim = vec3(0.2, 0.95, 1.0) * fresnel * (1.2 + activity * 0.6);
	vec3 glow = base * (1.4 + pulse * activity) + rim;

	ALBEDO = glow;
	EMISSION = glow * (2.5 + activity * 2.2 + fresnel * 1.8);
}