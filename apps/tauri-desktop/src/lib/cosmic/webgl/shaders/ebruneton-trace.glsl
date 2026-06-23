// Eric Bruneton — BSD-3-Clause (arxiv:2010.08735) — fonctions TraceRay adaptées
const float EB_KMU = 4.0 / 27.0;
const float EB_PI = 3.14159265359;

float eb_getUapsis(float e_square) {
  float x = (2.0 / EB_KMU) * e_square - 1.0;
  return 1.0 / 3.0 + (2.0 / 3.0) * sin(asin(x) / 3.0);
}

float eb_texCoord(float x, float size) {
  return 0.5 / size + x * (1.0 - 1.0 / size);
}

vec2 eb_lookupDeflection(sampler2D tex, vec2 texSize, float e_square, float u) {
  float tex_u = eb_texCoord(
    e_square < EB_KMU
      ? 0.5 - sqrt(-log(1.0 - e_square / EB_KMU) / 50.0)
      : 0.5 + sqrt(-log(1.0 - EB_KMU / e_square) / 50.0),
    texSize.x);
  float tex_v;
  if (e_square > EB_KMU) {
    float x = u < 2.0 / 3.0 ? -sqrt(2.0 / 3.0 - u) : sqrt(u - 2.0 / 3.0);
    tex_v = eb_texCoord((sqrt(2.0 / 3.0) + x) / (sqrt(2.0 / 3.0) + sqrt(1.0 / 3.0)), texSize.y);
  } else {
    tex_v = eb_texCoord(1.0 - sqrt(max(1.0 - u / eb_getUapsis(e_square), 0.0)), texSize.y);
  }
  return texture(tex, vec2(tex_u, tex_v)).rg;
}

float eb_traceDeflection(sampler2D deflectionTex, vec2 deflectionSize, float p_r, float delta) {
  float u = 1.0 / max(p_r, 0.001);
  float u_dot = -u / tan(max(delta, 0.02));
  float e_square = u_dot * u_dot + u * u * (1.0 - u);
  if (e_square < EB_KMU && u > 2.0 / 3.0) return -1.0;
  vec2 defl = eb_lookupDeflection(deflectionTex, deflectionSize, e_square, u);
  float ray_deflection = defl.x;
  if (u_dot > 0.0) {
    vec2 apsis = eb_lookupDeflection(deflectionTex, deflectionSize, e_square, eb_getUapsis(e_square));
    ray_deflection = e_square < EB_KMU ? 2.0 * apsis.x - ray_deflection : -1.0;
  }
  return ray_deflection;
}