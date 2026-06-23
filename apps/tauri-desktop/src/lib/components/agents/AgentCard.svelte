<script lang="ts">
  import AgentActions from "$lib/components/agents/AgentActions.svelte";
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";
  import type { AgentInfo } from "$lib/types/ui";
  import { agentStatusIndicator, isAgentAwake } from "$lib/ws/bridge";

  let {
    agent,
    onSelect,
  }: {
    agent: AgentInfo;
    onSelect?: (id: string) => void;
  } = $props();

  const awake = $derived(isAgentAwake(agent.status));
  const activityPct = $derived(Math.round(agent.activity * 100));
</script>

<article class="eh-card agent-card" data-agent-id={agent.id}>
  <div class="eh-card__lens" aria-hidden="true"></div>
  <div class="eh-card__accretion" aria-hidden="true"></div>

  <button
    type="button"
    class="agent-card__main"
    onclick={() => onSelect?.(agent.id)}
  >
    <div class="agent-card__header">
      <div class="min-w-0 flex-1">
        <h3 class="agent-card__name">{agent.name}</h3>
        <p class="agent-card__meta">
          <span class="font-mono">{agent.id}</span>
          {#if agent.role}
            · {agent.role}
          {/if}
        </p>
      </div>
      <StatusIndicator
        status={agentStatusIndicator(agent.status)}
        label={agent.status}
        pulse={awake}
      />
    </div>

    <dl class="agent-card__stats">
      <div>
        <dt>Modèle</dt>
        <dd>{agent.model || "—"}</dd>
      </div>
      <div>
        <dt>Activité</dt>
        <dd>{activityPct}%</dd>
      </div>
      <div>
        <dt>Inbox</dt>
        <dd>{agent.unreadInbox ?? 0}</dd>
      </div>
    </dl>

    {#if agent.lastAction}
      <p class="agent-card__action">{agent.lastAction}</p>
    {/if}
  </button>

  <footer class="agent-card__footer">
    <AgentActions agentId={agent.id} status={agent.status} compact />
  </footer>
</article>