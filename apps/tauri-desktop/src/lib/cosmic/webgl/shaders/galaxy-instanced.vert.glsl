#version 300 es
layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_center;
layout(location = 2) in float a_radius;
layout(location = 3) in vec3 a_tint;
layout(location = 4) in float a_nebula;

uniform vec2 u_resolution;
uniform vec2 u_bh_center;
uniform vec2 u_camera_pan;
uniform float u_camera_zoom;

out vec2 v_local;
out vec3 v_tint;
out float v_nebula;
out float v_radius;

void main() {
  vec2 world = a_center + a_position * a_radius;
  vec2 screen = u_bh_center + (world - u_bh_center) * u_camera_zoom + u_camera_pan;
  vec2 ndc = (screen / u_resolution) * 2.0 - 1.0;
  ndc.y = -ndc.y;
  gl_Position = vec4(ndc, 0.0, 1.0);
  v_local = a_position;
  v_tint = a_tint;
  v_nebula = a_nebula;
  v_radius = a_radius;
}