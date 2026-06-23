import { describe, expect, it } from "vitest";
import { buildCosmicScene } from "$lib/cosmic/cosmic-model";
import { buildCelestialBodies } from "$lib/cosmic/three/cosmic-body-layout";

function mem(id: string, tags: string[]) {
  return {
    id,
    title: `Memory ${id}`,
    tags,
    updated_at: "",
    backlink_count: 0,
    backlinks: [],
  };
}

describe("buildCelestialBodies", () => {
  const memories = [
    mem("m1", ["rust", "embedding", "vector"]),
    mem("m2", ["nuance", "agent", "insight"]),
    mem("m3", ["rust", "daemon", "websocket"]),
  ];
  const scene = buildCosmicScene(memories, new Map());

  it("retourne vide au niveau cosmos", () => {
    const bodies = buildCelestialBodies({
      scene,
      cx: 400,
      cy: 300,
      baseRadius: 120,
      time: 0,
      dockT: 0,
      zoomLevel: "cosmos",
      focusGalaxyId: null,
      focusStarId: null,
      zoomGalaxyT: 0,
    });
    expect(bodies).toHaveLength(0);
  });

  it("produit étoiles et planètes au niveau galaxie", () => {
    const galaxyId = scene.galaxies[0]?.id;
    const bodies = buildCelestialBodies({
      scene,
      cx: 400,
      cy: 300,
      baseRadius: 120,
      time: 1,
      dockT: 0,
      zoomLevel: "galaxy",
      focusGalaxyId: galaxyId ?? null,
      focusStarId: null,
      zoomGalaxyT: 1,
    });
    expect(bodies.some((b) => b.kind === "star")).toBe(true);
    expect(bodies.some((b) => b.kind === "planet")).toBe(true);
  });

  it("produit lunes au niveau corps", () => {
    const galaxy = scene.galaxies[0];
    const star = galaxy?.stars[0];
    const bodies = buildCelestialBodies({
      scene,
      cx: 400,
      cy: 300,
      baseRadius: 120,
      time: 2,
      dockT: 0,
      zoomLevel: "body",
      focusGalaxyId: galaxy?.id ?? null,
      focusStarId: star?.id ?? null,
      zoomGalaxyT: 1,
    });
    expect(bodies.some((b) => b.kind === "moon")).toBe(true);
  });
});