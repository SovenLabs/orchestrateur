export type CosmicCameraState = {
  panX: number;
  panY: number;
  zoom: number;
  tiltX: number;
  tiltY: number;
};

export const CAMERA_ZOOM_MIN = 0.3;
export const CAMERA_ZOOM_MAX = 5.0;

export function clampZoom(z: number): number {
  return Math.max(CAMERA_ZOOM_MIN, Math.min(CAMERA_ZOOM_MAX, z));
}

/** Normalise delta molette (Windows / trackpad / Firefox). */
export function normalizeWheelDelta(e: { deltaY: number; deltaMode?: number }): number {
  let delta = e.deltaY;
  if (e.deltaMode === 1) delta *= 20;
  else if (e.deltaMode === 2) delta *= 400;
  return delta;
}

/** Facteur multiplicatif : deltaY > 0 = dézoom, deltaY < 0 = zoom. */
export function wheelZoomFactor(deltaY: number): number {
  return Math.exp(-deltaY * 0.0022);
}

/** World → écran (zoom autour du pivot cx,cy). */
export function worldToScreen(
  wx: number,
  wy: number,
  cx: number,
  cy: number,
  cam: CosmicCameraState,
): { x: number; y: number } {
  return {
    x: cx + (wx - cx) * cam.zoom + cam.panX,
    y: cy + (wy - cy) * cam.zoom + cam.panY,
  };
}

/** Écran → monde. */
export function screenToWorld(
  sx: number,
  sy: number,
  cx: number,
  cy: number,
  cam: CosmicCameraState,
): { x: number; y: number } {
  return {
    x: cx + (sx - cx - cam.panX) / cam.zoom,
    y: cy + (sy - cy - cam.panY) / cam.zoom,
  };
}

/** Pan pour centrer un point monde à l'écran. */
export function panToCenterWorld(
  wx: number,
  wy: number,
  cx: number,
  cy: number,
  zoom: number,
): { panX: number; panY: number } {
  return {
    panX: -(wx - cx) * zoom,
    panY: -(wy - cy) * zoom,
  };
}

/** Zoom vers le curseur en conservant le point monde sous la souris. */
export function zoomAtPointer(
  cam: CosmicCameraState,
  cx: number,
  cy: number,
  pointerX: number,
  pointerY: number,
  factor: number,
): CosmicCameraState {
  const world = screenToWorld(pointerX, pointerY, cx, cy, cam);
  const nextZoom = clampZoom(cam.zoom * factor);
  const realFactor = nextZoom / cam.zoom;
  if (Math.abs(realFactor - 1) < 1e-6) return cam;
  return {
    ...cam,
    zoom: nextZoom,
    panX: pointerX - cx - (world.x - cx) * nextZoom,
    panY: pointerY - cy - (world.y - cy) * nextZoom,
  };
}

export function lerpCamera(
  current: CosmicCameraState,
  target: CosmicCameraState,
  t: number,
): CosmicCameraState {
  const k = Math.min(1, t);
  return {
    panX: current.panX + (target.panX - current.panX) * k,
    panY: current.panY + (target.panY - current.panY) * k,
    zoom: current.zoom + (target.zoom - current.zoom) * k,
    tiltX: current.tiltX + (target.tiltX - current.tiltX) * k,
    tiltY: current.tiltY + (target.tiltY - current.tiltY) * k,
  };
}

export function defaultCamera(): CosmicCameraState {
  return { panX: 0, panY: 0, zoom: 1, tiltX: 0, tiltY: 0 };
}