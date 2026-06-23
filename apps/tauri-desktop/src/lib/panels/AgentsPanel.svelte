<script lang="ts">
  import GlowCard from "$lib/components/GlowCard.svelte";
  import MetricBadge from "$lib/components/MetricBadge.svelte";
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
</script>

<div class="panel-enter grid gap-4 lg:grid-cols-2">
  <GlowCard title="Agents Registry" subtitle="Opérateurs de l'Esprit">
    <ul class="space-y-3">
      {#each connectionStore.agents as agent (agent.id)}
        <li class="flex items-center justify-between rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] px-3 py-3">
          <div>
            <p class="font-medium">{agent.name}</p>
            <p class="text-xs text-[var(--text-muted)]">{agent.lastAction ?? "En veille"}</p>
          </div>
          <StatusIndicator
            status={agent.status === "active" ? "ok" : agent.status === "error" ? "error" : "idle"}
            label={agent.status}
            pulse={agent.status === "active"}
          />
        </li>
      {/each}
    </ul>
  </GlowCard>

  <GlowCard title="Activity Feed" subtitle="Skills & outils invoqués">
    <div class="mb-4 grid grid-cols-2 gap-2">
      <MetricBadge label="Activité" value={Math.round(connectionStore.agentActivity * 100)} unit="%" />
      <MetricBadge label="Skills" value={connectionStore.skills.length} />
    </div>
    <ul class="max-h-56 space-y-2 overflow-auto scroll-thin text-sm text-[var(--text-secondary)]">
      {#each connectionStore.skills as skill}
        <li class="rounded bg-[var(--bg-input)] px-2 py-1.5">
          <span class="font-medium text-[var(--accent-cyan)]">{skill.name}</span>
          <span class="text-[var(--text-muted)]"> — {skill.description}</span>
        </li>
      {:else}
        <li class="text-[var(--text-muted)]">Liste des skills via daemon (ListSkills)…</li>
      {/each}
    </ul>
  </GlowCard>
</div>