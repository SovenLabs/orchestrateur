#version 300 es
precision highp float;

in vec2 v_uv;
out vec4 fragColor;

uniform sampler2D u_source;
uniform float u_threshold;

void main() {
  vec3 c = texture(u_source, v_uv).rgb;
  float br = max(c.r, max(c.g, c.b));
  vec3 bloom = br > u_threshold ? c : vec3(0.0);
  fragColor = vec4(bloom, 1.0);
}