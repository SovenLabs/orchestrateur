<script lang="ts">
  import AgentActions from "$lib/components/agents/AgentActions.svelte";
  import EhSectionTitle from "$lib/components/agents/EhSectionTitle.svelte";
  import HeartbeatViewer from "$lib/components/agents/HeartbeatViewer.svelte";
  import MemoryExplorer from "$lib/components/agents/MemoryExplorer.svelte";
  import MessageCenter from "$lib/components/agents/MessageCenter.svelte";
  import MetricBadge from "$lib/components/MetricBadge.svelte";
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";
  import { agentsStore } from "$lib/stores/agents.svelte";
  import { agentStatusIndicator } from "$lib/ws/bridge";

  const agent = $derived(agentsStore.selected);
  const unread = $derived(agentsStore.inbox.filter((m) => !m.read).length);
</script>

{#if !agent}
  <p class="text-sm text-[var(--text-muted)]">Sélectionnez un sub-agent dans la liste.</p>
{:else}
  <div class="space-y-4">
    <header class="eh-card flex flex-wrap items-start justify-between gap-3 p-4">
      <div>
        <button
          type="button"
          class="mb-2 text-xs text-[var(--accent-cyan)] hover:underline"
          onclick={() => agentsStore.backToList()}
        >
          ← Retour liste
        </button>
        <h2 class="text-lg font-medium">{agent.name}</h2>
        <p class="font-mono text-xs text-[var(--text-muted)]">{agent.id}</p>
      </div>
      <StatusIndicator
        status={agentStatusIndicator(agent.status)}
        label={agent.status}
        pulse={agent.status === "awake" || agent.status === "background"}
      />
    </header>

    <section class="eh-detail-section">
      <EhSectionTitle title="Identité" subtitle="Rôle, modèle et métriques" />
      <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
        <MetricBadge label="Rôle" value={agent.role || "—"} />
        <MetricBadge label="Modèle" value={agent.model || "—"} />
        <MetricBadge label="Inbox" value={unread} unit="non lus" />
        <MetricBadge label="Activité" value={Math.round(agent.activity * 100)} unit="%" />
      </div>
    </section>

    <section class="eh-detail-section">
      <EhSectionTitle title="Actions" subtitle="Cycle de vie de l'agent" />
      <AgentActions agentId={agent.id} status={agent.status} />
    </section>

    <div class="grid gap-4 lg:grid-cols-2">
      <section class="eh-detail-section">
        <EhSectionTitle title="Heartbeat" subtitle="Cycle de vie et dernier signal" />
        <HeartbeatViewer {agent} />
      </section>
      <section class="eh-detail-section">
        <EhSectionTitle title="Mémoires" subtitle="Exploration contextuelle" />
        <MemoryExplorer agentId={agent.id} />
      </section>
    </div>

    <section class="eh-detail-section">
      <EhSectionTitle title="Messages" subtitle="Inbox et échanges" />
      <MessageCenter agentId={agent.id} agentName={agent.name} />
    </section>
  </div>
{/if}