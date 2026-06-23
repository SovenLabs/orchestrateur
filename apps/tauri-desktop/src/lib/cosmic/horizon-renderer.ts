import { ringCountForDepth } from "./nuance";

const PALETTE = [
  { r: 200, g: 168, b: 208 },
  { r: 160, g: 196, b: 224 },
  { r: 216, g: 176, b: 192 },
  { r: 232, g: 208, b: 176 },
  { r: 168, g: 208, b: 208 },
];

export type HorizonParams = {
  cx: number;
  cy: number;
  baseRadius: number;
  depth: number;
  time: number;
  dockT: number;
  scale: number;
  connected: boolean;
};

function lerpColor(a: (typeof PALETTE)[0], b: (typeof PALETTE)[0], t: number) {
  return {
    r: a.r + (b.r - a.r) * t,
    g: a.g + (b.g - a.g) * t,
    b: a.b + (b.b - a.b) * t,
  };
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

export function drawHorizon(ctx: CanvasRenderingContext2D, p: HorizonParams): void {
  const rings = ringCountForDepth(p.depth);
  const holeR = p.baseRadius * 0.16 * p.scale;
  const photonPulse = p.connected ? 1 + Math.sin(p.time * 2.8) * 0.04 : 1;
  const photonR = holeR * (1.35 + Math.sin(p.time * 2.2) * 0.03) * photonPulse;

  const outerGlow = ctx.createRadialGradient(
    p.cx,
    p.cy,
    photonR * 0.6,
    p.cx,
    p.cy,
    p.baseRadius * 1.35 * p.scale,
  );
  const glowStrength = 0.08 + p.depth * 0.16;
  outerGlow.addColorStop(0, `rgba(160,196,224,${glowStrength * 0.35})`);
  outerGlow.addColorStop(0.45, `rgba(200,168,208,${glowStrength * 0.12})`);
  outerGlow.addColorStop(1, "rgba(10,10,18,0)");
  ctx.beginPath();
  ctx.arc(p.cx, p.cy, p.baseRadius * 1.35 * p.scale, 0, Math.PI * 2);
  ctx.fillStyle = outerGlow;
  ctx.fill();

  ctx.beginPath();
  ctx.arc(p.cx, p.cy, photonR, 0, Math.PI * 2);
  ctx.strokeStyle = `rgba(160,196,224,${(p.connected ? 0.45 : 0.22) + p.depth * 0.25})`;
  ctx.lineWidth = lerp(2.2, 1, p.dockT);
  ctx.stroke();

  for (let i = 0; i < rings; i++) {
    const t = i / Math.max(1, rings - 1);
    const c = lerpColor(PALETTE[i % PALETTE.length], PALETTE[(i + 1) % PALETTE.length], t);
    const wobble = Math.sin(p.time * 1.1 + i * 0.65) * lerp(2.2, 0.4, p.dockT);
    const r = p.baseRadius * (0.28 + t * 0.62) * p.scale + wobble;
    const alpha = (0.12 + p.depth * 0.22) * (1 - t * 0.35);
    ctx.beginPath();
    ctx.arc(p.cx, p.cy, r, 0, Math.PI * 2);
    ctx.strokeStyle = `rgba(${c.r},${c.g},${c.b},${alpha})`;
    ctx.lineWidth = lerp(1.2 + t * 1.2, 0.6, p.dockT);
    ctx.stroke();
  }

  if (p.dockT < 0.65) {
    const alpha = (1 - p.dockT) * 0.6;
    ctx.fillStyle = `rgba(160,196,224,${alpha})`;
    ctx.font = "9px Inter, system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.fillText("Orchestrateur", p.cx, p.cy + photonR + 14);
  }
}