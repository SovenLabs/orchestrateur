<script lang="ts">
  import type { CosmicHit } from "$lib/cosmic/cosmic-model";
  import { computeBlackholeLayout } from "$lib/cosmic/blackhole-layout";
  import { cosmicStore } from "$lib/stores/cosmic-store.svelte";
  import { cosmicCameraStore } from "$lib/stores/cosmic-camera.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { blackholeStore } from "$lib/stores/blackhole.svelte";

  let {
    hits = [],
    stageWidth = 0,
    stageHeight = 0,
    dockT = 0,
  }: {
    hits?: CosmicHit[];
    stageWidth?: number;
    stageHeight?: number;
    dockT?: number;
  } = $props();

  let hovered: CosmicHit | null = $state(null);

  function onHit(hit: CosmicHit): void {
    const layout = computeBlackholeLayout(stageWidth, stageHeight, dockT);

    if (hit.kind === "galaxy" && hit.galaxyId) {
      cosmicStore.zoomInGalaxy(hit.galaxyId);
      cosmicCameraStore.focusWorld(hit.worldX, hit.worldY, layout.cx, layout.cy, 2.0);
      return;
    }
    if (hit.kind === "star" && hit.starId) {
      cosmicStore.zoomInBody(hit.starId);
      cosmicCameraStore.focusWorld(hit.worldX, hit.worldY, layout.cx, layout.cy, 3.2);
      return;
    }
    if (hit.kind === "planet" && hit.starId) {
      cosmicStore.zoomInBody(hit.starId, hit.planetId);
      cosmicCameraStore.focusWorld(hit.worldX, hit.worldY, layout.cx, layout.cy, 3.6);
      return;
    }
    if (hit.kind === "moon" && hit.memoryId) {
      navigationStore.navigate("memory");
      navigationStore.insightsPanelOpen = true;
      navigationStore.leftDrawerOpen = false;
    }
  }
</script>

{#if blackholeStore.state !== "docked"}
  <div class="orbital-hit-layer" aria-hidden="false">
    {#each hits as hit (hit.id)}
      <button
        type="button"
        class="orbital-hit orbital-hit--{hit.kind === 'moon' ? 'memory' : hit.kind === 'galaxy' ? 'galaxy' : 'agent'}"
        style="left: {hit.x - hit.r}px; top: {hit.y - hit.r}px; width: {hit.r * 2}px; height: {hit.r * 2}px;"
        title="{hit.label}"
        aria-label="{hit.kind} {hit.label}"
        onpointerdown={(e) => e.stopPropagation()}
        onmouseenter={() => (hovered = hit)}
        onmouseleave={() => (hovered = null)}
        onclick={(e) => {
          e.stopPropagation();
          onHit(hit);
        }}
      ></button>
    {/each}
    {#if hovered}
      <div
        class="cosmic-hit-tooltip pointer-events-none absolute z-30 rounded border border-[var(--glass-border)] bg-[var(--glass-bg)] px-2 py-1 text-[10px] text-[var(--sky)] backdrop-blur-sm"
        style="left: {hovered.x}px; top: {hovered.y - hovered.r - 22}px; transform: translateX(-50%);"
      >
        {hovered.label}
        {#if hovered.kind === "galaxy"}
          <span class="text-[var(--text-muted)]"> · clic zoom</span>
        {/if}
      </div>
    {/if}
  </div>
{/if}