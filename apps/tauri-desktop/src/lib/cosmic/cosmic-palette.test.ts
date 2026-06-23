import { describe, expect, it } from "vitest";
import { COSMIC_PALETTE, paletteVec3 } from "$lib/cosmic/cosmic-palette";

describe("COSMIC_PALETTE", () => {
  it("aligne starfield Godot", () => {
    expect(COSMIC_PALETTE.starfieldTop).toEqual([0.02, 0.02, 0.06]);
    expect(COSMIC_PALETTE.starTint).toEqual([0.55, 0.7, 1.0]);
  });

  it("paletteVec3 retourne mauve par défaut", () => {
    expect(paletteVec3("mauve")).toEqual(COSMIC_PALETTE.mauve);
  });
});