shader_type sky;

uniform vec3 color_top : source_color = vec3(0.02, 0.02, 0.06);
uniform vec3 color_horizon : source_color = vec3(0.01, 0.01, 0.02);
uniform float time : hint_range(0.0, 100000.0) = 0.0;
uniform float star_density : hint_range(0.0, 1.0) = 0.35;

float hash21(vec2 p) {
	return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

void sky() {
	float h = EYEDIR.y * 0.5 + 0.5;
	vec3 bg = mix(color_horizon, color_top, pow(h, 1.4));

	vec2 uv = EYEDIR.xz / (abs(EYEDIR.y) + 0.35);
	float stars = 0.0;
	for (int i = 0; i < 3; i++) {
		float fi = float(i);
		vec2 g = floor(uv * (40.0 + fi * 17.0));
		float n = hash21(g + fi * 19.0);
		if (n > 1.0 - star_density * 0.02) {
			float tw = 0.6 + 0.4 * sin(time * 1.5 + n * 40.0);
			stars += tw * smoothstep(0.995, 1.0, n);
		}
	}

	COLOR = bg + vec3(0.55, 0.7, 1.0) * stars * 0.35;
}