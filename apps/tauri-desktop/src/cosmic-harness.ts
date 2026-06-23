import { defaultCamera } from "$lib/cosmic/cosmic-camera";
import { computeBlackholeLayout } from "$lib/cosmic/blackhole-layout";
import { resolvePresetConfig } from "$lib/cosmic/render-preset";
import { createCosmicGl } from "$lib/cosmic/webgl/cosmic-gl";

const params = new URLSearchParams(location.search);
const preset = params.get("preset") === "eco" ? "eco" : "cinema";
const time = Number(params.get("time") ?? "12.5");
const dock = Number(params.get("dock") ?? "0");

const canvas = document.getElementById("gl") as HTMLCanvasElement | null;
if (!canvas) {
  document.body.dataset.error = "no-canvas";
  document.body.dataset.ready = "1";
} else {
  try {
    const gl = createCosmicGl(canvas);
    if (!gl.available) {
      document.body.dataset.error = "webgl-unavailable";
      document.body.dataset.ready = "1";
    } else {
      const width = 640;
      const height = 480;
      gl.resize(width, height, 1);

      const config = resolvePresetConfig(preset, {
        webgl2: true,
        floatFramebuffer: true,
        floatBlend: true,
        floatLinear: true,
      });

      const layout = computeBlackholeLayout(width, height, dock);
      gl.render({
        layout,
        width,
        height,
        time,
        nuanceDepth: 0.55,
        dockT: dock,
        scale: 1,
        connected: true,
        thinking: false,
        camera: defaultCamera(),
        presetConfig: config,
      });
      document.body.dataset.ready = "1";
    }
  } catch (e) {
    document.body.dataset.error = e instanceof Error ? e.message : "harness-failed";
    document.body.dataset.ready = "1";
  }
}