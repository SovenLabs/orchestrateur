import { COSMIC_PALETTE } from "$lib/cosmic/cosmic-palette";
import type { BlackholeLayout } from "$lib/cosmic/blackhole-layout";
import type { CosmicCameraState } from "$lib/cosmic/cosmic-camera";
import type { GalaxyInstanceInput } from "$lib/cosmic/webgl/galaxy/galaxy-instanced";
import type { RenderPresetConfig } from "$lib/cosmic/render-preset";
import type { ZoomLevel } from "$lib/cosmic/cosmic-model";

export type CosmicUniformInput = {
  layout: BlackholeLayout;
  width: number;
  height: number;
  time: number;
  nuanceDepth: number;
  dockT: number;
  scale: number;
  connected: boolean;
  thinking: boolean;
  camera: CosmicCameraState;
  presetConfig: RenderPresetConfig;
  galaxyInput?: GalaxyInstanceInput & { zoomLevel: ZoomLevel };
};

export type CosmicUniforms = {
  resolution: [number, number];
  time: number;
  bhCenter: [number, number];
  bhRadius: number;
  activity: number;
  dockT: number;
  connected: number;
  thinking: number;
  coreTint: [number, number, number];
  cameraPan: [number, number];
  cameraZoom: number;
  cameraTilt: [number, number];
};

const CORE_TINT: [number, number, number] = [...COSMIC_PALETTE.mauve];

export function buildCosmicUniforms(input: CosmicUniformInput): CosmicUniforms {
  const holeR = input.layout.baseRadius * 0.16 * input.scale * input.camera.zoom;
  const thinkingBoost = input.thinking ? 0.18 : 0;
  const activity = Math.min(2.8, 0.35 + input.nuanceDepth * 1.4 + thinkingBoost);
  const { cx, cy } = input.layout;
  const cam = input.camera;

  return {
    resolution: [input.width, input.height],
    time: input.time,
    bhCenter: [cx, cy],
    bhRadius: Math.max(8, holeR),
    activity,
    dockT: input.dockT,
    connected: input.connected ? 1 : 0,
    thinking: input.thinking ? 1 : 0,
    coreTint: CORE_TINT,
    cameraPan: [cam.panX, cam.panY],
    cameraZoom: cam.zoom,
    cameraTilt: [cam.tiltX, cam.tiltY],
  };
}