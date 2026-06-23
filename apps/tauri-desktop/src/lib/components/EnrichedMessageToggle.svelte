<script lang="ts">
  interface Props {
    original: string;
    enriched?: string | null;
    expanded?: boolean;
    ontoggle?: (expanded: boolean) => void;
  }

  let {
    original,
    enriched = null,
    expanded = $bindable(false),
    ontoggle,
  }: Props = $props();

  const hasEnriched = $derived(Boolean(enriched && enriched !== original));

  function toggle() {
    expanded = !expanded;
    ontoggle?.(expanded);
  }
</script>

{#if hasEnriched}
  <div class="enriched-toggle">
    <button type="button" class="enriched-toggle__btn" onclick={toggle}>
      {expanded ? "Réduire" : "Voir enrichi"}
    </button>
    <p class="enriched-toggle__body">
      {expanded ? enriched : original}
    </p>
    {#if expanded}
      <details class="enriched-toggle__original">
        <summary>Original</summary>
        <p>{original}</p>
      </details>
    {/if}
  </div>
{:else}
  <p class="enriched-toggle__body">{original}</p>
{/if}

<style>
  .enriched-toggle__btn {
    margin-bottom: 0.35rem;
    border-radius: 0.4rem;
    border: 1px solid var(--glass-border);
    background: transparent;
    color: var(--accent-cyan);
    font-size: 10px;
    padding: 0.15rem 0.45rem;
    cursor: pointer;
  }

  .enriched-toggle__body {
    white-space: pre-wrap;
    line-height: 1.5;
  }

  .enriched-toggle__original {
    margin-top: 0.5rem;
    font-size: 11px;
    color: var(--text-muted);
  }
</style>