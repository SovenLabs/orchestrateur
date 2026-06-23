<script lang="ts">
  import { onMount } from "svelte";
  import { accretionScale } from "$lib/cosmic/accretion-renderer";
  import { drawAgentBelt } from "$lib/cosmic/agent-belt-renderer";
  import { normalizeWheelDelta, worldToScreen } from "$lib/cosmic/cosmic-camera";
  import { computeBlackholeLayout } from "$lib/cosmic/blackhole-layout";
  import { drawCortex } from "$lib/cosmic/cortex-renderer";
  import { buildCosmicHits, buildCosmicScene, type CosmicHit } from "$lib/cosmic/cosmic-model";
  import { drawGalaxyField } from "$lib/cosmic/galaxy-renderer";
  import { drawHorizon } from "$lib/cosmic/horizon-renderer";
  import { computeNuanceDepth } from "$lib/cosmic/nuance";
  import { createParticleSystem } from "$lib/cosmic/particle-system";
  import { drawWormholes } from "$lib/cosmic/wormhole-renderer";
  import {
    compositeCosmicLayers,
    createCompositeCanvas,
  } from "$lib/cosmic/cosmic-composite";
  import { COSMIC_PALETTE } from "$lib/cosmic/cosmic-palette";
  import { effectiveDpr } from "$lib/cosmic/cosmic-capabilities";
  import { createCosmicPlanetLayer, type PlanetLayer } from "$lib/cosmic/three/CosmicPlanetLayer";
  import { createCosmicGl, type CosmicGl } from "$lib/cosmic/webgl/cosmic-gl";
  import { blackholeStore } from "$lib/stores/blackhole.svelte";
  import { cosmicRenderStore } from "$lib/stores/cosmic-render.svelte";
  import { cosmicCameraStore } from "$lib/stores/cosmic-camera.svelte";
  import { cosmicStore } from "$lib/stores/cosmic-store.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { agentsStore } from "$lib/stores/agents.svelte";

  let {
    class: className = "",
    onHits,
    exportCanvasRef = $bindable<HTMLCanvasElement | undefined>(),
  }: {
    class?: string;
    onHits?: (hits: CosmicHit[]) => void;
    exportCanvasRef?: HTMLCanvasElement | undefined;
  } = $props();

  let glCanvas: HTMLCanvasElement | undefined = $state();
  let threeCanvas: HTMLCanvasElement | undefined = $state();
  let overlayCanvas: HTMLCanvasElement | undefined = $state();
  let container: HTMLDivElement | undefined = $state();
  let raf = 0;
  let lastTime = 0;
  let dockT = 0;
  let zoomGalaxyT = 0;
  const particles = createParticleSystem();
  let lastBurst = 0;
  let lastMemorySeed = 0;
  let reducedMotion = false;
  let cosmicGl: CosmicGl | null = null;
  let planetLayer: PlanetLayer | null = null;
  let compositeCanvas: HTMLCanvasElement | undefined = $state();
  let useWebGl = false;
  let useBodies3d = false;

  const targetDockT = $derived(blackholeStore.state === "docked" ? 1 : 0);
  const targetZoomGalaxyT = $derived(cosmicStore.zoomLevel === "cosmos" ? 0 : 1);
  const interactive = $derived(blackholeStore.state !== "docked");

  function readScene() {
    const memories = connectionStore.memories;
    const linkTotal =
      memories.length > 0
        ? memories.reduce((sum, m) => sum + m.backlink_count, 0)
        : connectionStore.memoryTotal * 4;
    const nuanceDepth = computeNuanceDepth(connectionStore.chatMessages, connectionStore.eventLog, {
      total: connectionStore.memoryTotal,
      linkTotal,
    });
    const scene = buildCosmicScene(memories, cosmicStore.placementCache);
    return {
      scene,
      nuanceDepth,
      memoryLoaded: memories.length,
      thinking: blackholeStore.thinking,
      connected: connectionStore.status === "connected",
    };
  }

  function resize() {
    if (!container) return;
    cosmicRenderStore.syncReducedMotion();
    const dpr = effectiveDpr(window.devicePixelRatio || 1, cosmicRenderStore.config.dprCap);
    const { width, height } = container.getBoundingClientRect();

    if (useWebGl && cosmicGl) {
      cosmicGl.resize(width, height, dpr);
    }

    if (useBodies3d && planetLayer) {
      planetLayer.resize(width, height, dpr);
    }

    if (overlayCanvas) {
      overlayCanvas.width = Math.floor(width * dpr);
      overlayCanvas.height = Math.floor(height * dpr);
      overlayCanvas.style.width = `${width}px`;
      overlayCanvas.style.height = `${height}px`;
      const ctx = overlayCanvas.getContext("2d");
      if (ctx) ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    }

    if (!compositeCanvas) {
      compositeCanvas = createCompositeCanvas(width, height, dpr);
    }
    exportCanvasRef = compositeCanvas;
  }

  function pointerPos(e: { clientX: number; clientY: number }) {
    if (!container) return { x: 0, y: 0 };
    const rect = container.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
  }

  function onPointerDown(e: PointerEvent) {
    if (!interactive || !container) return;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    const p = pointerPos(e);
    cosmicCameraStore.onPointerDown(p.x, p.y, e.button);
  }

  function onPointerMove(e: PointerEvent) {
    if (!interactive || !container) return;
    const p = pointerPos(e);
    const { width, height } = container.getBoundingClientRect();
    cosmicCameraStore.onPointerMove(p.x, p.y, width, height);
  }

  function onPointerUp(e: PointerEvent) {
    cosmicCameraStore.onPointerUp();
    try {
      (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    } catch {
      /* ignore */
    }
  }

  function onWheel(e: WheelEvent) {
    if (!interactive || !container) return;
    e.preventDefault();
    e.stopPropagation();
    const p = pointerPos(e);
    const { width, height } = container.getBoundingClientRect();
    const layout = computeBlackholeLayout(width, height, dockT);
    cosmicCameraStore.onWheel(normalizeWheelDelta(e), p.x, p.y, layout.cx, layout.cy);
  }

  function onDoubleClick() {
    if (!interactive || !container) return;
    cosmicCameraStore.reset();
    cosmicStore.resetZoom();
  }

  function applyCameraTransform(
    ctx: CanvasRenderingContext2D,
    cx: number,
    cy: number,
  ) {
    const cam = cosmicCameraStore.cam;
    ctx.translate(cx + cam.panX, cy + cam.panY);
    ctx.scale(cam.zoom, cam.zoom);
    ctx.translate(-cx, -cy);
  }

  function loop(now: number) {
    if (!container) {
      raf = requestAnimationFrame(loop);
      return;
    }

    const overlayCtx = overlayCanvas?.getContext("2d");
    if (!useWebGl && (!overlayCtx || !overlayCanvas)) {
      raf = requestAnimationFrame(loop);
      return;
    }

    if (document.hidden) {
      raf = requestAnimationFrame(loop);
      return;
    }

    const dt = Math.min(0.05, (now - lastTime) / 1000 || 0.016);
    lastTime = now;
    const { width, height } = container.getBoundingClientRect();

    cosmicCameraStore.step(dt, reducedMotion);

    const lerpSpeed = reducedMotion ? 12 : 5.5;
    dockT += (targetDockT - dockT) * Math.min(1, dt * lerpSpeed);
    if (Math.abs(targetDockT - dockT) < 0.002) dockT = targetDockT;

    zoomGalaxyT += (targetZoomGalaxyT - zoomGalaxyT) * Math.min(1, dt * 6);
    if (Math.abs(targetZoomGalaxyT - zoomGalaxyT) < 0.002) zoomGalaxyT = targetZoomGalaxyT;

    const layout = computeBlackholeLayout(width, height, dockT);
    const time = now / 1000;
    const frame = readScene();
    const scale = accretionScale(dockT);
    const cam = cosmicCameraStore.cam;

    const presetConfig = cosmicRenderStore.config;
    const showBodies3d =
      useBodies3d &&
      planetLayer &&
      presetConfig.bodies3d &&
      cosmicStore.zoomLevel !== "cosmos";

    if (showBodies3d && planetLayer) {
      planetLayer.render({
        scene: frame.scene,
        cx: layout.cx,
        cy: layout.cy,
        baseRadius: layout.baseRadius,
        time,
        dockT,
        zoomLevel: cosmicStore.zoomLevel,
        focusGalaxyId: cosmicStore.focusGalaxyId,
        focusStarId: cosmicStore.focusStarId,
        zoomGalaxyT,
        camera: cam,
      });
    } else if (planetLayer) {
      planetLayer.clear();
    }

    if (useWebGl && cosmicGl) {
      cosmicGl.render({
        layout,
        width,
        height,
        time,
        nuanceDepth: frame.nuanceDepth,
        dockT,
        scale,
        connected: frame.connected,
        thinking: frame.thinking,
        camera: cam,
        presetConfig,
        galaxyInput: presetConfig.galaxyWebGl
          ? {
              scene: frame.scene,
              cx: layout.cx,
              cy: layout.cy,
              baseRadius: layout.baseRadius,
              time,
              dockT,
              visibility: 1,
              zoomLevel: cosmicStore.zoomLevel,
            }
          : undefined,
      });
    }

    if (overlayCtx && overlayCanvas) {
      overlayCtx.clearRect(0, 0, width, height);
      overlayCtx.save();
      applyCameraTransform(overlayCtx, layout.cx, layout.cy);

      if (!useWebGl) {
        drawHorizon(overlayCtx, {
          cx: layout.cx,
          cy: layout.cy,
          baseRadius: layout.baseRadius,
          depth: frame.nuanceDepth,
          time,
          dockT,
          scale,
          connected: frame.connected,
        });
      }

      const prevComposite = overlayCtx.globalCompositeOperation;
      const prevAlpha = overlayCtx.globalAlpha;
      if (useWebGl) {
        overlayCtx.globalCompositeOperation = "screen";
        overlayCtx.globalAlpha = 0.82;
      }

      drawGalaxyField(overlayCtx, frame.scene, {
        cx: layout.cx,
        cy: layout.cy,
        baseRadius: layout.baseRadius,
        time,
        dockT,
        zoomLevel: cosmicStore.zoomLevel,
        focusGalaxyId: cosmicStore.focusGalaxyId,
        focusStarId: cosmicStore.focusStarId,
        zoomGalaxyT,
        skipCosmosSprites: useWebGl && presetConfig.galaxyWebGl,
        skipBodies2d: !!showBodies3d,
      });

      drawWormholes(overlayCtx, frame.scene, frame.scene.wormholes, {
        cx: layout.cx,
        cy: layout.cy,
        baseRadius: layout.baseRadius,
        time,
        dockT,
        zoomLevel: cosmicStore.zoomLevel,
        focusGalaxyId: cosmicStore.focusGalaxyId,
        zoomGalaxyT,
      });

      drawAgentBelt(overlayCtx, {
        cx: layout.cx,
        cy: layout.cy,
        baseRadius: layout.baseRadius,
        time,
        dockT,
        scale,
        agents: agentsStore.agents,
      });

      if (!useWebGl) {
        drawCortex(overlayCtx, {
          cx: layout.cx,
          cy: layout.cy,
          baseRadius: layout.baseRadius,
          depth: frame.nuanceDepth,
          thinking: frame.thinking,
          time,
          dockT,
          scale,
        });
      }

      overlayCtx.globalCompositeOperation = prevComposite;
      overlayCtx.globalAlpha = prevAlpha;

      if (!reducedMotion) {
        if (blackholeStore.feedBurstNonce !== lastBurst) {
          lastBurst = blackholeStore.feedBurstNonce;
          particles.emitFeedBurst(
            { x: width * 0.5, y: height - 56 },
            { x: layout.cx, y: layout.cy },
            Math.round(18 + frame.nuanceDepth * 16),
            frame.nuanceDepth,
          );
        }
        if (frame.memoryLoaded > 0 && frame.memoryLoaded !== lastMemorySeed) {
          lastMemorySeed = frame.memoryLoaded;
          particles.emitAmbient(
            Math.min(60, 12 + Math.floor(frame.memoryLoaded / 3)),
            layout.cx,
            layout.cy,
            layout.baseRadius,
            frame.nuanceDepth,
          );
        }
        const memoryBoost = Math.min(1, frame.memoryLoaded / 60);
        const ambientRate = 0.028 + frame.nuanceDepth * 0.04 + memoryBoost * 0.05;
        if (frame.thinking && Math.random() < ambientRate * 2) {
          particles.emitAmbient(1, layout.cx, layout.cy, layout.baseRadius, frame.nuanceDepth);
        } else if (Math.random() < ambientRate) {
          particles.emitAmbient(1, layout.cx, layout.cy, layout.baseRadius, frame.nuanceDepth);
        }
        const pull = 0.014 + (1 - dockT) * 0.012 + memoryBoost * 0.006;
        particles.step(dt, { x: layout.cx, y: layout.cy }, pull);
        overlayCtx.globalCompositeOperation = "screen";
        overlayCtx.globalAlpha = 0.55;
        particles.draw(overlayCtx);
        overlayCtx.globalCompositeOperation = prevComposite;
        overlayCtx.globalAlpha = prevAlpha;
      }

      overlayCtx.restore();
    }

    const worldHits = buildCosmicHits(
      frame.scene,
      layout.cx,
      layout.cy,
      layout.baseRadius,
      time,
      dockT,
      cosmicStore.zoomLevel,
      cosmicStore.focusGalaxyId,
      cosmicStore.focusStarId,
      zoomGalaxyT,
    );

    const screenHits = worldHits.map((hit) => {
      const s = worldToScreen(hit.x, hit.y, layout.cx, layout.cy, cam);
      return { ...hit, x: s.x, y: s.y, r: hit.r * cam.zoom };
    });
    onHits?.(screenHits);
    blackholeStore.setOrbitalHits([]);

    if (compositeCanvas) {
      const dpr = effectiveDpr(window.devicePixelRatio || 1, cosmicRenderStore.config.dprCap);
      compositeCosmicLayers(compositeCanvas, width, height, dpr, {
        gl: useWebGl ? glCanvas : undefined,
        three: showBodies3d ? threeCanvas : undefined,
        overlay: overlayCanvas,
        voidColor: COSMIC_PALETTE.void.hex,
        overlayBlend: useWebGl ? "screen" : "source-over",
        overlayAlpha: useWebGl ? 0.82 : 1,
      });
    }

    raf = requestAnimationFrame(loop);
  }

  onMount(() => {
    reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (glCanvas) {
      cosmicGl = createCosmicGl(glCanvas);
      useWebGl = cosmicGl.available;
      const gl = glCanvas.getContext("webgl2");
      cosmicRenderStore.initFromGl(gl);
    }
    if (threeCanvas) {
      planetLayer = createCosmicPlanetLayer(threeCanvas);
      useBodies3d = planetLayer.available;
    }
    resize();
    const ro = new ResizeObserver(resize);
    if (container) ro.observe(container);
    raf = requestAnimationFrame(loop);
    return () => {
      cancelAnimationFrame(raf);
      ro.disconnect();
      particles.clear();
      cosmicGl?.destroy();
      cosmicGl = null;
      planetLayer?.destroy();
      planetLayer = null;
    };
  });

  const ariaLabel = $derived(
    `Territoire cosmique — Cortex, ${connectionStore.memories.length} mémoires, niveau ${cosmicStore.zoomLevel}, nuance ${Math.round(blackholeStore.nuanceDepth * 100)}%`,
  );
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  bind:this={container}
  class="blackhole-stage absolute inset-0 {interactive ? 'blackhole-stage--interactive' : ''} {className}"
  aria-label={ariaLabel}
  role="img"
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
  onwheel={onWheel}
  ondblclick={onDoubleClick}
>
  <canvas bind:this={glCanvas} class="blackhole-stage__gl h-full w-full" aria-hidden="true"></canvas>
  <canvas
    bind:this={threeCanvas}
    class="blackhole-stage__three pointer-events-none absolute inset-0 h-full w-full"
    aria-hidden="true"
  ></canvas>
  <canvas
    bind:this={overlayCanvas}
    class="blackhole-stage__overlay pointer-events-none absolute inset-0 h-full w-full"
    aria-hidden="true"
  ></canvas>
</div>