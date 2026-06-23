#version 300 es
precision highp float;

in vec2 v_local;
in vec3 v_tint;
in float v_nebula;
in float v_radius;

out vec4 fragColor;

uniform float u_time;

float hash21(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

void main() {
  float d = length(v_local);
  if (d > 1.0) discard;

  float core = exp(-d * d * 3.5);
  float arms = 0.0;
  if (v_nebula > 0.5) {
    float angle = atan(v_local.y, v_local.x);
    arms = 0.25 + 0.75 * abs(sin(angle * 2.0 + u_time * 0.15));
  } else {
    float angle = atan(v_local.y, v_local.x);
    float spiral = angle + log(max(d, 0.05)) * 2.8 + u_time * 0.2;
    arms = 0.35 + 0.65 * abs(sin(spiral * 3.0));
    arms *= smoothstep(1.0, 0.15, d);
  }

  float halo = exp(-d * 1.8) * arms;
  vec3 col = mix(v_tint * 0.4, vec3(1.0, 0.97, 0.92), core * 0.7);
  col *= halo * (v_nebula > 0.5 ? 0.9 : 1.35);
  float alpha = halo * (0.35 + hash21(v_local * 17.0) * 0.1);
  fragColor = vec4(col * 1.8, alpha);
}