<script lang="ts">
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";
  import { agentsStore } from "$lib/stores/agents.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { agentStatusIndicator, isAgentAwake } from "$lib/ws/bridge";

  const disabled = $derived(connectionStore.status !== "connected");
</script>

<aside
  class="flex w-52 shrink-0 flex-col border-r border-[var(--border-subtle)] bg-[var(--bg-card)]/80"
  aria-label="Sub-Agents"
>
  <header class="flex items-center justify-between border-b border-[var(--border-subtle)] px-3 py-2">
    <div>
      <h2 class="text-xs font-medium uppercase tracking-wider text-[var(--accent-cyan)]">Sub-Agents</h2>
      <p class="text-[10px] text-[var(--text-muted)]">{agentsStore.agents.length} opérateurs</p>
    </div>
    <button
      type="button"
      class="rounded border border-[var(--border-subtle)] px-1.5 py-0.5 text-xs disabled:opacity-50"
      disabled={disabled}
      onclick={() => agentsStore.fetchAll()}
      aria-label="Rafraîchir"
    >
      ↻
    </button>
  </header>

  <ul class="flex-1 space-y-1 overflow-auto scroll-thin p-2">
    {#each agentsStore.agents as agent (agent.id)}
      <li>
        <button
          type="button"
          class="flex w-full items-center justify-between gap-1 rounded-lg px-2 py-2 text-left text-sm transition-colors
            {agentsStore.selectedId === agent.id
            ? 'bg-[var(--accent-cyan)]/12 text-[var(--accent-cyan)]'
            : 'hover:bg-[var(--bg-input)]'}"
          onclick={() => {
            agentsStore.openDetail(agent.id);
            navigationStore.leftDrawerOpen = true;
          }}
        >
          <span class="truncate">{agent.name}</span>
          <StatusIndicator
            status={agentStatusIndicator(agent.status)}
            label={agent.status}
            pulse={isAgentAwake(agent.status)}
          />
        </button>
      </li>
    {:else}
      <li class="px-2 py-6 text-center text-xs text-[var(--text-muted)]">
        {disabled ? "Hors ligne" : "Aucun agent"}
      </li>
    {/each}
  </ul>

  <footer class="border-t border-[var(--border-subtle)] p-2">
    <button
      type="button"
      class="w-full rounded-lg border border-[var(--border-subtle)] px-2 py-1.5 text-xs text-[var(--text-secondary)] hover:border-[var(--accent-cyan)] disabled:opacity-50"
      disabled={disabled}
      onclick={() => {
        agentsStore.openCommunicationGraph();
        navigationStore.leftDrawerOpen = true;
      }}
    >
      Graphe communication
    </button>
  </footer>
</aside>