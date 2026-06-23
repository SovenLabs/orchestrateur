import {
  probeCosmicCapabilities,
  type CosmicCapabilities,
} from "$lib/cosmic/cosmic-capabilities";
import {
  resolvePresetConfig,
  type RenderPreset,
  type RenderPresetConfig,
} from "$lib/cosmic/render-preset";

function readReducedMotion(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

class CosmicRenderStore {
  preset = $state<RenderPreset>("cinema");
  capabilities = $state<CosmicCapabilities>({
    webgl2: false,
    floatFramebuffer: false,
    floatBlend: false,
    floatLinear: false,
  });
  reducedMotion = $state(readReducedMotion());

  readonly effectivePreset = $derived.by((): RenderPreset => {
    if (this.reducedMotion) return "eco";
    if (!this.capabilities.floatFramebuffer) return "eco";
    return this.preset;
  });

  readonly config = $derived.by((): RenderPresetConfig => {
    return resolvePresetConfig(this.effectivePreset, this.capabilities);
  });

  initFromGl(gl: WebGL2RenderingContext | null) {
    this.capabilities = probeCosmicCapabilities(gl);
  }

  setPreset(preset: RenderPreset) {
    this.preset = preset;
  }

  togglePreset() {
    this.preset = this.effectivePreset === "cinema" ? "eco" : "cinema";
  }

  syncReducedMotion() {
    this.reducedMotion = readReducedMotion();
  }
}

export const cosmicRenderStore = new CosmicRenderStore();