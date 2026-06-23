<script lang="ts">
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { agentsStore } from "$lib/stores/agents.svelte";

  let {
    agentId,
    status = "sleeping",
    compact = false,
  }: {
    agentId: string;
    status?: string;
    compact?: boolean;
  } = $props();

  const disabled = $derived(
    connectionStore.status !== "connected" || agentsStore.actionPending,
  );

  async function onDelete(): Promise<void> {
    if (!confirm(`Supprimer définitivement l'agent « ${agentId} » ?`)) return;
    await agentsStore.deleteAgent(agentId);
  }
</script>

<div class="flex flex-wrap gap-2 {compact ? '' : 'mt-2'}">
  <button
    type="button"
    class="action-btn"
    disabled={disabled || status === "awake"}
    onclick={() => void agentsStore.wake(agentId)}
  >
    Réveiller
  </button>
  <button
    type="button"
    class="action-btn"
    disabled={disabled || status === "sleeping"}
    onclick={() => void agentsStore.sleep(agentId)}
  >
    Veille
  </button>
  <button
    type="button"
    class="action-btn"
    disabled={disabled}
    onclick={() => agentsStore.openDetail(agentId)}
  >
    Détail
  </button>
  <button
    type="button"
    class="action-btn action-btn--danger"
    disabled={disabled}
    onclick={() => void onDelete()}
  >
    Supprimer
  </button>
</div>

<style>
  .action-btn {
    border-radius: 0.5rem;
    border: 1px solid var(--border-subtle);
    background: var(--bg-input);
    padding: 0.375rem 0.75rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }
  .action-btn:hover:not(:disabled) {
    border-color: var(--accent-cyan);
  }
  .action-btn:disabled {
    opacity: 0.45;
  }
  .action-btn--danger:hover:not(:disabled) {
    border-color: var(--status-red);
    color: var(--status-red);
  }
</style>