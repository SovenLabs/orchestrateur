<script lang="ts">
  import { notificationsStore } from "$lib/stores/notifications.svelte";

  const colors = {
    info: "var(--accent-cyan)",
    warn: "var(--status-orange)",
    error: "var(--status-red)",
  };
</script>

{#if notificationsStore.items.length > 0}
  <div class="pointer-events-none fixed bottom-4 right-4 z-[60] flex max-w-sm flex-col gap-2">
    {#each notificationsStore.items.slice(0, 4) as n (n.id)}
      <div
        class="pointer-events-auto rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-card)] px-3 py-2 text-sm shadow-lg"
        style="border-left: 3px solid {colors[n.level]}"
        role="status"
      >
        <div class="flex items-start justify-between gap-2">
          <p class="text-[var(--text-secondary)]">{n.message}</p>
          <button
            type="button"
            class="text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)]"
            onclick={() => notificationsStore.dismiss(n.id)}
            aria-label="Fermer"
          >
            ✕
          </button>
        </div>
      </div>
    {/each}
  </div>
{/if}