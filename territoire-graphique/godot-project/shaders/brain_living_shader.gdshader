shader_type spatial;
render_mode unshaded, cull_disabled;

uniform float time : hint_range(0.0, 100000.0) = 0.0;
uniform float activity : hint_range(0.0, 1.0) = 0.35;
uniform float stress : hint_range(0.0, 1.0) = 0.0;
uniform float swirl : hint_range(0.0, 1.0) = 0.0;
uniform float living : hint_range(0.0, 1.0) = 1.0;
uniform float refraction_strength : hint_range(0.0, 0.15) = 0.04;

// vFinal — surface vivante, réfraction légère, glow réactif à l'activité
void vertex() {
	vec3 p = VERTEX;
	float ripple1 = sin(p.x * 5.0 + time * 1.35) * cos(p.y * 4.6 + time * 0.95);
	float ripple2 = sin(p.z * 6.2 + time * 1.1 + activity * 4.0) * sin(p.x * 3.4 + activity * 2.2);
	float ripple3 = cos(p.y * 7.0 + time * 0.65) * cos(p.z * 3.1 + time * 0.8);
	float organic = (ripple1 + ripple2 + ripple3) / 3.0;
	float living_wave = living * sin(time * 2.2 + p.y * 9.0 + p.x * 4.0) * 0.018;
	float swirl_warp = swirl * sin(time * 5.0 + p.y * 8.5) * 0.05;
	VERTEX += NORMAL * (organic * activity * 0.11 + living_wave + swirl_warp);
}

void fragment() {
	float fresnel = pow(1.0 - abs(dot(NORMAL, VIEW)), 3.0);
	float pulse = 0.5 + 0.5 * sin(time * (2.5 + activity * 6.0));
	float breathe = 0.5 + 0.5 * sin(time * 1.4 + activity * 2.0);

	// Réfraction simulée — décalage UV selon normale et temps
	vec2 refract_uv = UV + NORMAL.xy * refraction_strength * (0.6 + activity * 0.8) * sin(time * 3.5 + UV.x * 12.0);
	float pixel1 = fract(sin(dot(refract_uv * vec2(150.0, 110.0) + time * 0.6, vec2(12.9898, 78.233))) * 43758.5453);
	float pixel2 = fract(sin(dot(refract_uv * vec2(85.0, 65.0) - time * 0.75, vec2(39.3468, 11.135))) * 22578.1459);
	float pixels = (pixel1 + pixel2) * 0.5;

	vec3 cold = vec3(0.02, 0.07, 0.36);
	vec3 warm = vec3(0.22, 0.58, 1.0);
	vec3 hot = vec3(0.5, 0.9, 1.0);
	vec3 degraded_col = vec3(0.45, 0.38, 0.22);
	vec3 error_col = vec3(0.88, 0.2, 0.12);

	vec3 base = mix(cold, warm, activity);
	base = mix(base, hot, activity * activity * pulse);
	base = mix(base, error_col, stress * 0.75);
	base = mix(base, degraded_col, stress * 0.35 * (1.0 - activity));

	float glow_power = 1.15 + activity * 2.2 + fresnel * (0.8 + activity * 1.1) + breathe * activity * 0.35;
	vec3 edge = vec3(0.2, 0.52, 1.0) * fresnel * (0.7 + activity * 1.1);
	vec3 core_glow = base * glow_power;
	vec3 pixel_glow = vec3(pixels * 0.32 * (0.4 + activity));

	vec3 glow = core_glow + edge + pixel_glow;
	ALBEDO = glow;
	EMISSION = glow * (1.35 + activity * 1.8 + fresnel * 0.65 + swirl * 0.25);
}