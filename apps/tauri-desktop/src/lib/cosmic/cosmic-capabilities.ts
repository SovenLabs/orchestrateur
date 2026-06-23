export type CosmicCapabilities = {
  webgl2: boolean;
  floatFramebuffer: boolean;
  floatBlend: boolean;
  floatLinear: boolean;
};

const FALLBACK: CosmicCapabilities = {
  webgl2: false,
  floatFramebuffer: false,
  floatBlend: false,
  floatLinear: false,
};

export function probeCosmicCapabilities(
  gl: WebGL2RenderingContext | null,
): CosmicCapabilities {
  if (!gl) return FALLBACK;

  const floatFb = gl.getExtension("EXT_color_buffer_float") !== null;
  const halfFloat = gl.getExtension("OES_texture_half_float") !== null;
  const floatBlend = gl.getExtension("EXT_float_blend") !== null;
  const floatLinear = gl.getExtension("OES_texture_float_linear") !== null;

  return {
    webgl2: true,
    floatFramebuffer: floatFb && halfFloat,
    floatBlend,
    floatLinear,
  };
}

export function effectiveDpr(devicePixelRatio: number, dprCap: number): number {
  return Math.min(devicePixelRatio || 1, dprCap);
}