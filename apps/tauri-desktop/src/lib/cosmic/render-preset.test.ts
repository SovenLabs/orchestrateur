import { describe, expect, it } from "vitest";
import { postFxBitmask, resolvePresetConfig } from "$lib/cosmic/render-preset";

describe("resolvePresetConfig", () => {
  const fullCaps = {
    webgl2: true,
    floatFramebuffer: true,
    floatBlend: true,
    floatLinear: true,
  };

  it("cinema active bloom et galaxies WebGL", () => {
    const cfg = resolvePresetConfig("cinema", fullCaps);
    expect(cfg.bloom).toBe(true);
    expect(cfg.galaxyWebGl).toBe(true);
    expect(cfg.bodies3d).toBe(true);
    expect(cfg.kawasePasses).toBeGreaterThan(0);
  });

  it("eco désactive bloom", () => {
    const cfg = resolvePresetConfig("eco", fullCaps);
    expect(cfg.bloom).toBe(false);
    expect(cfg.galaxyWebGl).toBe(false);
  });

  it("fallback eco si pas de float FBO", () => {
    const cfg = resolvePresetConfig("cinema", {
      ...fullCaps,
      floatFramebuffer: false,
    });
    expect(cfg.preset).toBe("eco");
    expect(cfg.bloom).toBe(false);
  });
});

describe("postFxBitmask", () => {
  it("encode les flags post", () => {
    expect(postFxBitmask({ aces: true, aberration: false, grain: false, vignette: true })).toBe(9);
    expect(postFxBitmask({ aces: true, aberration: true, grain: true, vignette: true })).toBe(15);
  });
});