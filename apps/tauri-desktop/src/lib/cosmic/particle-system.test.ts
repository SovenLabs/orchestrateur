import { describe, expect, it } from "vitest";
import { createParticleSystem, feedBurstHue } from "./particle-system";

describe("feedBurstHue", () => {
  it("varie avec la profondeur nuance", () => {
    expect(feedBurstHue(0)).toBe(255);
    expect(feedBurstHue(1)).toBe(330);
  });
});

describe("createParticleSystem", () => {
  it("émet un burst feed vers l'attracteur", () => {
    const ps = createParticleSystem();
    ps.emitFeedBurst({ x: 0, y: 100 }, { x: 200, y: 50 }, 8);
    expect(ps.particles.length).toBe(8);
    ps.clear();
    expect(ps.particles.length).toBe(0);
  });

  it("fait évoluer et retirer les particules mortes", () => {
    const ps = createParticleSystem();
    ps.emitAmbient(3, 100, 100, 40);
    const before = ps.particles.length;
    for (let i = 0; i < 200; i++) {
      ps.step(0.05, { x: 100, y: 100 });
    }
    expect(ps.particles.length).toBeLessThan(before);
  });

  it("respecte le plafond de particules", () => {
    const ps = createParticleSystem();
    ps.emitFeedBurst({ x: 0, y: 0 }, { x: 100, y: 100 }, 200);
    expect(ps.particles.length).toBeLessThanOrEqual(240);
  });
});