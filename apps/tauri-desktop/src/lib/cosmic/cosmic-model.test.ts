import { describe, expect, it } from "vitest";
import { buildCosmicScene, buildWormholes } from "./cosmic-model";
import type { CosmicPlacement } from "./cosmic-taxonomy";

function mem(
  id: string,
  tags: string[],
  backlinks: Array<{ target: string; score: number; kind: string }> = [],
) {
  return {
    id,
    title: `Memory ${id}`,
    tags,
    updated_at: "",
    backlink_count: backlinks.length,
    backlinks,
  };
}

describe("buildCosmicScene", () => {
  it("groupe les mémoires en galaxies", () => {
    const cache = new Map<string, CosmicPlacement>();
    const scene = buildCosmicScene(
      Array.from({ length: 20 }, (_, i) =>
        mem(String(i), i % 2 === 0 ? ["rust", "embedding", "vector"] : ["nuance", "agent", "insight"]),
      ),
      cache,
    );
    expect(scene.galaxies.length).toBeGreaterThan(0);
    const total = scene.galaxies.reduce((n, g) => n + g.memoryCount, 0);
    expect(total).toBe(20);
  });
});

describe("buildWormholes", () => {
  it("crée un trou de ver cross-étoile", () => {
    const cache = new Map<string, CosmicPlacement>();
    const memories = [
      mem("a", ["rust", "embedding", "vector"], [{ target: "b", score: 0.9, kind: "semantic" }]),
      mem("b", ["nuance", "agent", "insight"]),
    ];
    const scene = buildCosmicScene(memories, cache);
    const wormholes = buildWormholes(memories, scene.placementByMemory);
    expect(wormholes.length).toBeGreaterThan(0);
    expect(wormholes[0].fromStarId).not.toBe(wormholes[0].toStarId);
  });
});