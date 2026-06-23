<script lang="ts">
  import { blackholeStore } from "$lib/stores/blackhole.svelte";

  let {
    visible = false,
    onactivate,
  }: {
    visible?: boolean;
    onactivate?: () => void;
  } = $props();

  const depthPct = $derived(Math.round(blackholeStore.nuanceDepth * 100));
  const thinking = $derived(blackholeStore.thinking);
</script>

{#if visible}
  <button
    type="button"
    class="nuance-orb glass-panel fixed top-6 z-30 flex h-[5.5rem] w-[5.5rem] flex-col items-center justify-center rounded-full shadow-lg {thinking ? 'nuance-orb--thinking' : ''}"
    style="right: calc(var(--cosmic-rail-w) + 1.25rem)"
    onclick={() => onactivate?.()}
    aria-label="Nuance Orb — {depthPct}% — rouvrir le trou noir"
  >
    <span
      class="nuance-orb__ring"
      style="--nuance-depth: {blackholeStore.nuanceDepth}"
      aria-hidden="true"
    ></span>
    <span class="nuance-orb__core" aria-hidden="true"></span>
    <span class="text-[10px] font-medium uppercase tracking-widest text-[var(--mauve)]">
      {depthPct}%
    </span>
  </button>
{/if}