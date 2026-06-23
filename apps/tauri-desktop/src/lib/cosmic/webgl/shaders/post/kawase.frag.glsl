#version 300 es
precision highp float;

in vec2 v_uv;
out vec4 fragColor;

uniform sampler2D u_source;
uniform vec2 u_resolution;
uniform float u_offset;
uniform int u_pass; // 0 = downsample, 1 = kawase blur

void main() {
  vec2 texel = 1.0 / u_resolution;

  if (u_pass == 0) {
    vec3 sum = texture(u_source, v_uv).rgb;
    sum += texture(u_source, v_uv + vec2(texel.x, 0.0)).rgb;
    sum += texture(u_source, v_uv + vec2(0.0, texel.y)).rgb;
    sum += texture(u_source, v_uv + texel).rgb;
    fragColor = vec4(sum * 0.25, 1.0);
    return;
  }

  vec2 d = texel * u_offset;
  vec3 sum = texture(u_source, v_uv + vec2(-d.x, d.y)).rgb;
  sum += texture(u_source, v_uv + vec2(d.x, d.y)).rgb;
  sum += texture(u_source, v_uv + vec2(d.x, -d.y)).rgb;
  sum += texture(u_source, v_uv + vec2(-d.x, -d.y)).rgb;
  fragColor = vec4(sum * 0.25, 1.0);
}