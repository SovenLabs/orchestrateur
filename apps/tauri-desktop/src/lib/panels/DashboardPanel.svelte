<script lang="ts">
  import GlowCard from "$lib/components/GlowCard.svelte";
  import MetricBadge from "$lib/components/MetricBadge.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";

  const activityPercent = $derived(Math.round(connectionStore.agentActivity * 100));
  const windows = $derived(connectionStore.serverHealth?.connected_windows);
</script>

<div class="panel-enter grid gap-4 lg:grid-cols-3">
  <GlowCard title="AI Core / Cortex" subtitle="État du squelette souverain" class="lg:col-span-1">
    <dl class="space-y-2 text-sm">
      <div class="flex justify-between">
        <dt class="text-[var(--text-muted)]">Statut</dt>
        <dd>{connectionStore.health?.status ?? "—"}</dd>
      </div>
      <div class="flex justify-between">
        <dt class="text-[var(--text-muted)]">LLM</dt>
        <dd>{connectionStore.health?.llm_available ? "disponible" : "indisponible"}</dd>
      </div>
      <div class="flex justify-between">
        <dt class="text-[var(--text-muted)]">Embeddings</dt>
        <dd>{connectionStore.health?.embedding_available ? "disponible" : "indisponible"}</dd>
      </div>
      <div class="flex justify-between">
        <dt class="text-[var(--text-muted)]">Mémoires</dt>
        <dd>{connectionStore.memoryTotal}</dd>
      </div>
      {#if connectionStore.sessionId}
        <div class="flex justify-between gap-2">
          <dt class="text-[var(--text-muted)]">Session</dt>
          <dd class="truncate font-mono text-xs">{connectionStore.sessionId.slice(0, 12)}…</dd>
        </div>
      {/if}
    </dl>
  </GlowCard>

  <GlowCard title="Territoire multi-fenêtre" subtitle="Daemon hub — clients WS synchronisés" class="lg:col-span-2">
    {#if windows}
      <div class="mb-4 grid grid-cols-2 gap-2 sm:grid-cols-4">
        <MetricBadge label="Desktop" value={windows.desktop} />
        <MetricBadge label="Sphère" value={windows.sphere} />
        <MetricBadge label="Main Godot" value={windows.main} />
        <MetricBadge label="Extensions" value={windows.extension} />
      </div>
    {:else}
      <p class="mb-4 text-sm text-[var(--text-muted)]">Compteurs fenêtres — daemon non joint</p>
    {/if}
    <div class="flex flex-wrap gap-2">
      <button
        type="button"
        class="rounded-md border border-[var(--accent-cyan)]/40 px-3 py-1.5 text-xs text-[var(--accent-cyan)] hover:border-[var(--accent-cyan)] disabled:opacity-40"
        onclick={() => void uiStore.openSphere()}
        disabled={connectionStore.status !== "connected"}
      >
        ◉ Ouvrir Sphère
      </button>
      <button
        type="button"
        class="rounded-md border border-[var(--accent-cyan)]/30 px-3 py-1.5 text-xs text-[var(--accent-cyan)] hover:border-[var(--accent-cyan)] disabled:opacity-40"
        onclick={() => navigationStore.openTerritoryOverlay()}
        disabled={connectionStore.status !== "connected"}
      >
        ◎ Territoire embed
      </button>
      <button
        type="button"
        class="rounded-md border border-[var(--border-subtle)] px-3 py-1.5 text-xs hover:border-[var(--border-focus)] disabled:opacity-40"
        onclick={() => void uiStore.openTerritory()}
        disabled={connectionStore.status !== "connected"}
      >
        Territoire natif
      </button>
      <span class="self-center text-[10px] text-[var(--text-muted)]">
        {uiStore.sphereLaunchState === "running" ? "Sphère lancée" : "Prêt"}
        · {uiStore.territoryLaunchState === "running" ? "Territoire actif" : "—"}
      </span>
    </div>
  </GlowCard>

  <GlowCard title="Activité agent" subtitle="Pulse temps réel (brain_pulse)" class="lg:col-span-2">
    <div class="flex items-end gap-6">
      <div class="flex-1">
        <div class="h-3 overflow-hidden rounded-full bg-[var(--bg-input)]">
          <div
            class="h-full rounded-full bg-gradient-to-r from-[var(--accent-cyan)]/40 to-[var(--accent-cyan)] transition-all duration-500"
            style="width: {activityPercent}%"
          ></div>
        </div>
        <p class="mt-2 text-xs text-[var(--text-muted)]">Intensité {activityPercent}%</p>
      </div>
      <MetricBadge label="Event rate" value={connectionStore.eventRate} unit="/min" trend="flat" />
      <MetricBadge label="Latence WS" value={connectionStore.latencyMs ?? "—"} unit="ms" />
    </div>
  </GlowCard>

  <GlowCard title="Protocole" subtitle="Orchestrateur v2 hybride" class="lg:col-span-1">
    <dl class="space-y-2 text-sm">
      <div class="flex justify-between">
        <dt class="text-[var(--text-muted)]">Orchestrateur</dt>
        <dd class="font-mono text-xs">v{connectionStore.version ?? "—"}</dd>
      </div>
      <div class="flex justify-between">
        <dt class="text-[var(--text-muted)]">Protocole WS</dt>
        <dd class="font-mono text-xs">{connectionStore.protocolVersion ?? "—"}</dd>
      </div>
      <div class="flex justify-between">
        <dt class="text-[var(--text-muted)]">Clients WS</dt>
        <dd>{connectionStore.serverHealth?.connected_clients ?? "—"}</dd>
      </div>
    </dl>
  </GlowCard>

  <GlowCard title="Flux récent" subtitle="Derniers événements territoriaux" class="lg:col-span-3">
    <ul class="max-h-48 space-y-1.5 overflow-auto scroll-thin font-mono text-xs text-[var(--text-secondary)]">
      {#each connectionStore.eventLog.slice(0, 12) as ev (JSON.stringify(ev))}
        <li class="rounded bg-[var(--bg-input)] px-2 py-1">{JSON.stringify(ev)}</li>
      {:else}
        <li class="text-[var(--text-muted)]">En attente d'activité…</li>
      {/each}
    </ul>
  </GlowCard>
</div>