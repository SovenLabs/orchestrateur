<script lang="ts">
  import { connectionStore } from "$lib/stores/connection.svelte";

  const watcher = $derived(connectionStore.watcher);
  const statusLabel = $derived.by(() => {
    if (!watcher) return "Watcher —";
    if (!watcher.enabled) return "Watcher off";
    if (watcher.running) return `Watcher ▶ (${watcher.drafts_pending})`;
    return "Watcher ⏹";
  });
</script>

<button
  type="button"
  class="watcher-indicator"
  class:watcher-indicator--active={watcher?.running}
  title={watcher?.last_error ?? "Statut watcher sessions"}
  onclick={() => connectionStore.refreshWatcher()}
  disabled={connectionStore.status !== "connected"}
>
  <span aria-hidden="true">{watcher?.running ? "▶" : "⏹"}</span>
  <span>{statusLabel}</span>
</button>

<style>
  .watcher-indicator {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    border-radius: 999px;
    border: 1px solid var(--glass-border);
    background: var(--bg-input);
    color: var(--text-muted);
    font-size: 10px;
    padding: 0.25rem 0.6rem;
    cursor: pointer;
  }

  .watcher-indicator--active {
    color: var(--accent-cyan);
    border-color: color-mix(in srgb, var(--accent-cyan) 40%, transparent);
  }

  .watcher-indicator:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>