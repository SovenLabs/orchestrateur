import { drawCortex } from "./cortex-renderer";
import { drawHorizon } from "./horizon-renderer";

export type AccretionParams = {
  cx: number;
  cy: number;
  baseRadius: number;
  depth: number;
  thinking: boolean;
  time: number;
  dockT: number;
  connected?: boolean;
};

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

export function accretionScale(dockT: number): number {
  return lerp(1, 0.28, Math.max(0, Math.min(1, dockT)));
}

/** Compat tests — délègue à Cortex + Horizon. */
export function drawAccretionDisk(ctx: CanvasRenderingContext2D, p: AccretionParams): void {
  const scale = accretionScale(p.dockT);
  drawHorizon(ctx, {
    cx: p.cx,
    cy: p.cy,
    baseRadius: p.baseRadius,
    depth: p.depth,
    time: p.time,
    dockT: p.dockT,
    scale,
    connected: p.connected ?? true,
  });
  drawCortex(ctx, {
    cx: p.cx,
    cy: p.cy,
    baseRadius: p.baseRadius,
    depth: p.depth,
    thinking: p.thinking,
    time: p.time,
    dockT: p.dockT,
    scale,
  });
}