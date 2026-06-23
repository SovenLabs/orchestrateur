import {
  galaxyPosition,
  moonPositionInPlanet,
  planetPositionInStar,
  starPositionInGalaxy,
  findGalaxy,
  findPlanet,
  findStar,
  type CosmicScene,
  type WormholeEdge,
  type ZoomLevel,
} from "./cosmic-model";

export type WormholeRenderParams = {
  cx: number;
  cy: number;
  baseRadius: number;
  time: number;
  dockT: number;
  zoomLevel: ZoomLevel;
  focusGalaxyId: string | null;
  zoomGalaxyT: number;
};

function bodyPoint(
  scene: CosmicScene,
  memoryId: string,
  p: WormholeRenderParams,
): { x: number; y: number } | null {
  const placement = scene.placementByMemory.get(memoryId);
  if (!placement) return null;

  const galaxy = findGalaxy(scene, placement.galaxyId);
  if (!galaxy) return null;

  if (p.zoomLevel === "cosmos") {
    return galaxyPosition(p.cx, p.cy, p.baseRadius, galaxy, p.time, p.dockT);
  }

  const gPos = galaxyPosition(p.cx, p.cy, p.baseRadius, galaxy, p.time, p.dockT);
  const gx = p.cx + (gPos.x - p.cx) * p.zoomGalaxyT;
  const gy = p.cy + (gPos.y - p.cy) * p.zoomGalaxyT;

  const star = findStar(galaxy, placement.starId);
  if (!star) return { x: gx, y: gy };

  const spos = starPositionInGalaxy(gx, gy, galaxy, star, p.time, p.zoomGalaxyT);
  const planet = findPlanet(star, placement.planetId);
  if (!planet) return { x: spos.x, y: spos.y };

  const ppos = planetPositionInStar(spos.x, spos.y, spos.radius, planet, p.time);
  const moon = planet.moons.find((m) => m.memoryId === memoryId);
  if (!moon) return { x: ppos.x, y: ppos.y };

  return moonPositionInPlanet(ppos.x, ppos.y, ppos.radius, moon, p.time);
}

export function drawWormholes(
  ctx: CanvasRenderingContext2D,
  scene: CosmicScene,
  edges: WormholeEdge[],
  p: WormholeRenderParams,
): void {
  const visibility = Math.max(0.1, 1 - p.dockT * 0.8);
  if (visibility < 0.08) return;

  for (const edge of edges) {
    if (p.zoomLevel === "galaxy" && p.focusGalaxyId && edge.fromGalaxyId !== p.focusGalaxyId && edge.toGalaxyId !== p.focusGalaxyId) {
      continue;
    }

    const a = bodyPoint(scene, edge.fromMemoryId, p);
    const b = bodyPoint(scene, edge.toMemoryId, p);
    if (!a || !b) continue;

    const crossGalaxy = edge.fromGalaxyId !== edge.toGalaxyId;
    const cosmosFade = p.zoomLevel === "cosmos" ? 0.35 : 1.0;
    const alpha = edge.strength * (crossGalaxy ? 0.28 : 0.14) * visibility * cosmosFade;
    const mx = (a.x + b.x) / 2 + Math.sin(p.time * 0.7 + edge.strength * 6) * 12;
    const my = (a.y + b.y) / 2 + Math.cos(p.time * 0.5) * 10;

    ctx.setLineDash(crossGalaxy ? [5, 6] : [3, 5]);
    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.quadraticCurveTo(mx, my, b.x, b.y);
    ctx.strokeStyle = `rgba(168,120,220,${alpha})`;
    ctx.lineWidth = 0.7 + edge.strength * 0.8;
    ctx.stroke();
    ctx.setLineDash([]);
  }
}