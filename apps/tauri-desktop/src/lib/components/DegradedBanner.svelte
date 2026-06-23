<script lang="ts">
  import type { UiConnectionStatus } from "$lib/stores/connection.svelte";

  let { status }: { status: UiConnectionStatus } = $props();

  const visible = $derived(status !== "connected");
  const message = $derived(
    status === "reconnecting"
      ? "Reconnexion au daemon en cours…"
      : status === "connecting"
        ? "Connexion au daemon…"
        : "Mode dégradé — daemon inaccessible",
  );
</script>

{#if visible}
  <div
    class="border-b border-amber-500/25 bg-amber-500/10 px-6 py-2.5 text-sm text-amber-100"
    role="alert"
  >
    <span class="font-medium">{message}</span>
    {#if status === "disconnected"}
      <span class="ml-2 text-amber-200/80">
        Lancez
        <code class="text-[var(--accent-cyan)]">orchestrateur daemon run --workspace workspace</code>
        avec <code>ORCHESTRATEUR_DAEMON_TOKEN=dev</code>.
      </span>
    {/if}
  </div>
{/if}