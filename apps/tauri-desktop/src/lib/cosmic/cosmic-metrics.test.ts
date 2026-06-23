import { describe, expect, it } from "vitest";
import { computeAgentsSync, computeCoherence } from "./cosmic-metrics";

describe("computeAgentsSync", () => {
  it("compte les agents actifs", () => {
    const r = computeAgentsSync([
      { id: "a", name: "A", status: "active", activity: 1 },
      { id: "b", name: "B", status: "idle", activity: 0 },
    ]);
    expect(r.label).toBe("1/2");
  });
});

describe("computeCoherence", () => {
  it("retourne 0 si déconnecté", () => {
    expect(computeCoherence({ connected: false, llmAvailable: true, embeddingAvailable: true, agentActivity: 1, nuanceDepth: 1 })).toBe(0);
  });

  it("monte avec la santé et la nuance", () => {
    const low = computeCoherence({ connected: true, llmAvailable: false, embeddingAvailable: false, agentActivity: 0, nuanceDepth: 0 });
    const high = computeCoherence({ connected: true, llmAvailable: true, embeddingAvailable: true, agentActivity: 0.8, nuanceDepth: 0.9 });
    expect(high).toBeGreaterThan(low);
  });
});