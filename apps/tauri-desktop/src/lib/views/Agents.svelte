<script lang="ts">
  import AgentCard from "$lib/components/agents/AgentCard.svelte";
  import AgentFilters from "$lib/components/agents/AgentFilters.svelte";
  import GlowCard from "$lib/components/GlowCard.svelte";
  import MetricBadge from "$lib/components/MetricBadge.svelte";
  import AgentDetailView from "$lib/views/AgentDetail.svelte";
  import CommunicationHub from "$lib/views/CommunicationHub.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { agentsStore } from "$lib/stores/agents.svelte";
  import { isAgentAwake } from "$lib/ws/bridge";

  const disabled = $derived(connectionStore.status !== "connected");
  const awakeCount = $derived(
    agentsStore.agents.filter((a) => isAgentAwake(a.status)).length,
  );
</script>

<div class="panel-enter agents-dashboard space-y-4">
  {#if agentsStore.viewMode === "detail"}
    <AgentDetailView />
  {:else if agentsStore.viewMode === "communication"}
    <CommunicationHub />
  {:else}
    <GlowCard title="Sub-Agents" subtitle="Opérateurs persistants · temps réel">
      {#snippet header()}
        <header class="mb-3 flex flex-wrap items-start justify-between gap-2">
          <div>
            <h3 class="text-sm font-medium text-[var(--accent-cyan)]">Sub-Agents</h3>
            <p class="mt-0.5 text-xs text-[var(--text-muted)]">
              {agentsStore.agents.length} enregistré{agentsStore.agents.length === 1 ? "" : "s"}
              · {awakeCount} actif{awakeCount === 1 ? "" : "s"}
            </p>
          </div>
          <div class="flex gap-1">
            <button
              type="button"
              class="rounded-lg border border-[var(--border-subtle)] px-2 py-1 text-xs disabled:opacity-50"
              disabled={disabled}
              onclick={() => agentsStore.openCommunicationGraph()}
            >
              Hub
            </button>
            <button
              type="button"
              class="rounded-lg border border-[var(--border-subtle)] px-2 py-1 text-xs disabled:opacity-50"
              disabled={disabled || agentsStore.loading}
              onclick={() => agentsStore.fetchAll()}
            >
              {agentsStore.loading ? "…" : "↻"}
            </button>
          </div>
        </header>
      {/snippet}

      <AgentFilters />

      <div class="mb-3 mt-3 grid grid-cols-3 gap-2">
        <MetricBadge label="Total" value={agentsStore.filteredAgents.length} />
        <MetricBadge label="Actifs" value={awakeCount} />
        <MetricBadge label="Skills" value={connectionStore.skills.length} />
      </div>

      <div class="agents-grid">
        {#each agentsStore.filteredAgents as agent (agent.id)}
          <AgentCard {agent} onSelect={(id) => agentsStore.openDetail(id)} />
        {:else}
          <div
            class="eh-card col-span-full px-3 py-8 text-center text-sm text-[var(--text-muted)]"
          >
            {#if agentsStore.loading}
              Chargement…
            {:else if disabled}
              Daemon déconnecté.
            {:else}
              Aucun sub-agent — <code class="text-xs">orch agent create</code>
            {/if}
          </div>
        {/each}
      </div>
    </GlowCard>
  {/if}
</div>