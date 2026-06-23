<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children?: Snippet } = $props();
</script>

<div class="drawer-panel-host">
  {#if children}
    {@render children()}
  {/if}
</div>

<style>
  .drawer-panel-host {
    min-width: 0;
    width: 100%;
    overflow-x: hidden;
  }

  .drawer-panel-host :global(.panel-enter) {
    display: flex !important;
    flex-direction: column !important;
    gap: 0.75rem;
    min-width: 0;
    width: 100%;
  }

  .drawer-panel-host :global(.panel-enter > *) {
    width: 100% !important;
    min-width: 0 !important;
    max-width: 100% !important;
  }

  .drawer-panel-host :global(.glow-card) {
    padding: 0.75rem;
  }

  .drawer-panel-host :global(input),
  .drawer-panel-host :global(select),
  .drawer-panel-host :global(textarea) {
    max-width: 100%;
  }

  .drawer-panel-host :global(.flex-wrap) {
    flex-wrap: wrap;
  }

  .drawer-panel-host :global(input[type="search"]),
  .drawer-panel-host :global(input.min-w-\[240px\]) {
    min-width: 0 !important;
    flex: 1 1 100%;
  }
</style>