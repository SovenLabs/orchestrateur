/**
 * Chargeur textures précalculées ebruneton (BSD-3-Clause).
 * Format : float32[width, height, rg × w × h] — preprocess_main.cc
 */
export type BhTextureData = {
  width: number;
  height: number;
  pixels: Float32Array;
};

export type BhPrecomputedAssets = {
  deflection: BhTextureData | null;
  inverseRadius: BhTextureData | null;
  ready: boolean;
};

const ASSET_PATHS = {
  deflection: "/cosmic/precomputed/deflection.dat",
  inverseRadius: "/cosmic/precomputed/inverse_radius.dat",
} as const;

export function parseBhDatFile(buffer: ArrayBuffer): BhTextureData | null {
  const floats = new Float32Array(buffer);
  if (floats.length < 4) return null;
  const width = Math.floor(floats[0]);
  const height = Math.floor(floats[1]);
  if (width < 1 || height < 1) return null;
  const expected = 2 + width * height * 2;
  if (floats.length < expected) return null;
  return {
    width,
    height,
    pixels: floats.subarray(2, expected),
  };
}

export async function loadBhPrecomputedAssets(): Promise<BhPrecomputedAssets> {
  try {
    const [deflectionRes, inverseRes] = await Promise.all([
      fetch(ASSET_PATHS.deflection),
      fetch(ASSET_PATHS.inverseRadius),
    ]);
    if (!deflectionRes.ok || !inverseRes.ok) {
      return { deflection: null, inverseRadius: null, ready: false };
    }
    const [defBuf, invBuf] = await Promise.all([
      deflectionRes.arrayBuffer(),
      inverseRes.arrayBuffer(),
    ]);
    const deflection = parseBhDatFile(defBuf);
    const inverseRadius = parseBhDatFile(invBuf);
    return {
      deflection,
      inverseRadius,
      ready: !!(deflection && inverseRadius),
    };
  } catch {
    return { deflection: null, inverseRadius: null, ready: false };
  }
}