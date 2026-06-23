<script lang="ts">
  import GlowCard from "$lib/components/GlowCard.svelte";
  import MetricBadge from "$lib/components/MetricBadge.svelte";
  import { useConnection } from "$lib/hooks/useConnection.svelte";
  import { useRealtimeMetrics } from "$lib/hooks/useRealtimeMetrics.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";

  const conn = useConnection();
  const metrics = useRealtimeMetrics();
</script>

<div class="panel-enter grid gap-4 md:grid-cols-2 xl:grid-cols-3">
  <GlowCard title="WebSocket Client">
    <div class="grid grid-cols-2 gap-2">
      <MetricBadge label="Statut" value={conn.status} />
      <MetricBadge label="Latence" value={conn.latencyMs ?? "—"} unit="ms" />
      <MetricBadge label="Event rate" value={conn.eventRate} unit="/min" trend="up" />
      <MetricBadge label="File d'attente" value={metrics.queuedMessages} />
      <MetricBadge label="Requêtes pending" value={metrics.pendingRequests} />
      <MetricBadge label="Protocole" value={conn.protocolVersion ?? "—"} />
    </div>
    <div class="mt-4 flex gap-2">
      <button
        type="button"
        class="rounded-md border border-[var(--border-subtle)] px-3 py-1.5 text-xs hover:border-[var(--border-focus)] disabled:opacity-40"
        onclick={() => conn.ping()}
        disabled={!conn.isConnected}
      >
        Ping
      </button>
      <button
        type="button"
        class="rounded-md border border-[var(--border-subtle)] px-3 py-1.5 text-xs hover:border-[var(--border-focus)] disabled:opacity-40"
        onclick={() => conn.healthCheck()}
        disabled={!conn.isConnected}
      >
        HealthCheck
      </button>
      <button
        type="button"
        class="rounded-md border border-[var(--border-subtle)] px-3 py-1.5 text-xs hover:border-[var(--border-focus)] disabled:opacity-40"
        onclick={() => metrics.refreshServerHealth()}
        disabled={!conn.isConnected}
      >
        Refresh /health
      </button>
    </div>
  </GlowCard>

  <GlowCard title="Daemon Serveur" subtitle="/health · /metrics">
    {#if metrics.serverMetrics}
      <div class="grid grid-cols-2 gap-2 text-sm">
        <MetricBadge label="Clients WS" value={metrics.connectedClients} />
        {#if connectionStore.serverHealth?.connected_windows}
          {@const w = connectionStore.serverHealth.connected_windows}
          <MetricBadge label="Desktop" value={w.desktop} />
          <MetricBadge label="Sphère" value={w.sphere} />
          <MetricBadge label="Main Godot" value={w.main} />
          <MetricBadge label="Extensions" value={w.extension} />
        {/if}
        <MetricBadge label="Msgs reçus" value={metrics.serverMetrics.messages_received} />
        <MetricBadge label="Msgs envoyés" value={metrics.serverMetrics.messages_sent} />
        <MetricBadge label="Broadcasts" value={metrics.serverMetrics.broadcasts_sent} />
        <MetricBadge label="Execute" value={metrics.serverMetrics.execute_requests} />
        <MetricBadge label="Pings" value={metrics.serverMetrics.ping_requests} />
      </div>
    {:else}
      <p class="text-sm text-[var(--text-muted)]">Métriques serveur indisponibles — daemon arrêté ?</p>
    {/if}
  </GlowCard>

  <GlowCard title="Providers">
    <dl class="grid gap-3 text-sm">
      <div class="rounded-lg bg-[var(--bg-input)] p-3">
        <dt class="text-[var(--text-muted)]">LLM</dt>
        <dd class="mt-1 font-medium">{connectionStore.health?.llm_available ? "OK" : "DOWN"}</dd>
      </div>
      <div class="rounded-lg bg-[var(--bg-input)] p-3">
        <dt class="text-[var(--text-muted)]">Embeddings</dt>
        <dd class="mt-1 font-medium">{connectionStore.health?.embedding_available ? "OK" : "DOWN"}</dd>
      </div>
    </dl>
  </GlowCard>

  <GlowCard title="Event log" class="md:col-span-2 xl:col-span-3">
    <ul class="max-h-72 space-y-1 overflow-auto scroll-thin font-mono text-xs text-[var(--text-secondary)]">
      {#each connectionStore.eventLog as ev (JSON.stringify(ev))}
        <li class="rounded bg-[var(--bg-input)] px-2 py-1">{JSON.stringify(ev)}</li>
      {:else}
        <li class="text-[var(--text-muted)]">Aucun événement</li>
      {/each}
    </ul>
  </GlowCard>
</div>