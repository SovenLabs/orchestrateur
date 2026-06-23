import { describe, expect, it } from "vitest";
import {
  buildMemoryEdges,
  buildOrbitalNodes,
  hitTestOrbitalNode,
  nodePosition,
  orbitalNodeBudget,
} from "./orbital-nodes";

describe("orbitalNodeBudget", () => {
  it("augmente avec le nombre de mémoires", () => {
    expect(orbitalNodeBudget(0)).toBe(8);
    expect(orbitalNodeBudget(100)).toBeGreaterThan(8);
  });
});

describe("buildOrbitalNodes", () => {
  it("limite à 8 nodes et priorise agents", () => {
    const agents = Array.from({ length: 6 }, (_, i) => ({
      id: String(i),
      name: `Agent ${i}`,
      status: "idle" as const,
      activity: i / 10,
    }));
    const nodes = buildOrbitalNodes(agents, [], [], 8);
    expect(nodes.length).toBeLessThanOrEqual(8);
    expect(nodes.filter((n) => n.kind === "agent").length).toBe(4);
  });
});

describe("buildMemoryEdges", () => {
  it("relie les nœuds mémoire en anneau", () => {
    const nodes = [
      { id: "memory-a", kind: "memory" as const, label: "A", activity: 0.5, angle: 0, orbitFactor: 1 },
      { id: "memory-b", kind: "memory" as const, label: "B", activity: 0.5, angle: 1, orbitFactor: 1 },
      { id: "memory-c", kind: "memory" as const, label: "C", activity: 0.5, angle: 2, orbitFactor: 1 },
    ];
    const edges = buildMemoryEdges(nodes);
    expect(edges.length).toBeGreaterThanOrEqual(3);
  });
});

describe("hitTestOrbitalNode", () => {
  it("détecte un clic sur un node", () => {
    const nodes = buildOrbitalNodes(
      [{ id: "a", name: "Esprit", status: "active", activity: 0.9 }],
      [],
      [],
      8,
    );
    const pos = nodePosition(400, 300, 120, nodes[0], 0, 0);
    const hit = hitTestOrbitalNode(400, 300, 120, nodes, 0, 0, pos.x, pos.y);
    expect(hit?.id).toBe(nodes[0].id);
  });
});