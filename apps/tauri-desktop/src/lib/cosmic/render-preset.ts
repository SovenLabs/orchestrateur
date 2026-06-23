import type { CosmicCapabilities } from "$lib/cosmic/cosmic-capabilities";

export type RenderPreset = "eco" | "cinema";

export type PostFxFlags = {
  aces: boolean;
  aberration: boolean;
  grain: boolean;
  vignette: boolean;
};

export type RenderPresetConfig = {
  preset: RenderPreset;
  dprCap: number;
  bloom: boolean;
  bloomScale: number;
  kawasePasses: number;
  postFx: PostFxFlags;
  galaxyWebGl: boolean;
  bodies3d: boolean;
  exportMax: { width: number; height: number; fps: number; bitrate: number };
};

const ECO: RenderPresetConfig = {
  preset: "eco",
  dprCap: 1,
  bloom: false,
  bloomScale: 0,
  kawasePasses: 0,
  postFx: { aces: true, aberration: false, grain: false, vignette: true },
  galaxyWebGl: false,
  bodies3d: false,
  exportMax: { width: 1920, height: 1080, fps: 30, bitrate: 8_000_000 },
};

const CINEMA: RenderPresetConfig = {
  preset: "cinema",
  dprCap: 2,
  bloom: true,
  bloomScale: 0.5,
  kawasePasses: 4,
  postFx: { aces: true, aberration: true, grain: true, vignette: true },
  galaxyWebGl: true,
  bodies3d: true,
  exportMax: { width: 3840, height: 2160, fps: 60, bitrate: 28_000_000 },
};

export function resolvePresetConfig(
  preset: RenderPreset,
  capabilities: CosmicCapabilities,
): RenderPresetConfig {
  const base = preset === "cinema" ? CINEMA : ECO;
  if (!capabilities.floatFramebuffer) {
    return {
      ...ECO,
      preset: "eco",
      postFx: { aces: true, aberration: false, grain: false, vignette: true },
    };
  }
  if (!capabilities.floatBlend && preset === "cinema") {
    return { ...base, bloom: false, kawasePasses: 0 };
  }
  return base;
}

export function postFxBitmask(flags: PostFxFlags): number {
  let mask = 0;
  if (flags.aces) mask |= 1;
  if (flags.aberration) mask |= 2;
  if (flags.grain) mask |= 4;
  if (flags.vignette) mask |= 8;
  return mask;
}