import { describe, expect, it } from "vitest";
import { computeBlackholeLayout } from "./blackhole-layout";

describe("computeBlackholeLayout", () => {
  it("interpole entre expanded et docked", () => {
    const expanded = computeBlackholeLayout(1200, 800, 0);
    const docked = computeBlackholeLayout(1200, 800, 1);
    const mid = computeBlackholeLayout(1200, 800, 0.5);

    expect(expanded.baseRadius).toBeGreaterThan(docked.baseRadius);
    expect(expanded.cx).toBe(600);
    expect(expanded.cy).toBe(400);
    expect(mid.cx).toBeGreaterThan(expanded.cx);
    expect(mid.cx).toBeLessThan(docked.cx);
    expect(mid.chatFade).toBeLessThan(1);
  });
});