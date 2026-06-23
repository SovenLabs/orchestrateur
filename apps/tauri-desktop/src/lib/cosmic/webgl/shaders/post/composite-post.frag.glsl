#version 300 es
precision highp float;

in vec2 v_uv;
out vec4 fragColor;

uniform sampler2D u_scene;
uniform sampler2D u_bloom;
uniform float u_bloomIntensity;
uniform float u_time;
uniform float u_activity;
uniform vec2 u_resolution;
uniform int u_fx_mask; // 1=aces 2=aberration 4=grain 8=vignette

const float PI = 3.14159265359;

float hash21(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

vec3 acesFilm(vec3 x) {
  return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), 0.0, 1.0);
}

vec3 sampleScene(vec2 uv) {
  vec3 scene = texture(u_scene, uv).rgb;
  vec3 bloom = texture(u_bloom, uv).rgb;
  return scene + bloom * u_bloomIntensity;
}

void main() {
  vec2 uv = v_uv;
  vec3 col;

  if ((u_fx_mask & 2) != 0) {
    vec2 center = uv - 0.5;
    float dist = length(center);
    float aberr = 0.0006 + dist * 0.0012;
    col.r = sampleScene(uv + center * aberr).r;
    col.g = sampleScene(uv).g;
    col.b = sampleScene(uv - center * aberr).b;
  } else {
    col = sampleScene(uv);
  }

  if ((u_fx_mask & 1) != 0) {
    float exposure = 1.05 + u_activity * 0.08;
    col = acesFilm(col * exposure);
    col = pow(col, vec3(0.92));
  } else {
    col = clamp(col, 0.0, 1.0);
  }

  if ((u_fx_mask & 4) != 0) {
    float grain = (hash21(uv * u_resolution + u_time * 17.3) - 0.5) * 0.035;
    col += grain;
  }

  if ((u_fx_mask & 8) != 0) {
    vec2 vigUv = (uv - 0.5) * vec2(0.92, 1.0);
    float vig = smoothstep(1.4, 0.15, length(vigUv));
    col *= mix(0.5, 1.0, vig);
  }

  fragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}