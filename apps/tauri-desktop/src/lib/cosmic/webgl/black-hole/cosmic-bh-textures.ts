import { parseBhDatFile, type BhTextureData } from "$lib/cosmic/cosmic-bh-assets";

export type BhGlTextures = {
  deflection: WebGLTexture;
  inverseRadius: WebGLTexture;
  deflectionSize: [number, number];
  inverseSize: [number, number];
};

export function uploadBhTexture(
  gl: WebGL2RenderingContext,
  data: BhTextureData,
): WebGLTexture | null {
  const tex = gl.createTexture();
  if (!tex) return null;

  const rgba = new Float32Array(data.width * data.height * 4);
  for (let i = 0; i < data.width * data.height; i++) {
    rgba[i * 4] = data.pixels[i * 2];
    rgba[i * 4 + 1] = data.pixels[i * 2 + 1];
    rgba[i * 4 + 2] = 0;
    rgba[i * 4 + 3] = 1;
  }

  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texImage2D(
    gl.TEXTURE_2D,
    0,
    gl.RGBA32F,
    data.width,
    data.height,
    0,
    gl.RGBA,
    gl.FLOAT,
    rgba,
  );
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.bindTexture(gl.TEXTURE_2D, null);
  return tex;
}

export function createBhGlTextures(
  gl: WebGL2RenderingContext,
  deflection: BhTextureData | null,
  inverseRadius: BhTextureData | null,
): BhGlTextures | null {
  if (!deflection || !inverseRadius) return null;
  const deflectionTex = uploadBhTexture(gl, deflection);
  const inverseTex = uploadBhTexture(gl, inverseRadius);
  if (!deflectionTex || !inverseTex) return null;
  return {
    deflection: deflectionTex,
    inverseRadius: inverseTex,
    deflectionSize: [deflection.width, deflection.height],
    inverseSize: [inverseRadius.width, inverseRadius.height],
  };
}