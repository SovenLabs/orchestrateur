<script lang="ts">
  import type { AgentInfo } from "$lib/types/ui";
  import { isAgentAwake } from "$lib/ws/bridge";

  let { agent }: { agent: AgentInfo | null } = $props();

  const phases = [
    { id: "sleeping", label: "Veille", color: "var(--text-muted)" },
    { id: "background", label: "Background", color: "var(--status-orange)" },
    { id: "awake", label: "Éveillé", color: "var(--status-green)" },
  ];
</script>

{#if agent}
  <div>
    <p class="mb-3 text-xs uppercase tracking-wider text-[var(--text-muted)]">Cycle de vie</p>
    <div class="flex items-center gap-1">
      {#each phases as phase, i (phase.id)}
        <div
          class="h-2 flex-1 rounded-full transition-opacity
            {agent.status === phase.id ? 'opacity-100' : 'opacity-25'}"
          style="background: {phase.color}"
          title={phase.label}
        ></div>
        {#if i < phases.length - 1}
          <span class="text-[10px] text-[var(--text-muted)]">→</span>
        {/if}
      {/each}
    </div>
    <dl class="mt-3 grid gap-2 text-xs">
      <div class="flex justify-between gap-2">
        <dt class="text-[var(--text-muted)]">Statut actuel</dt>
        <dd class="font-mono">{agent.status}</dd>
      </div>
      <div class="flex justify-between gap-2">
        <dt class="text-[var(--text-muted)]">Dernier heartbeat</dt>
        <dd class="truncate font-mono">{agent.lastHeartbeat ?? "—"}</dd>
      </div>
      <div class="flex justify-between gap-2">
        <dt class="text-[var(--text-muted)]">Actif</dt>
        <dd>{isAgentAwake(agent.status) ? "oui" : "non"}</dd>
      </div>
    </dl>
  </div>
{/if}