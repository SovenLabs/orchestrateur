import { describe, expect, it } from "vitest";
import { mergeFragmentShaders } from "$lib/cosmic/webgl/gl-utils";

describe("mergeFragmentShaders", () => {
  it("place #version et precision en tête", () => {
    const merged = mergeFragmentShaders(
      "const float A = 1.0;",
      "#version 300 es\nprecision highp float;\nvoid main() {}",
    );
    expect(merged.startsWith("#version 300 es\nprecision highp float;\n")).toBe(true);
    expect(merged).toContain("const float A = 1.0;");
    expect(merged).toContain("void main() {}");
    expect(merged.match(/#version/g)?.length).toBe(1);
    expect(merged.match(/precision highp float/g)?.length).toBe(1);
  });
});