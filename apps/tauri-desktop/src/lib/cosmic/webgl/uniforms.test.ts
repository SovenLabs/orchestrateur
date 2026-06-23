import { describe, expect, it } from "vitest";
import { buildCosmicUniforms } from "$lib/cosmic/webgl/uniforms";
import { defaultCamera } from "$lib/cosmic/cosmic-camera";
import { resolvePresetConfig } from "$lib/cosmic/render-preset";

describe("buildCosmicUniforms", () => {
  const base = {
    layout: { cx: 400, cy: 300, baseRadius: 120, chatFade: 1 },
    width: 800,
    height: 600,
    time: 12.5,
    nuanceDepth: 0.5,
    dockT: 0,
    scale: 1,
    connected: false,
    thinking: false,
    presetConfig: resolvePresetConfig("cinema", {
      webgl2: true,
      floatFramebuffer: true,
      floatBlend: true,
      floatLinear: true,
    }),
    camera: defaultCamera(),
  };

  it("maps pivot fixe et pan séparé", () => {
    const u = buildCosmicUniforms(base);
    expect(u.bhCenter).toEqual([400, 300]);
    expect(u.cameraPan).toEqual([0, 0]);
    expect(u.bhRadius).toBeCloseTo(120 * 0.16, 1);
  });

  it("applique zoom sur le rayon écran", () => {
    const u = buildCosmicUniforms({
      ...base,
      camera: { ...defaultCamera(), panX: 50, panY: -20, zoom: 2 },
    });
    expect(u.bhCenter).toEqual([400, 300]);
    expect(u.cameraPan).toEqual([50, -20]);
    expect(u.cameraZoom).toBe(2);
    expect(u.bhRadius).toBeCloseTo(120 * 0.16 * 2, 1);
  });
});