import { describe, expect, it } from "vitest";
import { assignPlacement, resolveGalaxyId, resolveStarId } from "./cosmic-taxonomy";

function mem(id: string, tags: string[]) {
  return {
    id,
    title: `Note ${tags[0] ?? "x"} alpha`,
    tags,
    updated_at: "",
    backlink_count: 2,
    backlinks: [],
  };
}

describe("cosmic-taxonomy", () => {
  it("assigne une galaxie via les tags", () => {
    expect(resolveGalaxyId(mem("1", ["embedding", "vector", "rust"]))).toBe("technique");
    expect(resolveGalaxyId(mem("2", ["nuance", "agent", "insight"]))).toBe("cognition");
  });

  it("garde une assignation stable en cache", () => {
    const cache = new Map();
    const m = mem("stable-1", ["graphe", "wikilink", "memoire"]);
    const first = assignPlacement(m, cache);
    const second = assignPlacement({ ...m, tags: ["autre"] }, cache);
    expect(first).toEqual(second);
  });

  it("produit des étoiles distinctes dans une galaxie", () => {
    const g = resolveGalaxyId(mem("a", ["memoire", "graphe", "lancedb"]));
    const s1 = resolveStarId(mem("a", ["memoire", "graphe", "lancedb"]), g);
    const s2 = resolveStarId(mem("b", ["memoire", "wikilink", "backlink"]), g);
    expect(s1).not.toBe(s2);
  });
});