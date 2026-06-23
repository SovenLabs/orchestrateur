<script lang="ts">
  import type { OrbitalHit } from "$lib/stores/blackhole.svelte";
  import { blackholeStore } from "$lib/stores/blackhole.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";

  function openNode(hit: OrbitalHit): void {
    if (hit.kind === "agent") {
      navigationStore.leftDrawerOpen = true;
      navigationStore.insightsPanelOpen = false;
      return;
    }
    if (hit.kind === "memory") {
      navigationStore.navigate("memory");
      navigationStore.insightsPanelOpen = true;
      navigationStore.leftDrawerOpen = false;
      return;
    }
    navigationStore.navigate("agents");
    navigationStore.insightsPanelOpen = true;
    navigationStore.leftDrawerOpen = false;
  }
</script>

{#if blackholeStore.state !== "docked"}
  <div class="orbital-hit-layer" aria-hidden="false">
    {#each blackholeStore.orbitalHits as hit (hit.id)}
      <button
        type="button"
        class="orbital-hit orbital-hit--{hit.kind}"
        style="left: {hit.x - hit.r}px; top: {hit.y - hit.r}px; width: {hit.r * 2}px; height: {hit.r * 2}px;"
        title="{hit.label} — {hit.kind}"
        aria-label="Neurone {hit.label}"
        onclick={() => openNode(hit)}
      ></button>
    {/each}
  </div>
{/if}