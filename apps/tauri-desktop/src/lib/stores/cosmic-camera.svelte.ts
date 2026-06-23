import {
  clampZoom,
  defaultCamera,
  panToCenterWorld,
  wheelZoomFactor,
  zoomAtPointer,
  lerpCamera,
  type CosmicCameraState,
} from "$lib/cosmic/cosmic-camera";

class CosmicCameraStore {
  cam = $state<CosmicCameraState>(defaultCamera());
  target = $state<CosmicCameraState>(defaultCamera());

  private dragging = false;
  private lastX = 0;
  private lastY = 0;

  step(dt: number, reducedMotion: boolean): void {
    const speed = reducedMotion ? 14 : 5.5;
    this.cam = lerpCamera(this.cam, this.target, Math.min(1, dt * speed));
  }

  reset(): void {
    const d = defaultCamera();
    this.cam = { ...d };
    this.target = { ...d };
  }

  onPointerDown(x: number, y: number, button: number): void {
    if (button !== 0 && button !== 1) return;
    this.dragging = true;
    this.lastX = x;
    this.lastY = y;
    this.target = { ...this.target, tiltX: 0, tiltY: 0 };
  }

  onPointerMove(x: number, y: number, width: number, height: number): void {
    if (this.dragging) {
      const dx = x - this.lastX;
      const dy = y - this.lastY;
      this.lastX = x;
      this.lastY = y;
      this.target = {
        ...this.target,
        panX: this.target.panX + dx,
        panY: this.target.panY + dy,
        tiltX: 0,
        tiltY: 0,
      };
      this.cam = { ...this.target };
      return;
    }

    if (this.target.zoom <= 1.15) {
      const nx = (x / width - 0.5) * 2;
      const ny = (y / height - 0.5) * 2;
      this.target = {
        ...this.target,
        tiltX: nx * 0.06,
        tiltY: ny * 0.04,
      };
    }
  }

  onPointerUp(): void {
    this.dragging = false;
  }

  onWheel(deltaY: number, pointerX: number, pointerY: number, cx: number, cy: number): void {
    const factor = wheelZoomFactor(deltaY);
    this.target = zoomAtPointer(this.target, cx, cy, pointerX, pointerY, factor);
  }

  zoomBy(factor: number, cx: number, cy: number, pointerX?: number, pointerY?: number): void {
    const px = pointerX ?? cx;
    const py = pointerY ?? cy;
    this.target = zoomAtPointer(this.target, cx, cy, px, py, factor);
  }

  focusWorld(wx: number, wy: number, cx: number, cy: number, zoom = 2.2): void {
    const z = clampZoom(zoom);
    const pan = panToCenterWorld(wx, wy, cx, cy, z);
    this.target = { panX: pan.panX, panY: pan.panY, zoom: z, tiltX: 0, tiltY: 0 };
  }
}

export const cosmicCameraStore = new CosmicCameraStore();