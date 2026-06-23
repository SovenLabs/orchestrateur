#version 300 es
precision highp float;

in vec2 v_uv;
out vec4 fragColor;

uniform vec2 u_resolution;
uniform float u_time;
uniform vec2 u_bh_center;
uniform float u_bh_radius;
uniform float u_activity;
uniform float u_dock_t;
uniform float u_connected;
uniform float u_thinking;
uniform vec3 u_core_tint;
uniform vec2 u_camera_pan;
uniform float u_camera_zoom;
uniform vec2 u_camera_tilt;
uniform float u_use_ebruneton;
uniform sampler2D u_deflection_tex;
uniform vec2 u_deflection_size;

// Godot starfield_background.gdshader
const vec3 SKY_TOP = vec3(0.02, 0.02, 0.06);
const vec3 SKY_HORIZON = vec3(0.01, 0.01, 0.02);
const vec3 STAR_TINT = vec3(0.55, 0.7, 1.0);

const float PI = 3.14159265359;
const float DISK_INCL = 0.42;

float hash21(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

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
  for (int i = 0; i < 6; i++) {
    v += a * noise3(p);
    p *= 2.07;
    a *= 0.5;
  }
  return v;
}

float starLayer(vec3 rd, float scale, float density, float layer) {
  vec2 uv = rd.xy / (abs(rd.z) + 0.38);
  vec2 gv = floor(uv * scale);
  vec2 f = fract(uv * scale) - 0.5;
  float n = hash21(gv + layer * 17.3);
  if (n < 1.0 - density) return 0.0;
  float br = pow(hash21(gv + layer * 3.1), 3.5);
  float sz = mix(0.01, 0.04, hash21(gv + 9.0));
  float d = length(f);
  float core = smoothstep(sz, sz * 0.06, d);
  float tw = 0.55 + 0.45 * sin(u_time * (0.7 + n * 4.0) + n * 50.0);
  return core * br * tw;
}

vec3 starfield(vec3 rd) {
  float h = rd.y * 0.5 + 0.5;
  vec3 col = mix(SKY_HORIZON, SKY_TOP, pow(h, 1.4));
  col += vec3(0.06, 0.04, 0.12) * fbm(rd * 4.0 + vec3(u_time * 0.008)) * 0.35;
  col += STAR_TINT * starLayer(rd, 260.0, 0.022, 0.0) * 0.35;
  col += STAR_TINT * starLayer(rd, 420.0, 0.014, 1.0) * 0.28;
  col += vec3(1.0, 0.95, 0.85) * starLayer(rd, 680.0, 0.007, 2.0) * 0.22;
  col += vec3(1.0, 0.9, 0.7) * starLayer(rd, 110.0, 0.04, 3.0) * 0.5;
  return col;
}

vec3 bendRay(vec3 rd, vec3 bhPos, float mass) {
  vec3 outRd = rd;
  for (int i = 0; i < 24; i++) {
    vec3 toBH = bhPos - outRd * 2.8;
    float h2 = dot(toBH.xy, toBH.xy) + 0.0008;
    float strength = mass / (h2 * (1.0 + float(i) * 0.04));
    outRd.xy += toBH.xy * strength * 0.0011;
    outRd = normalize(outRd);
  }
  return outRd;
}

float diskIntensity(vec2 p, float bhR) {
  float c = cos(DISK_INCL);
  float s = sin(DISK_INCL);
  vec2 q = vec2(p.x, p.y * c - p.y * s * 0.15);
  float r = length(q) / bhR;
  float inner = 1.55;
  float outer = 6.8 * (0.7 + u_dock_t * 0.3);
  float band = smoothstep(inner, inner + 0.15, r) * (1.0 - smoothstep(outer - 0.5, outer, r));
  if (band < 0.01) return 0.0;

  float phi = atan(q.y, q.x);
  float spin = phi + u_time * (2.0 + u_activity) - log(max(r, 1.0)) * 4.0;
  float turb = fbm(vec3(q * 0.06, u_time * 0.45));
  float streaks = 0.4 + 0.6 * abs(sin(spin * 9.0 + turb * 6.0));
  float thickness = smoothstep(0.08, 0.0, abs(q.y) / bhR);
  return band * streaks * turb * thickness;
}

vec3 diskColor(vec2 p, float bhR, float intensity) {
  float c = cos(DISK_INCL);
  vec2 q = vec2(p.x, p.y * c);
  float phi = atan(q.y, q.x);
  float approach = 0.5 + 0.5 * sin(phi);
  float beaming = pow(approach, 2.8) * 1.6 + pow(1.0 - approach, 0.6) * 0.35;
  vec3 hot = mix(vec3(1.0, 0.28, 0.04), vec3(0.38, 0.68, 1.0), approach);
  return hot * intensity * beaming * 1.45;
}

vec2 worldFromScreen(vec2 screenPx) {
  return u_bh_center + (screenPx - u_bh_center - u_camera_pan) / u_camera_zoom;
}

void main() {
  vec2 fragCoord = v_uv * u_resolution;
  vec2 worldPx = worldFromScreen(fragCoord);
  vec2 uv = (worldPx - 0.5 * u_resolution) / u_resolution.y;
  vec2 bhUV = (u_bh_center - 0.5 * u_resolution) / u_resolution.y;
  vec2 p = uv - bhUV;

  float bhR = max(0.035, u_bh_radius / u_resolution.y / u_camera_zoom);
  float mass = bhR * bhR * (22.0 + u_activity * 8.0);
  float dist = length(p);

  float pulse = 1.0;
  if (u_connected > 0.5) pulse += sin(u_time * 2.4) * 0.035;
  if (u_thinking > 0.5) pulse += sin(u_time * 3.5) * 0.025;

  vec3 rd = normalize(vec3(uv + u_camera_tilt * 0.15, 1.2));
  vec3 bentRd = bendRay(rd, vec3(bhUV, 0.0), mass);
  vec3 col = starfield(bentRd);

  float rs = bhR * 0.88;
  float diskI = diskIntensity(p, bhR);
  col += diskColor(p, bhR, diskI);

  float ringR1 = 1.50;
  float ringR2 = 1.62;
  if (u_use_ebruneton > 0.5) {
    float delta = atan(length(p), 1.2);
    float eb = eb_traceDeflection(u_deflection_tex, u_deflection_size, dist / max(bhR, 0.001), delta);
    if (eb > 0.0) {
      ringR1 += sin(eb) * 0.04;
      ringR2 += cos(eb * 0.5) * 0.03;
    }
  }
  float ring1 = exp(-pow((dist - bhR * ringR1 * pulse) / (bhR * 0.014), 2.0));
  float ring2 = exp(-pow((dist - bhR * ringR2 * pulse) / (bhR * 0.008), 2.0)) * 0.42;
  float ring3 = exp(-pow((dist - bhR * 2.35 * pulse) / (bhR * 0.05), 2.0)) * 0.08;
  col += vec3(1.0, 0.96, 0.88) * (ring1 * 1.65 + ring2 * 1.05 + ring3);

  float lensArc = exp(-pow((dist - bhR * 2.8) / (bhR * 0.25), 2.0)) * 0.12;
  col += starfield(normalize(vec3(normalize(p) * 0.4 + bentRd.xy, bentRd.z))) * lensArc;

  float horizon = smoothstep(rs * 1.02, rs * 0.55, dist);
  col *= 1.0 - horizon * 0.15;
  col = mix(col, vec3(0.0), smoothstep(rs * 0.98, rs * 0.35, dist));

  col += u_core_tint * exp(-dist / (bhR * 5.0)) * 0.05 * (1.0 + u_activity);

  // HDR scene buffer — tone mapping in post pipeline
  col *= 1.35;
  fragColor = vec4(col, 1.0);
}