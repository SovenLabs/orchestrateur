import type { MemoryItem } from "$lib/types/ui";
import {
  GALAXY_CATALOG,
  assignPlacement,
  galaxyLabel,
  galaxyRadiusScale,
  planetLabel,
  starLabel,
  type CosmicPlacement,
} from "./cosmic-taxonomy";

export type ZoomLevel = "cosmos" | "galaxy" | "body";

export type CosmicMoon = {
  id: string;
  memoryId: string;
  title: string;
  slot: number;
  activity: number;
};

export type CosmicPlanet = {
  id: string;
  label: string;
  moons: CosmicMoon[];
  angle: number;
  orbitFactor: number;
};

export type CosmicStar = {
  id: string;
  label: string;
  planets: CosmicPlanet[];
  angle: number;
};

export type CosmicGalaxy = {
  id: string;
  label: string;
  stars: CosmicStar[];
  memoryCount: number;
  radius: number;
  angle: number;
  isNebula: boolean;
};

export type WormholeEdge = {
  fromMemoryId: string;
  toMemoryId: string;
  fromStarId: string;
  toStarId: string;
  fromGalaxyId: string;
  toGalaxyId: string;
  strength: number;
};

export type CosmicScene = {
  galaxies: CosmicGalaxy[];
  wormholes: WormholeEdge[];
  placementByMemory: Map<string, CosmicPlacement>;
};

export type CosmicHit = {
  id: string;
  x: number;
  y: number;
  worldX: number;
  worldY: number;
  r: number;
  kind: "galaxy" | "star" | "planet" | "moon";
  label: string;
  galaxyId?: string;
  starId?: string;
  planetId?: string;
  memoryId?: string;
};

export function buildCosmicScene(
  memories: MemoryItem[],
  cache: Map<string, CosmicPlacement>,
): CosmicScene {
  const placementByMemory = new Map<string, CosmicPlacement>();
  for (const mem of memories) {
    const p = assignPlacement(mem, cache);
    placementByMemory.set(mem.id, p);
  }

  const galaxyBuckets = new Map<string, MemoryItem[]>();
  for (const mem of memories) {
    const p = placementByMemory.get(mem.id)!;
    const bucket = galaxyBuckets.get(p.galaxyId) ?? [];
    bucket.push(mem);
    galaxyBuckets.set(p.galaxyId, bucket);
  }

  const activeGalaxyIds = [...galaxyBuckets.keys()].sort();
  const galaxies: CosmicGalaxy[] = [];

  for (let gi = 0; gi < activeGalaxyIds.length; gi++) {
    const galaxyId = activeGalaxyIds[gi];
    const items = galaxyBuckets.get(galaxyId) ?? [];
    const starBuckets = new Map<string, MemoryItem[]>();

    for (const mem of items) {
      const p = placementByMemory.get(mem.id)!;
      const bucket = starBuckets.get(p.starId) ?? [];
      bucket.push(mem);
      starBuckets.set(p.starId, bucket);
    }

    const starIds = [...starBuckets.keys()].sort();
    const stars: CosmicStar[] = starIds.map((starId, si) => {
      const starMemories = starBuckets.get(starId) ?? [];
      const planetBuckets = new Map<string, MemoryItem[]>();
      for (const mem of starMemories) {
        const p = placementByMemory.get(mem.id)!;
        const bucket = planetBuckets.get(p.planetId) ?? [];
        bucket.push(mem);
        planetBuckets.set(p.planetId, bucket);
      }

      const planetIds = [...planetBuckets.keys()].sort();
      const planets: CosmicPlanet[] = planetIds.map((planetId, pi) => {
        const planetMemories = [...(planetBuckets.get(planetId) ?? [])].sort((a, b) =>
          a.id.localeCompare(b.id),
        );
        return {
          id: planetId,
          label: planetLabel(planetId),
          angle: (pi / Math.max(1, planetIds.length)) * Math.PI * 2,
          orbitFactor: 0.85 + (pi % 3) * 0.08,
          moons: planetMemories.map((mem, mi) => ({
            id: `moon-${mem.id}`,
            memoryId: mem.id,
            title: mem.title.slice(0, 18),
            slot: mi,
            activity: Math.min(1, 0.25 + mem.backlink_count * 0.1),
          })),
        };
      });

      return {
        id: starId,
        label: starLabel(starId),
        angle: (si / Math.max(1, starIds.length)) * Math.PI * 2,
        planets,
      };
    });

    galaxies.push({
      id: galaxyId,
      label: galaxyLabel(galaxyId),
      stars,
      memoryCount: items.length,
      radius: galaxyRadiusScale(items.length),
      angle: (gi / Math.max(1, activeGalaxyIds.length)) * Math.PI * 2 - Math.PI / 2,
      isNebula: items.length < 3,
    });
  }

  for (const def of GALAXY_CATALOG) {
    if (!galaxies.some((g) => g.id === def.id) && memories.length === 0) continue;
  }

  const wormholes = buildWormholes(memories, placementByMemory);

  return { galaxies, wormholes, placementByMemory };
}

export function buildWormholes(
  memories: MemoryItem[],
  placementByMemory: Map<string, CosmicPlacement>,
): WormholeEdge[] {
  const edges: WormholeEdge[] = [];
  const seen = new Set<string>();

  for (const mem of memories) {
    const from = placementByMemory.get(mem.id);
    if (!from) continue;
    for (const link of mem.backlinks ?? []) {
      const to = placementByMemory.get(link.target);
      if (!to) continue;
      if (from.starId === to.starId) continue;
      const key = [mem.id, link.target].sort().join(":");
      if (seen.has(key)) continue;
      seen.add(key);
      edges.push({
        fromMemoryId: mem.id,
        toMemoryId: link.target,
        fromStarId: from.starId,
        toStarId: to.starId,
        fromGalaxyId: from.galaxyId,
        toGalaxyId: to.galaxyId,
        strength: Math.min(1, link.score),
      });
    }
  }

  return edges;
}

export function galaxyPosition(
  cx: number,
  cy: number,
  baseRadius: number,
  galaxy: CosmicGalaxy,
  time: number,
  dockT: number,
): { x: number; y: number } {
  const visibility = Math.max(0.2, 1 - dockT * 0.75);
  const r = baseRadius * (0.95 + galaxy.radius * 0.01) * visibility;
  const drift = time * 0.012;
  const a = galaxy.angle + drift;
  return { x: cx + Math.cos(a) * r, y: cy + Math.sin(a) * r * 0.88 };
}

export function starPositionInGalaxy(
  gx: number,
  gy: number,
  galaxy: CosmicGalaxy,
  star: CosmicStar,
  time: number,
  zoomGalaxyT: number,
): { x: number; y: number; radius: number } {
  const spread = galaxy.radius * (0.35 + zoomGalaxyT * 0.45);
  const a = star.angle + time * 0.04;
  const r = spread * 0.55;
  return {
    x: gx + Math.cos(a) * r,
    y: gy + Math.sin(a) * r * 0.9,
    radius: 5 + Math.min(8, star.planets.length * 1.2),
  };
}

export function planetPositionInStar(
  sx: number,
  sy: number,
  starRadius: number,
  planet: CosmicPlanet,
  time: number,
): { x: number; y: number; radius: number } {
  const a = planet.angle + time * 0.08;
  const r = starRadius * (1.8 + planet.orbitFactor);
  return {
    x: sx + Math.cos(a) * r,
    y: sy + Math.sin(a) * r * 0.9,
    radius: 3.5 + Math.min(5, planet.moons.length * 0.6),
  };
}

export function moonPositionInPlanet(
  px: number,
  py: number,
  planetRadius: number,
  moon: CosmicMoon,
  time: number,
): { x: number; y: number; radius: number } {
  const a = (moon.slot / Math.max(1, 6)) * Math.PI * 2 + time * 0.15 + moon.slot;
  const r = planetRadius * (1.6 + moon.slot * 0.12);
  return {
    x: px + Math.cos(a) * r,
    y: py + Math.sin(a) * r * 0.85,
    radius: 2 + moon.activity * 2,
  };
}

export function findGalaxy(scene: CosmicScene, id: string): CosmicGalaxy | undefined {
  return scene.galaxies.find((g) => g.id === id);
}

export function findStar(galaxy: CosmicGalaxy, starId: string): CosmicStar | undefined {
  return galaxy.stars.find((s) => s.id === starId);
}

export function findPlanet(star: CosmicStar, planetId: string): CosmicPlanet | undefined {
  return star.planets.find((p) => p.id === planetId);
}

export function buildCosmicHits(
  scene: CosmicScene,
  cx: number,
  cy: number,
  baseRadius: number,
  time: number,
  dockT: number,
  zoomLevel: ZoomLevel,
  focusGalaxyId: string | null,
  focusStarId: string | null,
  zoomGalaxyT: number,
): CosmicHit[] {
  const hits: CosmicHit[] = [];

  if (zoomLevel === "cosmos") {
    for (const galaxy of scene.galaxies) {
      const pos = galaxyPosition(cx, cy, baseRadius, galaxy, time, dockT);
      hits.push({
        id: `galaxy-${galaxy.id}`,
        x: pos.x,
        y: pos.y,
        worldX: pos.x,
        worldY: pos.y,
        r: Math.max(28, galaxy.radius * 0.75),
        kind: "galaxy",
        label: galaxy.label,
        galaxyId: galaxy.id,
      });
    }
    return hits;
  }

  const galaxy = focusGalaxyId ? findGalaxy(scene, focusGalaxyId) : scene.galaxies[0];
  if (!galaxy) return hits;

  const gPos = galaxyPosition(cx, cy, baseRadius, galaxy, time, dockT);
  const gx = gPos.x + (cx - gPos.x) * zoomGalaxyT;
  const gy = gPos.y + (cy - gPos.y) * zoomGalaxyT;

  if (zoomLevel === "galaxy") {
    for (const star of galaxy.stars) {
      const pos = starPositionInGalaxy(gx, gy, galaxy, star, time, zoomGalaxyT);
      hits.push({
        id: `star-${star.id}`,
        x: pos.x,
        y: pos.y,
        worldX: pos.x,
        worldY: pos.y,
        r: Math.max(14, pos.radius + 6),
        kind: "star",
        label: star.label,
        galaxyId: galaxy.id,
        starId: star.id,
      });
      for (const planet of star.planets) {
        const ppos = planetPositionInStar(pos.x, pos.y, pos.radius, planet, time);
        hits.push({
          id: `planet-${planet.id}`,
          x: ppos.x,
          y: ppos.y,
          worldX: ppos.x,
          worldY: ppos.y,
          r: Math.max(12, ppos.radius + 5),
          kind: "planet",
          label: planet.label,
          galaxyId: galaxy.id,
          starId: star.id,
          planetId: planet.id,
        });
      }
    }
    return hits;
  }

  const star = focusStarId
    ? findStar(galaxy, focusStarId)
    : galaxy.stars[0];
  if (!star) return hits;

  const spos = starPositionInGalaxy(gx, gy, galaxy, star, time, 1);
  for (const planet of star.planets) {
    const ppos = planetPositionInStar(spos.x, spos.y, spos.radius, planet, time);
    for (const moon of planet.moons) {
      const mpos = moonPositionInPlanet(ppos.x, ppos.y, ppos.radius, moon, time);
      hits.push({
        id: moon.id,
        x: mpos.x,
        y: mpos.y,
        worldX: mpos.x,
        worldY: mpos.y,
        r: Math.max(10, mpos.radius + 4),
        kind: "moon",
        label: moon.title,
        galaxyId: galaxy.id,
        starId: star.id,
        planetId: planet.id,
        memoryId: moon.memoryId,
      });
    }
  }

  return hits;
}