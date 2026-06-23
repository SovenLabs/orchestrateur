<script lang="ts">
  import { kindColor, MEMORY_KIND_LABELS, type MemoryKindId } from "$lib/cosmic/cosmic-palette";

  interface Props {
    kind?: string;
    compact?: boolean;
  }

  let { kind = "context", compact = false }: Props = $props();

  const label = $derived(
    MEMORY_KIND_LABELS[(kind as MemoryKindId) ?? "context"] ?? kind,
  );
  const color = $derived(kindColor(kind));
</script>

<span
  class="kind-badge {compact ? 'kind-badge--compact' : ''}"
  style="--kind-color: {color}"
  title={label}
>
  {label}
</span>

<style>
  .kind-badge {
    display: inline-flex;
    align-items: center;
    border-radius: 999px;
    border: 1px solid color-mix(in srgb, var(--kind-color) 55%, transparent);
    background: color-mix(in srgb, var(--kind-color) 18%, transparent);
    color: var(--kind-color);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    padding: 0.2rem 0.55rem;
    line-height: 1.2;
  }

  .kind-badge--compact {
    font-size: 9px;
    padding: 0.1rem 0.4rem;
  }
</style>