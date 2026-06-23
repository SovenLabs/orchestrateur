export type CompositeLayerSources = {
  gl?: HTMLCanvasElement | null;
  three?: HTMLCanvasElement | null;
  overlay?: HTMLCanvasElement | null;
  voidColor?: string;
  overlayBlend?: GlobalCompositeOperation;
  overlayAlpha?: number;
};

export function createCompositeCanvas(width: number, height: number, dpr = 1): HTMLCanvasElement {
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.floor(width * dpr));
  canvas.height = Math.max(1, Math.floor(height * dpr));
  canvas.style.width = `${width}px`;
  canvas.style.height = `${height}px`;
  return canvas;
}

export function compositeCosmicLayers(
  target: HTMLCanvasElement,
  cssWidth: number,
  cssHeight: number,
  dpr: number,
  sources: CompositeLayerSources,
): void {
  const w = Math.max(1, Math.floor(cssWidth * dpr));
  const h = Math.max(1, Math.floor(cssHeight * dpr));
  if (target.width !== w || target.height !== h) {
    target.width = w;
    target.height = h;
    target.style.width = `${cssWidth}px`;
    target.style.height = `${cssHeight}px`;
  }

  const ctx = target.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.globalCompositeOperation = "source-over";
  ctx.globalAlpha = 1;
  ctx.fillStyle = sources.voidColor ?? "#0a0a12";
  ctx.fillRect(0, 0, cssWidth, cssHeight);

  if (sources.gl) {
    ctx.drawImage(sources.gl, 0, 0, cssWidth, cssHeight);
  }
  if (sources.three) {
    ctx.drawImage(sources.three, 0, 0, cssWidth, cssHeight);
  }
  if (sources.overlay) {
    const prevOp = ctx.globalCompositeOperation;
    const prevAlpha = ctx.globalAlpha;
    ctx.globalCompositeOperation = sources.overlayBlend ?? "screen";
    ctx.globalAlpha = sources.overlayAlpha ?? 0.82;
    ctx.drawImage(sources.overlay, 0, 0, cssWidth, cssHeight);
    ctx.globalCompositeOperation = prevOp;
    ctx.globalAlpha = prevAlpha;
  }
}