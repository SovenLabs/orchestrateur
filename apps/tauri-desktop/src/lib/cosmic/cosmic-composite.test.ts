import { describe, expect, it } from "vitest";

describe("cosmic-composite", () => {
  it("module exporté", async () => {
    const mod = await import("$lib/cosmic/cosmic-composite");
    expect(typeof mod.compositeCosmicLayers).toBe("function");
    expect(typeof mod.createCompositeCanvas).toBe("function");
  });
});