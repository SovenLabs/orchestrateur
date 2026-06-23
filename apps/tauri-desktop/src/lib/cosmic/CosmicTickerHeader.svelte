<script lang="ts">
  import { computeAgentsSync, computeCoherence } from "$lib/cosmic/cosmic-metrics";
  import { blackholeStore } from "$lib/stores/blackhole.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";

  const collapsed = $derived(blackholeStore.state === "docked");
  const nuancePct = $derived(Math.round(blackholeStore.nuanceDepth * 100));
  const agents = $derived(computeAgentsSync(connectionStore.agents));
  const coherence = $derived(
    computeCoherence({
      connected: connectionStore.status === "connected",
      llmAvailable: connectionStore.health?.llm_available ?? false,
      embeddingAvailable: connectionStore.health?.embedding_available ?? false,
      agentActivity: connectionStore.agentActivity,
      nuanceDepth: blackholeStore.nuanceDepth,
    }),
  );

  const statusDot = $derived(
    connectionStore.status === "connected"
      ? "var(--status-green)"
      : connectionStore.status === "reconnecting"
        ? "var(--status-orange)"
        : "var(--status-red)",
  );
</script>

<header
  class="cosmic-ticker {collapsed ? 'cosmic-ticker--collapsed' : ''}"
  aria-label="Métriques Orchestrateur"
>
  <div class="cosmic-ticker__brand">
    <span class="cosmic-ticker__logo" aria-hidden="true">◎</span>
    <span class="cosmic-ticker__name">Orchestrateur</span>
    <span class="cosmic-ticker__status" style="background: {statusDot}" title={connectionStore.status}></span>
  </div>

  <div class="cosmic-ticker__track" aria-live="polite">
    <div class="cosmic-ticker__metrics">
      <span class="cosmic-ticker__metric">
        <span class="cosmic-ticker__label">Nuance</span>
        <span class="cosmic-ticker__value">{nuancePct}%</span>
      </span>
      <span class="cosmic-ticker__sep" aria-hidden="true">·</span>
      <span class="cosmic-ticker__metric">
        <span class="cosmic-ticker__label">Agents</span>
        <span class="cosmic-ticker__value">{agents.label}</span>
      </span>
      <span class="cosmic-ticker__sep" aria-hidden="true">·</span>
      <span class="cosmic-ticker__metric">
        <span class="cosmic-ticker__label">Cohérence</span>
        <span class="cosmic-ticker__value">{coherence}%</span>
      </span>
      <span class="cosmic-ticker__sep" aria-hidden="true">·</span>
      <span class="cosmic-ticker__metric">
        <span class="cosmic-ticker__label">Mémoires</span>
        <span class="cosmic-ticker__value">{connectionStore.memoryTotal}</span>
      </span>
    </div>
    <p class="cosmic-ticker__hints">
      <kbd>Échap</kbd> menu · <kbd>]</kbd> insights · <kbd>[</kbd> agents
    </p>
  </div>
</header>