import {
  galaxyPosition,
  moonPositionInPlanet,
  planetPositionInStar,
  starPositionInGalaxy,
  findGalaxy,
  type CosmicGalaxy,
  type CosmicScene,
  type ZoomLevel,
} from "./cosmic-model";

const GALAXY_TINTS: Record<string, { r: number; g: number; b: number }> = {
  cognition: { r: 200, g: 168, b: 208 },
  memoire: { r: 160, g: 196, b: 224 },
  technique: { r: 148, g: 200, b: 212 },
  interface: { r: 216, g: 184, b: 168 },
  strategie: { r: 192, g: 160, b: 200 },
  architecture: { r: 176, g: 208, b: 192 },
  histoire: { r: 208, g: 192, b: 160 },
};

function tint(galaxyId: string) {
  return GALAXY_TINTS[galaxyId] ?? { r: 170, g: 190, b: 220 };
}

export type GalaxyRenderParams = {
  cx: number;
  cy: number;
  baseRadius: number;
  time: number;
  dockT: number;
  zoomLevel: ZoomLevel;
  focusGalaxyId: string | null;
  focusStarId: string | null;
  zoomGalaxyT: number;
  /** Galaxies cosmos rendues en WebGL — ne pas redessiner les sprites 2D. */
  skipCosmosSprites?: boolean;
  /** Corps 3D Three.js — ne pas redessiner sphères 2D (labels conservés). */
  skipBodies2d?: boolean;
};

function drawGalaxySprite(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  galaxy: CosmicGalaxy,
  visibility: number,
  showLabels: boolean,
): void {
  const c = tint(galaxy.id);
  const r = galaxy.isNebula ? galaxy.radius * 0.35 : galaxy.radius * 0.75;

  const halo = ctx.createRadialGradient(x, y, 0, x, y, r * 2.2);
  halo.addColorStop(0, `rgba(255,255,255,${0.08 * visibility})`);
  halo.addColorStop(0.3, `rgba(${c.r},${c.g},${c.b},${0.12 * visibility})`);
  halo.addColorStop(1, "rgba(0,0,0,0)");
  ctx.fillStyle = halo;
  ctx.beginPath();
  ctx.arc(x, y, r * 2.2, 0, Math.PI * 2);
  ctx.fill();

  if (!galaxy.isNebula) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(galaxy.angle + 0.3);
    for (let i = 0; i < 2; i++) {
      const a = (i / 2) * Math.PI;
      ctx.beginPath();
      ctx.ellipse(Math.cos(a) * r * 0.12, Math.sin(a) * r * 0.08, r * 0.9, r * 0.22, a, 0, Math.PI * 2);
      ctx.strokeStyle = `rgba(${c.r},${c.g},${c.b},${0.06 * visibility})`;
      ctx.lineWidth = 0.8;
      ctx.stroke();
    }
    ctx.restore();
  }

  const core = ctx.createRadialGradient(x, y, 0, x, y, r * 0.6);
  core.addColorStop(0, `rgba(255,250,245,${0.35 * visibility})`);
  core.addColorStop(0.5, `rgba(${c.r},${c.g},${c.b},${0.15 * visibility})`);
  core.addColorStop(1, "rgba(0,0,0,0)");
  ctx.fillStyle = core;
  ctx.beginPath();
  ctx.arc(x, y, r * 0.6, 0, Math.PI * 2);
  ctx.fill();

  if (showLabels) {
    ctx.save();
    ctx.shadowColor = "rgba(0,0,0,0.95)";
    ctx.shadowBlur = 8;
    ctx.fillStyle = `rgba(230,238,248,${0.7 * visibility})`;
    ctx.font = "9px Inter, system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.fillText(galaxy.label, x, y - r - 5);
    ctx.fillStyle = `rgba(160,196,224,${0.55 * visibility})`;
    ctx.font = "8px Inter, system-ui, sans-serif";
    ctx.fillText(`${galaxy.memoryCount}`, x, y + r + 9);
    ctx.restore();
  }
}

function drawIntraLinks(
  ctx: CanvasRenderingContext2D,
  galaxy: CosmicGalaxy,
  gx: number,
  gy: number,
  time: number,
  zoomGalaxyT: number,
  visibility: number,
): void {
  const c = tint(galaxy.id);
  for (const star of galaxy.stars) {
    const spos = starPositionInGalaxy(gx, gy, galaxy, star, time, zoomGalaxyT);
    for (const planet of star.planets) {
      const ppos = planetPositionInStar(spos.x, spos.y, spos.radius, planet, time);
      ctx.beginPath();
      ctx.moveTo(spos.x, spos.y);
      ctx.lineTo(ppos.x, ppos.y);
      ctx.strokeStyle = `rgba(${c.r},${c.g},${c.b},${0.1 * visibility})`;
      ctx.lineWidth = 0.5;
      ctx.stroke();
    }
  }
}

export function drawGalaxyField(
  ctx: CanvasRenderingContext2D,
  scene: CosmicScene,
  p: GalaxyRenderParams,
): void {
  const visibility = Math.max(0.1, 1 - p.dockT * 0.9);
  if (visibility < 0.08) return;

  if (p.zoomLevel === "cosmos") {
    if (!p.skipCosmosSprites) {
      for (const galaxy of scene.galaxies) {
        const pos = galaxyPosition(p.cx, p.cy, p.baseRadius, galaxy, p.time, p.dockT);
        drawGalaxySprite(ctx, pos.x, pos.y, galaxy, visibility * 0.85, true);
      }
    }
    return;
  }

  const galaxy = p.focusGalaxyId ? findGalaxy(scene, p.focusGalaxyId) : scene.galaxies[0];
  if (!galaxy) return;

  const gPos = galaxyPosition(p.cx, p.cy, p.baseRadius, galaxy, p.time, p.dockT);
  const lerpGx = gPos.x + (p.cx - gPos.x) * p.zoomGalaxyT;
  const lerpGy = gPos.y + (p.cy - gPos.y) * p.zoomGalaxyT;

  drawGalaxySprite(
    ctx,
    lerpGx,
    lerpGy,
    { ...galaxy, radius: galaxy.radius * (0.85 + p.zoomGalaxyT * 0.25) },
    visibility * 0.55,
    p.zoomGalaxyT > 0.7,
  );
  if (p.zoomGalaxyT > 0.5) {
    drawIntraLinks(ctx, galaxy, lerpGx, lerpGy, p.time, p.zoomGalaxyT, visibility * 0.4);
  }

  for (const star of galaxy.stars) {
    const spos = starPositionInGalaxy(lerpGx, lerpGy, galaxy, star, p.time, p.zoomGalaxyT);
    const c = tint(galaxy.id);
    const starAlpha = 0.4 * visibility;

    if (!p.skipBodies2d) {
      const grad = ctx.createRadialGradient(spos.x, spos.y, 0, spos.x, spos.y, spos.radius * 2.0);
      grad.addColorStop(0, `rgba(255,248,220,${starAlpha})`);
      grad.addColorStop(0.4, `rgba(${c.r},${c.g},${c.b},${starAlpha * 0.5})`);
      grad.addColorStop(1, "rgba(0,0,0,0)");
      ctx.fillStyle = grad;
      ctx.beginPath();
      ctx.arc(spos.x, spos.y, spos.radius * 2.0, 0, Math.PI * 2);
      ctx.fill();
    }

    const showStarLabel = p.zoomLevel === "body" && p.focusStarId === star.id;
    if (showStarLabel) {
      ctx.save();
      ctx.shadowColor = "rgba(0,0,0,0.85)";
      ctx.shadowBlur = 4;
      ctx.fillStyle = `rgba(255,245,230,${0.75 * visibility})`;
      ctx.font = "9px Inter, system-ui, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(star.label, spos.x, spos.y - spos.radius - 4);
      ctx.restore();
    }

    for (const planet of star.planets) {
      if (p.zoomLevel === "body" && p.focusStarId && p.focusStarId !== star.id) continue;
      const ppos = planetPositionInStar(spos.x, spos.y, spos.radius, planet, p.time);

      if (!p.skipBodies2d) {
        ctx.beginPath();
        ctx.arc(ppos.x, ppos.y, ppos.radius, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(${c.r},${c.g},${c.b},${0.35 * visibility})`;
        ctx.fill();
      }

      if (p.zoomLevel === "body") {
        ctx.fillStyle = `rgba(220,230,240,${0.65 * visibility})`;
        ctx.font = "8px Inter, system-ui, sans-serif";
        ctx.fillText(planet.label, ppos.x, ppos.y - ppos.radius - 3);
      }

      if (p.zoomLevel === "body") {
        for (const moon of planet.moons) {
          const mpos = moonPositionInPlanet(ppos.x, ppos.y, ppos.radius, moon, p.time);
          if (!p.skipBodies2d) {
            ctx.beginPath();
            ctx.arc(mpos.x, mpos.y, mpos.radius, 0, Math.PI * 2);
            ctx.fillStyle = `rgba(230,240,250,${0.75 * visibility})`;
            ctx.fill();
          }
          ctx.fillStyle = `rgba(200,220,235,${0.6 * visibility})`;
          ctx.font = "7px Inter, system-ui, sans-serif";
          ctx.fillText(moon.title, mpos.x, mpos.y + mpos.radius + 7);
        }
      }
    }
  }
}