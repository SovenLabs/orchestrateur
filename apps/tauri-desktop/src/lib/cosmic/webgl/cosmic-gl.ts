import {
  createFramebuffer,
  createFullscreenQuad,
  linkProgram,
  mergeFragmentShaders,
} from "$lib/cosmic/webgl/gl-utils";
import {
  createGalaxyInstancedLayer,
  galaxyVisibility,
  type GalaxyInstanceInput,
} from "$lib/cosmic/webgl/galaxy/galaxy-instanced";
import {
  buildCosmicUniforms,
  type CosmicUniformInput,
} from "$lib/cosmic/webgl/uniforms";
import { postFxBitmask, type RenderPresetConfig } from "$lib/cosmic/render-preset";

import { loadBhPrecomputedAssets } from "$lib/cosmic/cosmic-bh-assets";
import {
  createBhGlTextures,
  type BhGlTextures,
} from "$lib/cosmic/webgl/black-hole/cosmic-bh-textures";

import bgVert from "./shaders/cosmic-bg.vert.glsl?raw";
import bgFragCore from "./shaders/cosmic-bg.frag.glsl?raw";
import ebrunetonTrace from "./shaders/ebruneton-trace.glsl?raw";

const bgFrag = mergeFragmentShaders(ebrunetonTrace, bgFragCore);
import bloomVert from "./shaders/bloom.vert.glsl?raw";
import thresholdFrag from "./shaders/post/bloom-threshold.frag.glsl?raw";
import kawaseFrag from "./shaders/post/kawase.frag.glsl?raw";
import compositePostFrag from "./shaders/post/composite-post.frag.glsl?raw";

export type CosmicGl = {
  readonly available: boolean;
  resize: (width: number, height: number, dpr: number) => void;
  render: (input: CosmicUniformInput) => void;
  destroy: () => void;
  getCanvas: () => HTMLCanvasElement;
};

type Fb = { fbo: WebGLFramebuffer; texture: WebGLTexture; width: number; height: number };

export function createCosmicGl(canvas: HTMLCanvasElement): CosmicGl {
  const gl = canvas.getContext("webgl2", {
    alpha: false,
    antialias: false,
    depth: false,
    stencil: false,
    preserveDrawingBuffer: true,
  });

  if (!gl) {
    return {
      available: false,
      resize: () => {},
      render: () => {},
      destroy: () => {},
      getCanvas: () => canvas,
    };
  }

  const glCtx = gl;

  const bgProgram = linkProgram(glCtx, bgVert, bgFrag);
  const thresholdProgram = linkProgram(glCtx, bloomVert, thresholdFrag);
  const kawaseProgram = linkProgram(glCtx, bloomVert, kawaseFrag);
  const postProgram = linkProgram(glCtx, bloomVert, compositePostFrag);
  const quad = createFullscreenQuad(glCtx);
  const galaxyLayer = createGalaxyInstancedLayer(glCtx);

  if (!bgProgram || !thresholdProgram || !kawaseProgram || !postProgram || !quad) {
    console.error("[cosmic-gl] init failed");
    return {
      available: false,
      resize: () => {},
      render: () => {},
      destroy: () => {},
      getCanvas: () => canvas,
    };
  }

  const bgProg = bgProgram;
  const thresholdProg = thresholdProgram;
  const kawaseProg = kawaseProgram;
  const postProg = postProgram;
  const quadData = quad;

  const bgLocs = {
    resolution: glCtx.getUniformLocation(bgProg, "u_resolution"),
    time: glCtx.getUniformLocation(bgProg, "u_time"),
    bhCenter: glCtx.getUniformLocation(bgProg, "u_bh_center"),
    bhRadius: glCtx.getUniformLocation(bgProg, "u_bh_radius"),
    activity: glCtx.getUniformLocation(bgProg, "u_activity"),
    dockT: glCtx.getUniformLocation(bgProg, "u_dock_t"),
    connected: glCtx.getUniformLocation(bgProg, "u_connected"),
    thinking: glCtx.getUniformLocation(bgProg, "u_thinking"),
    coreTint: glCtx.getUniformLocation(bgProg, "u_core_tint"),
    cameraPan: glCtx.getUniformLocation(bgProg, "u_camera_pan"),
    cameraTilt: glCtx.getUniformLocation(bgProg, "u_camera_tilt"),
    cameraZoom: glCtx.getUniformLocation(bgProg, "u_camera_zoom"),
    useEbruneton: glCtx.getUniformLocation(bgProg, "u_use_ebruneton"),
    deflectionTex: glCtx.getUniformLocation(bgProg, "u_deflection_tex"),
    deflectionSize: glCtx.getUniformLocation(bgProg, "u_deflection_size"),
  };

  let bhTextures: BhGlTextures | null = null;
  loadBhPrecomputedAssets().then((assets) => {
    if (!assets.ready || !assets.deflection || !assets.inverseRadius) return;
    bhTextures = createBhGlTextures(glCtx, assets.deflection, assets.inverseRadius);
  });

  const thresholdLocs = {
    source: glCtx.getUniformLocation(thresholdProg, "u_source"),
    threshold: glCtx.getUniformLocation(thresholdProg, "u_threshold"),
  };

  const kawaseLocs = {
    source: glCtx.getUniformLocation(kawaseProg, "u_source"),
    resolution: glCtx.getUniformLocation(kawaseProg, "u_resolution"),
    offset: glCtx.getUniformLocation(kawaseProg, "u_offset"),
    pass: glCtx.getUniformLocation(kawaseProg, "u_pass"),
  };

  const postLocs = {
    scene: glCtx.getUniformLocation(postProg, "u_scene"),
    bloom: glCtx.getUniformLocation(postProg, "u_bloom"),
    bloomIntensity: glCtx.getUniformLocation(postProg, "u_bloomIntensity"),
    time: glCtx.getUniformLocation(postProg, "u_time"),
    activity: glCtx.getUniformLocation(postProg, "u_activity"),
    resolution: glCtx.getUniformLocation(postProg, "u_resolution"),
    fxMask: glCtx.getUniformLocation(postProg, "u_fx_mask"),
  };

  let sceneFb: Fb | null = null;
  let bloomPing: Fb | null = null;
  let bloomPong: Fb | null = null;
  let bloomResult: WebGLTexture | null = null;

  function allocFb(width: number, height: number): Fb | null {
    const fb = createFramebuffer(glCtx, width, height);
    if (!fb) return null;
    return { ...fb, width, height };
  }

  function releaseFb(fb: Fb | null) {
    if (!fb) return;
    glCtx.deleteFramebuffer(fb.fbo);
    glCtx.deleteTexture(fb.texture);
  }

  function ensureFramebuffers(fullW: number, fullH: number, bloomScale: number) {
    const w = Math.max(1, Math.floor(fullW));
    const h = Math.max(1, Math.floor(fullH));
    const bw = Math.max(1, Math.floor(w * bloomScale));
    const bh = Math.max(1, Math.floor(h * bloomScale));
    if (sceneFb?.width === w && sceneFb.height === h && bloomPing?.width === bw) return;

    releaseFb(sceneFb);
    releaseFb(bloomPing);
    releaseFb(bloomPong);

    sceneFb = allocFb(w, h);
    bloomPing = allocFb(bw, bh);
    bloomPong = allocFb(bw, bh);
    bloomResult = null;
  }

  function drawFullscreen(program: WebGLProgram) {
    glCtx.useProgram(program);
    glCtx.bindVertexArray(quadData.vao);
    glCtx.drawArrays(glCtx.TRIANGLE_STRIP, 0, 4);
    glCtx.bindVertexArray(null);
  }

  function bindTex(unit: number, texture: WebGLTexture) {
    glCtx.activeTexture(glCtx.TEXTURE0 + unit);
    glCtx.bindTexture(glCtx.TEXTURE_2D, texture);
  }

  function setBgUniforms(u: ReturnType<typeof buildCosmicUniforms>) {
    glCtx.uniform2f(bgLocs.resolution, u.resolution[0], u.resolution[1]);
    glCtx.uniform1f(bgLocs.time, u.time);
    glCtx.uniform2f(bgLocs.bhCenter, u.bhCenter[0], u.bhCenter[1]);
    glCtx.uniform1f(bgLocs.bhRadius, u.bhRadius);
    glCtx.uniform1f(bgLocs.activity, u.activity);
    glCtx.uniform1f(bgLocs.dockT, u.dockT);
    glCtx.uniform1f(bgLocs.connected, u.connected);
    glCtx.uniform1f(bgLocs.thinking, u.thinking);
    glCtx.uniform3f(bgLocs.coreTint, u.coreTint[0], u.coreTint[1], u.coreTint[2]);
    glCtx.uniform2f(bgLocs.cameraPan, u.cameraPan[0], u.cameraPan[1]);
    glCtx.uniform2f(bgLocs.cameraTilt, u.cameraTilt[0], u.cameraTilt[1]);
    glCtx.uniform1f(bgLocs.cameraZoom, u.cameraZoom);
    const eb = bhTextures !== null;
    glCtx.uniform1f(bgLocs.useEbruneton, eb ? 1 : 0);
    if (eb && bhTextures) {
      bindTex(2, bhTextures.deflection);
      glCtx.uniform1i(bgLocs.deflectionTex, 2);
      glCtx.uniform2f(
        bgLocs.deflectionSize,
        bhTextures.deflectionSize[0],
        bhTextures.deflectionSize[1],
      );
    } else {
      glCtx.uniform1f(bgLocs.useEbruneton, 0);
    }
  }

  function renderScene(u: ReturnType<typeof buildCosmicUniforms>, input: CosmicUniformInput) {
    if (!sceneFb) return;
    glCtx.bindFramebuffer(glCtx.FRAMEBUFFER, sceneFb.fbo);
    glCtx.viewport(0, 0, sceneFb.width, sceneFb.height);
    glCtx.clearColor(0, 0, 0, 1);
    glCtx.clear(glCtx.COLOR_BUFFER_BIT);

    glCtx.useProgram(bgProg);
    setBgUniforms(u);
    drawFullscreen(bgProg);

    const cfg = input.presetConfig;
    if (cfg.galaxyWebGl && galaxyLayer && input.galaxyInput && input.galaxyInput.zoomLevel === "cosmos") {
      const count = galaxyLayer.updateInstances({
        ...input.galaxyInput,
        visibility: galaxyVisibility(input.dockT),
      });
      galaxyLayer.draw(count, {
        resolution: u.resolution,
        time: u.time,
        bhCenter: u.bhCenter,
        cameraPan: u.cameraPan,
        cameraZoom: u.cameraZoom,
      });
    }
  }

  function runBloomChain(config: RenderPresetConfig) {
    if (!sceneFb || !bloomPing || !bloomPong) return;

    glCtx.bindFramebuffer(glCtx.FRAMEBUFFER, bloomPing.fbo);
    glCtx.viewport(0, 0, bloomPing.width, bloomPing.height);
    bindTex(0, sceneFb.texture);
    glCtx.useProgram(thresholdProg);
    glCtx.uniform1i(thresholdLocs.source, 0);
    glCtx.uniform1f(thresholdLocs.threshold, 0.55);
    drawFullscreen(thresholdProg);

    let read = bloomPing;
    let write = bloomPong;
    const passes = Math.max(2, config.kawasePasses);

    for (let i = 0; i < passes; i++) {
      glCtx.bindFramebuffer(glCtx.FRAMEBUFFER, write.fbo);
      glCtx.viewport(0, 0, write.width, write.height);
      bindTex(0, read.texture);
      glCtx.useProgram(kawaseProg);
      glCtx.uniform1i(kawaseLocs.source, 0);
      glCtx.uniform2f(kawaseLocs.resolution, write.width, write.height);
      glCtx.uniform1f(kawaseLocs.offset, 1.0 + i * 0.75);
      glCtx.uniform1i(kawaseLocs.pass, 1);
      drawFullscreen(kawaseProg);
      const tmp = read;
      read = write;
      write = tmp;
    }

    bloomResult = read.texture;
  }

  function compositeToScreen(
    u: ReturnType<typeof buildCosmicUniforms>,
    config: RenderPresetConfig,
    bloomEnabled: boolean,
  ) {
    if (!sceneFb) return;

    glCtx.bindFramebuffer(glCtx.FRAMEBUFFER, null);
    glCtx.viewport(0, 0, canvas.width, canvas.height);

    bindTex(0, sceneFb.texture);
    bindTex(1, bloomEnabled && bloomResult ? bloomResult : sceneFb.texture);

    glCtx.useProgram(postProg);
    glCtx.uniform1i(postLocs.scene, 0);
    glCtx.uniform1i(postLocs.bloom, 1);
    glCtx.uniform1f(postLocs.bloomIntensity, bloomEnabled ? 1.15 + u.activity * 0.3 : 0);
    glCtx.uniform1f(postLocs.time, u.time);
    glCtx.uniform1f(postLocs.activity, u.activity);
    glCtx.uniform2f(postLocs.resolution, canvas.width, canvas.height);
    glCtx.uniform1i(postLocs.fxMask, postFxBitmask(config.postFx));
    drawFullscreen(postProg);
    glCtx.bindTexture(glCtx.TEXTURE_2D, null);
  }

  return {
    available: true,
    getCanvas: () => canvas,

    resize(width: number, height: number, dpr: number) {
      const w = Math.floor(width * dpr);
      const h = Math.floor(height * dpr);
      canvas.width = w;
      canvas.height = h;
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
      glCtx.viewport(0, 0, w, h);
      ensureFramebuffers(w, h, 0.5);
    },

    render(input: CosmicUniformInput) {
      const u = buildCosmicUniforms(input);
      const w = canvas.width;
      const h = canvas.height;
      if (w < 1 || h < 1) return;

      const config = input.presetConfig;
      ensureFramebuffers(w, h, config.bloom ? config.bloomScale : 0.5);

      renderScene(u, input);

      if (config.bloom && sceneFb && bloomPing) {
        runBloomChain(config);
      }

      compositeToScreen(u, config, config.bloom);
    },

    destroy() {
      releaseFb(sceneFb);
      releaseFb(bloomPing);
      releaseFb(bloomPong);
      if (bhTextures) {
        glCtx.deleteTexture(bhTextures.deflection);
        glCtx.deleteTexture(bhTextures.inverseRadius);
      }
      galaxyLayer?.destroy();
      glCtx.deleteProgram(bgProg);
      glCtx.deleteProgram(thresholdProg);
      glCtx.deleteProgram(kawaseProg);
      glCtx.deleteProgram(postProg);
      glCtx.deleteBuffer(quadData.buffer);
      glCtx.deleteVertexArray(quadData.vao);
    },
  };
}