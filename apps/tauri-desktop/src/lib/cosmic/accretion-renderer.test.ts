import { describe, expect, it } from "vitest";
import { accretionScale } from "./accretion-renderer";

describe("accretionScale", () => {
  it("réduit l'échelle en mode docké", () => {
    expect(accretionScale(0)).toBe(1);
    expect(accretionScale(1)).toBeCloseTo(0.28);
    expect(accretionScale(0.5)).toBeCloseTo(0.64);
  });
});