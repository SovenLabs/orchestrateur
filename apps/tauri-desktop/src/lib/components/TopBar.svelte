<script lang="ts">
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { PANEL_META } from "$lib/types/ui";

  const panelMeta = $derived(PANEL_META[navigationStore.activePanel]);

  const connStatus = $derived.by(() => {
    switch (connectionStore.status) {
      case "connected":
        return { status: "ok" as const, label: "Connecté", pulse: false };
      case "connecting":
      case "reconnecting":
        return { status: "warn" as const, label: connectionStore.status, pulse: true };
      default:
        return { status: "error" as const, label: "Déconnecté", pulse: false };
    }
  });

  const cortexStatus = $derived.by(() => {
    const h = connectionStore.health;
    if (!h) return { status: "idle" as const, label: "Cortex — en attente" };
    if (h.status === "ok") return { status: "ok" as const, label: "Cortex — opérationnel" };
    return { status: "warn" as const, label: `Cortex — ${h.status}` };
  });

  const windowSummary = $derived.by(() => {
    const w = connectionStore.serverHealth?.connected_windows;
    if (!w || w.total === 0) return null;
    return `${w.total} fenêtre${w.total > 1 ? "s" : ""} · D${w.desktop} S${w.sphere}`;
  });
</script>

<header class="flex items-center justify-between border-b border-[var(--border-subtle)] bg-[var(--bg-base)]/80 px-6 py-3 backdrop-blur-sm">
  <div>
    <h2 class="text-base font-medium tracking-tight">{panelMeta.label}</h2>
    <p class="text-xs text-[var(--text-muted)]">{panelMeta.description}</p>
  </div>

  <div class="flex items-center gap-3">
    <StatusIndicator status={cortexStatus.status} label={cortexStatus.label} />
    <StatusIndicator
      status={connStatus.status}
      label={connStatus.label}
      pulse={connStatus.pulse}
    />
    {#if windowSummary}
      <span class="rounded border border-[var(--border-subtle)] px-2 py-0.5 font-mono text-[10px] text-[var(--text-muted)]">
        {windowSummary}
      </span>
    {/if}
    {#if connectionStore.version}
      <span class="font-mono text-xs text-[var(--text-muted)]">v{connectionStore.version}</span>
    {/if}
    <button
      type="button"
      class="rounded-md border border-[var(--border-subtle)] px-2.5 py-1.5 text-xs text-[var(--text-secondary)] hover:border-[var(--border-focus)] hover:text-[var(--text-primary)]"
      onclick={() => uiStore.openCommandPalette()}
      title="Palette de commandes (Ctrl+K)"
    >
      ⌘K
    </button>
  </div>
</header>