import {
  findGalaxy,
  findStar,
  galaxyPosition,
  moonPositionInPlanet,
  planetPositionInStar,
  starPositionInGalaxy,
  type CosmicScene,
  type ZoomLevel,
} from "$lib/cosmic/cosmic-model";

export type CelestialBody = {
  id: string;
  kind: "star" | "planet" | "moon";
  x: number;
  y: number;
  radius: number;
  color: [number, number, number];
};

const GALAXY_TINTS: Record<string, [number, number, number]> = {
  cognition: [0.78, 0.66, 0.82],
  memoire: [0.63, 0.77, 0.88],
  technique: [0.58, 0.78, 0.83],
  interface: [0.85, 0.72, 0.66],
  strategie: [0.75, 0.63, 0.78],
  architecture: [0.69, 0.82, 0.75],
  histoire: [0.82, 0.75, 0.63],
};

function tint(galaxyId: string): [number, number, number] {
  return GALAXY_TINTS[galaxyId] ?? [0.67, 0.75, 0.86];
}

export type BodyLayoutInput = {
  scene: CosmicScene;
  cx: number;
  cy: number;
  baseRadius: number;
  time: number;
  dockT: number;
  zoomLevel: ZoomLevel;
  focusGalaxyId: string | null;
  focusStarId: string | null;
  zoomGalaxyT: number;
};

export function buildCelestialBodies(input: BodyLayoutInput): CelestialBody[] {
  const { scene, cx, cy, baseRadius, time, dockT, zoomLevel, focusGalaxyId, focusStarId, zoomGalaxyT } =
    input;

  if (zoomLevel === "cosmos") return [];

  const galaxy = focusGalaxyId ? findGalaxy(scene, focusGalaxyId) : scene.galaxies[0];
  if (!galaxy) return [];

  const gPos = galaxyPosition(cx, cy, baseRadius, galaxy, time, dockT);
  const gx = gPos.x + (cx - gPos.x) * zoomGalaxyT;
  const gy = gPos.y + (cy - gPos.y) * zoomGalaxyT;
  const c = tint(galaxy.id);
  const bodies: CelestialBody[] = [];

  if (zoomLevel === "galaxy") {
    for (const star of galaxy.stars) {
      const spos = starPositionInGalaxy(gx, gy, galaxy, star, time, zoomGalaxyT);
      bodies.push({
        id: `star-${star.id}`,
        kind: "star",
        x: spos.x,
        y: spos.y,
        radius: spos.radius * 1.1,
        color: [1.0, 0.97, 0.9],
      });
      for (const planet of star.planets) {
        const ppos = planetPositionInStar(spos.x, spos.y, spos.radius, planet, time);
        bodies.push({
          id: `planet-${planet.id}`,
          kind: "planet",
          x: ppos.x,
          y: ppos.y,
          radius: ppos.radius,
          color: c,
        });
      }
    }
    return bodies;
  }

  const star = focusStarId ? findStar(galaxy, focusStarId) : galaxy.stars[0];
  if (!star) return [];

  const spos = starPositionInGalaxy(gx, gy, galaxy, star, time, 1);
  bodies.push({
    id: `star-${star.id}`,
    kind: "star",
    x: spos.x,
    y: spos.y,
    radius: spos.radius * 1.35,
    color: [1.0, 0.96, 0.88],
  });

  for (const planet of star.planets) {
    const ppos = planetPositionInStar(spos.x, spos.y, spos.radius, planet, time);
    bodies.push({
      id: `planet-${planet.id}`,
      kind: "planet",
      x: ppos.x,
      y: ppos.y,
      radius: ppos.radius * 1.25,
      color: c,
    });
    for (const moon of planet.moons) {
      const mpos = moonPositionInPlanet(ppos.x, ppos.y, ppos.radius, moon, time);
      bodies.push({
        id: `moon-${moon.id}`,
        kind: "moon",
        x: mpos.x,
        y: mpos.y,
        radius: mpos.radius,
        color: [0.9, 0.94, 1.0],
      });
    }
  }

  return bodies;
}