import { describe, expect, it } from "vitest";
import { computeReconnectDelay } from "./reconnect";

describe("computeReconnectDelay", () => {
  it("applique le backoff exponentiel", () => {
    expect(computeReconnectDelay(0, 500, 30_000)).toBe(500);
    expect(computeReconnectDelay(1, 500, 30_000)).toBe(1000);
    expect(computeReconnectDelay(2, 500, 30_000)).toBe(2000);
  });

  it("plafonne au max", () => {
    expect(computeReconnectDelay(10, 500, 30_000)).toBe(30_000);
  });
});