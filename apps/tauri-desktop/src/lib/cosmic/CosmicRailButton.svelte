<script lang="ts">
  let {
    icon,
    label,
    shortcut,
    active = false,
    badge,
    onclick,
  }: {
    icon: string;
    label: string;
    shortcut: string;
    active?: boolean;
    badge?: number | string;
    onclick?: () => void;
  } = $props();
</script>

<button
  type="button"
  class="cosmic-rail-btn {active ? 'cosmic-rail-btn--active' : ''}"
  {onclick}
  aria-label="{label} — raccourci {shortcut}"
  aria-pressed={active}
  title="{label} ({shortcut})"
>
  <span class="cosmic-rail-btn__icon" aria-hidden="true">{icon}</span>
  <span class="cosmic-rail-btn__label">{label}</span>
  <kbd class="cosmic-rail-btn__key">{shortcut}</kbd>
  {#if badge !== undefined && badge !== 0 && badge !== ""}
    <span class="cosmic-rail-btn__badge">{badge}</span>
  {/if}
</button>