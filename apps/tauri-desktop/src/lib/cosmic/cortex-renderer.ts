export type CortexParams = {
  cx: number;
  cy: number;
  baseRadius: number;
  depth: number;
  thinking: boolean;
  time: number;
  dockT: number;
  scale: number;
};

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

export function drawCortex(ctx: CanvasRenderingContext2D, p: CortexParams): void {
  const pulse = p.thinking ? 1 + Math.sin(p.time * 3.5) * 0.05 : 1;
  const holeR = p.baseRadius * 0.16 * p.scale * pulse;

  const grad = ctx.createRadialGradient(p.cx, p.cy, 0, p.cx, p.cy, holeR * 2.8);
  grad.addColorStop(0, "rgba(0,0,0,1)");
  grad.addColorStop(0.42, "rgba(2,2,10,0.98)");
  grad.addColorStop(0.72, "rgba(8,8,18,0.55)");
  grad.addColorStop(1, "rgba(10,10,18,0)");
  ctx.beginPath();
  ctx.arc(p.cx, p.cy, holeR * 2.8, 0, Math.PI * 2);
  ctx.fillStyle = grad;
  ctx.fill();

  const glow = 0.06 + p.depth * 0.14;
  const innerGlow = ctx.createRadialGradient(p.cx, p.cy, holeR * 0.2, p.cx, p.cy, holeR * 1.6);
  innerGlow.addColorStop(0, `rgba(200,168,208,${glow})`);
  innerGlow.addColorStop(1, "rgba(10,10,18,0)");
  ctx.beginPath();
  ctx.arc(p.cx, p.cy, holeR * 1.6, 0, Math.PI * 2);
  ctx.fillStyle = innerGlow;
  ctx.fill();

  if (p.dockT < 0.7) {
    const alpha = (1 - p.dockT * 0.85) * 0.75;
    ctx.fillStyle = `rgba(200,168,208,${alpha})`;
    ctx.font = "11px Inter, system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.fillText("Cortex", p.cx, p.cy + holeR * 0.35);
  }
}