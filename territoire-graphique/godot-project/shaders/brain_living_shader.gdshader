shader_type spatial;
render_mode unshaded, cull_disabled;

uniform float time : hint_range(0.0, 100000.0) = 0.0;
uniform float activity : hint_range(0.0, 1.0) = 0.35;
uniform float stress : hint_range(0.0, 1.0) = 0.0;
uniform float swirl : hint_range(0.0, 1.0) = 0.0;
uniform float living : hint_range(0.0, 1.0) = 1.0;
uniform float refraction_strength : hint_range(0.0, 0.15) = 0.04;
uniform float plexus_strength : hint_range(0.0, 2.0) = 0.85;

// Inspiré Plexus (FencerDevLog) + composite procédural — surface organique vivante
float hash31(vec3 p) {
	p = fract(p * 0.3183099 + vec3(0.1, 0.2, 0.3));
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
	for (int i = 0; i < 4; i++) {
		v += a * noise3(p);
		p *= 2.1;
		a *= 0.5;
	}
	return v;
}

// Réseau plexus procédural sur la sphère (filaments + nœuds)
float plexus_filaments(vec3 dir, float t) {
	vec3 flow = dir * 3.5 + vec3(t * 0.12, t * 0.09, t * 0.11);
	float field = fbm(flow);
	float lines_a = smoothstep(0.52, 0.54, field) + smoothstep(0.68, 0.695, field);
	float lines_b = smoothstep(0.38, 0.395, fbm(flow * 1.7 + vec3(2.1, 0.4, 1.8)));
	float pulse = 0.65 + 0.35 * sin(t * 2.8 + field * 9.0);
	return (lines_a + lines_b * 0.6) * pulse;
}

void vertex() {
	vec3 p = VERTEX;
	vec3 dir = normalize(VERTEX);
	float t = time;

	float organic = fbm(dir * 2.8 + vec3(t * 0.35, t * 0.28, t * 0.22));
	float ripple = sin(p.x * 5.0 + t * 1.35) * cos(p.y * 4.6 + t * 0.95);
	float living_wave = living * sin(t * 2.2 + p.y * 9.0 + p.x * 4.0) * 0.02;
	float swirl_warp = swirl * sin(t * 5.0 + p.y * 8.5) * 0.055;
	float displacement = organic * activity * 0.14 + ripple * activity * 0.04 + living_wave + swirl_warp;
	VERTEX += NORMAL * displacement;
}

void fragment() {
	vec3 dir = normalize(VERTEX);
	float t = time;
	float fresnel = pow(1.0 - abs(dot(NORMAL, VIEW)), 3.2);
	float pulse = 0.5 + 0.5 * sin(t * (2.5 + activity * 6.0));
	float breathe = 0.5 + 0.5 * sin(t * 1.4 + activity * 2.0);

	vec2 refract_uv = UV + NORMAL.xy * refraction_strength * (0.6 + activity * 0.8) * sin(t * 3.5 + UV.x * 12.0);
	float pixel1 = fract(sin(dot(refract_uv * vec2(150.0, 110.0) + t * 0.6, vec2(12.9898, 78.233))) * 43758.5453);
	float pixel2 = fract(sin(dot(refract_uv * vec2(85.0, 65.0) - t * 0.75, vec2(39.3468, 11.135))) * 22578.1459);
	float pixels = (pixel1 + pixel2) * 0.5;

	float veins = plexus_filaments(dir, t) * plexus_strength * (0.35 + activity * 0.9);

	vec3 cold = vec3(0.02, 0.07, 0.36);
	vec3 warm = vec3(0.22, 0.58, 1.0);
	vec3 hot = vec3(0.5, 0.9, 1.0);
	vec3 vein_col = vec3(0.35, 0.78, 1.0);
	vec3 degraded_col = vec3(0.45, 0.38, 0.22);
	vec3 error_col = vec3(0.88, 0.2, 0.12);

	vec3 base = mix(cold, warm, activity);
	base = mix(base, hot, activity * activity * pulse);
	base = mix(base, error_col, stress * 0.75);
	base = mix(base, degraded_col, stress * 0.35 * (1.0 - activity));
	base += vein_col * veins * (0.5 + swirl * 0.4);

	float glow_power = 1.15 + activity * 2.4 + fresnel * (0.85 + activity * 1.2) + breathe * activity * 0.4;
	vec3 edge = vec3(0.2, 0.55, 1.0) * fresnel * (0.75 + activity * 1.15);
	vec3 chroma = vec3(0.08, 0.14, 0.22) * fresnel * swirl;
	vec3 core_glow = base * glow_power;
	vec3 pixel_glow = vec3(pixels * 0.34 * (0.4 + activity));

	vec3 glow = core_glow + edge + pixel_glow + chroma;
	ALBEDO = glow;
	EMISSION = glow * (1.45 + activity * 2.0 + fresnel * 0.75 + swirl * 0.3 + veins * 0.5);
}