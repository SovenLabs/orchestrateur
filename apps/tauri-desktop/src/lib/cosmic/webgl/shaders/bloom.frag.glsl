#version 300 es
precision highp float;

in vec2 v_uv;
out vec4 fragColor;

uniform sampler2D u_source;
uniform vec2 u_resolution;
uniform vec2 u_direction;
uniform float u_threshold;
uniform float u_intensity;
uniform int u_pass; // 0 = threshold, 1 = blur horizontal, 2 = blur vertical, 3 = composite

vec3 sampleBlur(vec2 uv, vec2 dir) {
  vec2 texel = 1.0 / u_resolution;
  vec3 sum = vec3(0.0);
  sum += texture(u_source, uv + dir * texel * -4.0).rgb * 0.05;
  sum += texture(u_source, uv + dir * texel * -3.0).rgb * 0.09;
  sum += texture(u_source, uv + dir * texel * -2.0).rgb * 0.12;
  sum += texture(u_source, uv + dir * texel * -1.0).rgb * 0.15;
  sum += texture(u_source, uv).rgb * 0.16;
  sum += texture(u_source, uv + dir * texel * 1.0).rgb * 0.15;
  sum += texture(u_source, uv + dir * texel * 2.0).rgb * 0.12;
  sum += texture(u_source, uv + dir * texel * 3.0).rgb * 0.09;
  sum += texture(u_source, uv + dir * texel * 4.0).rgb * 0.05;
  return sum;
}

void main() {
  if (u_pass == 0) {
    vec3 c = texture(u_source, v_uv).rgb;
    float br = max(c.r, max(c.g, c.b));
    vec3 bloom = br > u_threshold ? c : vec3(0.0);
    fragColor = vec4(bloom, 1.0);
    return;
  }

  if (u_pass == 1 || u_pass == 2) {
    vec2 dir = u_pass == 1 ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
    fragColor = vec4(sampleBlur(v_uv, dir), 1.0);
    return;
  }

  vec3 base = texture(u_source, v_uv).rgb;
  fragColor = vec4(base, 1.0);
}