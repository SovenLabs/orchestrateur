import { describe, expect, it } from "vitest";
import { parseBhDatFile } from "$lib/cosmic/cosmic-bh-assets";

describe("parseBhDatFile", () => {
  it("parse width/height et pixels RG", () => {
    const raw = new Float32Array(2 + 2 * 2 * 2);
    raw[0] = 2;
    raw[1] = 2;
    raw[2] = 1.5;
    raw[3] = 0.25;
    raw[4] = 2.5;
    raw[5] = 0.5;
    raw[6] = 3.5;
    raw[7] = 0.75;
    raw[8] = 4.5;
    raw[9] = 1.0;

    const parsed = parseBhDatFile(raw.buffer);
    expect(parsed?.width).toBe(2);
    expect(parsed?.height).toBe(2);
    expect(parsed?.pixels.length).toBe(8);
    expect(parsed?.pixels[0]).toBeCloseTo(1.5);
    expect(parsed?.pixels[7]).toBeCloseTo(1.0);
  });

  it("rejette buffer trop court", () => {
    const raw = new Float32Array([512, 512]);
    expect(parseBhDatFile(raw.buffer)).toBeNull();
  });
});