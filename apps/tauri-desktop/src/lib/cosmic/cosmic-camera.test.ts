import { describe, expect, it } from "vitest";
import {
  defaultCamera,
  panToCenterWorld,
  screenToWorld,
  wheelZoomFactor,
  worldToScreen,
  zoomAtPointer,
} from "$lib/cosmic/cosmic-camera";

describe("cosmic-camera", () => {
  const cx = 400;
  const cy = 300;

  it("worldToScreen centre le pivot", () => {
    const cam = defaultCamera();
    const s = worldToScreen(cx, cy, cx, cy, cam);
    expect(s.x).toBe(cx);
    expect(s.y).toBe(cy);
  });

  it("zoomAtPointer conserve le point sous le curseur", () => {
    const cam = { ...defaultCamera(), zoom: 1 };
    const ptr = { x: 500, y: 350 };
    const before = screenToWorld(ptr.x, ptr.y, cx, cy, cam);
    const next = zoomAtPointer(cam, cx, cy, ptr.x, ptr.y, 1.5);
    const after = screenToWorld(ptr.x, ptr.y, cx, cy, next);
    expect(after.x).toBeCloseTo(before.x, 1);
    expect(after.y).toBeCloseTo(before.y, 1);
  });

  it("wheelZoomFactor dézoome avec delta positif", () => {
    expect(wheelZoomFactor(120)).toBeLessThan(1);
    expect(wheelZoomFactor(-120)).toBeGreaterThan(1);
  });

  it("panToCenterWorld aligne un point monde au centre", () => {
    const pan = panToCenterWorld(600, 400, cx, cy, 2);
    const cam = { ...defaultCamera(), ...pan, zoom: 2 };
    const s = worldToScreen(600, 400, cx, cy, cam);
    expect(s.x).toBeCloseTo(cx, 1);
    expect(s.y).toBeCloseTo(cy, 1);
  });
});