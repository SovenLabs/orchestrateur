<script lang="ts">
  import { galaxyLabel } from "$lib/cosmic/cosmic-taxonomy";
  import { downloadBlob, recordCosmicLoop } from "$lib/cosmic/cosmic-export";
  import { cosmicCameraStore } from "$lib/stores/cosmic-camera.svelte";
  import { cosmicRenderStore } from "$lib/stores/cosmic-render.svelte";
  import { cosmicStore } from "$lib/stores/cosmic-store.svelte";

  let {
    exportCanvas,
    stageCenterX = 0,
    stageCenterY = 0,
  }: {
    exportCanvas?: HTMLCanvasElement;
    stageCenterX?: number;
    stageCenterY?: number;
  } = $props();

  let exporting = $state(false);
  let exportError = $state<string | null>(null);

  const crumbs = $derived(
    cosmicStore.breadcrumb().map((part, i) => {
      if (i === 0) return "Cosmos";
      if (i === 1) return galaxyLabel(part);
      return part.charAt(0).toUpperCase() + part.slice(1);
    }),
  );

  const zoomPct = $derived(Math.round(cosmicCameraStore.cam.zoom * 100));
  const renderLabel = $derived(
    cosmicRenderStore.effectivePreset === "cinema" ? "Cinéma" : "Éco",
  );

  async function exportLoop() {
    if (!exportCanvas || exporting) return;
    exporting = true;
    exportError = null;
    try {
      const cfg = cosmicRenderStore.config;
      const blob = await recordCosmicLoop(exportCanvas, {
        durationSec: 5,
        fps: cfg.exportMax.fps,
        videoBitsPerSecond: cfg.exportMax.bitrate,
        preset: cfg,
      });
      const stamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-");
      downloadBlob(blob, `orchestrateur-cosmos-${stamp}.webm`);
    } catch (e) {
      exportError = e instanceof Error ? e.message : "Export impossible";
    } finally {
      exporting = false;
    }
  }

  function zoomIn() {
    cosmicCameraStore.zoomBy(1.2, stageCenterX, stageCenterY);
  }

  function zoomOut() {
    cosmicCameraStore.zoomBy(1 / 1.2, stageCenterX, stageCenterY);
  }

  function resetView() {
    cosmicCameraStore.reset();
    cosmicStore.resetZoom();
  }
</script>

<div class="cosmic-legend pointer-events-none absolute bottom-28 left-5 z-20 max-w-xs">
  <p class="cosmic-legend__crumb text-[10px] text-[var(--sky)]">
    {crumbs.join(" › ")} · {zoomPct}%
  </p>
  <ul class="cosmic-legend__items mt-2 space-y-1 text-[9px] uppercase tracking-wider text-[var(--text-muted)]">
    <li><span class="text-[var(--mauve)]">●</span> Cortex — singularité</li>
    <li><span class="text-[var(--sky)]">○</span> Orchestrateur — horizon</li>
    <li><span class="text-[var(--apricot)]">◆</span> Galaxie — grande catégorie</li>
    <li><span class="text-[#fff8dc]">★</span> Étoile — sujet stable</li>
    <li><span class="text-[var(--teal)]">—</span> Trou de ver — lien cross-sujet</li>
  </ul>
  <p class="mt-2 text-[9px] leading-relaxed text-[var(--text-muted)]">
    Glisser = déplacer la carte · Molette haut = zoom · Molette bas = dézoom
    <br />Double-clic ou Échap = vue d'ensemble · Clic galaxie = entrer
  </p>
  <div class="pointer-events-auto mt-3 flex flex-wrap items-center gap-1.5">
    <button
      type="button"
      class="cosmic-zoom-btn rounded border border-[var(--glass-border)] bg-[var(--glass-bg)] px-2 py-1 text-[11px] text-[var(--sky)] backdrop-blur-sm hover:border-[var(--sky)]"
      onclick={zoomOut}
      title="Dézoomer"
    >−</button>
    <button
      type="button"
      class="cosmic-zoom-btn rounded border border-[var(--glass-border)] bg-[var(--glass-bg)] px-2 py-1 text-[11px] text-[var(--sky)] backdrop-blur-sm hover:border-[var(--sky)]"
      onclick={zoomIn}
      title="Zoomer"
    >+</button>
    <button
      type="button"
      class="rounded border border-[var(--glass-border)] bg-[var(--glass-bg)] px-2 py-1 text-[9px] uppercase tracking-wider text-[var(--text-muted)] backdrop-blur-sm hover:border-[var(--sky)] hover:text-[var(--sky)]"
      onclick={resetView}
    >Reset</button>
    <button
      type="button"
      class="rounded border border-[var(--glass-border)] bg-[var(--glass-bg)] px-2 py-1 text-[9px] uppercase tracking-wider text-[var(--mauve)] backdrop-blur-sm hover:border-[var(--mauve)]"
      onclick={() => cosmicRenderStore.togglePreset()}
      title="Qualité rendu"
    >{renderLabel}</button>
    <button
      type="button"
      class="cosmic-export-btn rounded border border-[var(--glass-border)] bg-[var(--glass-bg)] px-2 py-1 text-[9px] uppercase tracking-wider text-[var(--sky)] backdrop-blur-sm transition hover:border-[var(--sky)] disabled:opacity-50"
      disabled={!exportCanvas || exporting}
      onclick={exportLoop}
    >
      {exporting ? "Export…" : "WebM 5s"}
    </button>
  </div>
  {#if exportError}
    <p class="pointer-events-auto mt-1 text-[9px] text-[var(--status-red)]">{exportError}</p>
  {/if}
</div>