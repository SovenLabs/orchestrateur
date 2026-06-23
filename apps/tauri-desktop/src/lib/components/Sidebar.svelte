<script lang="ts">
  import { onMount } from "svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { PANEL_META, type PanelId } from "$lib/types/ui";

  const panels = Object.keys(PANEL_META) as PanelId[];

  let sphereBusy = $state(false);

  onMount(() => {
    void uiStore.refreshLaunchStatus();
    const timer = setInterval(() => void uiStore.refreshLaunchStatus(), 8000);
    return () => clearInterval(timer);
  });

  async function openSphere() {
    if (sphereBusy) return;
    sphereBusy = true;
    try {
      await uiStore.openSphere();
    } finally {
      sphereBusy = false;
    }
  }

  async function openTerritory() {
    if (sphereBusy) return;
    sphereBusy = true;
    try {
      await uiStore.openTerritory();
    } finally {
      sphereBusy = false;
    }
  }
</script>

<aside class="flex w-60 shrink-0 flex-col border-r border-[var(--border-subtle)] bg-[var(--bg-elevated)]">
  <div class="border-b border-[var(--border-subtle)] px-5 py-5">
    <p class="text-[10px] uppercase tracking-[0.25em] text-[var(--accent-cyan)]">Orchestrateur</p>
    <h1 class="mt-1 text-lg font-semibold tracking-tight">Territoire</h1>
    <p class="mt-1 text-xs text-[var(--text-muted)]">Interface de commandement IA</p>
  </div>

  <nav class="flex-1 space-y-1 p-3" aria-label="Navigation principale">
    {#each panels as panel}
      {@const meta = PANEL_META[panel]}
      <button
        type="button"
        class="flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left text-sm transition-colors
          {navigationStore.activePanel === panel
          ? 'bg-[var(--bg-card)] text-[var(--accent-cyan)] shadow-[var(--glow-cyan)]'
          : 'text-[var(--text-secondary)] hover:bg-[var(--bg-card)]/60 hover:text-[var(--text-primary)]'}"
        onclick={() => navigationStore.navigate(panel)}
        aria-current={navigationStore.activePanel === panel ? "page" : undefined}
      >
        <span class="w-4 text-center text-base opacity-80">{meta.icon}</span>
        <span>
          <span class="block font-medium">{meta.label}</span>
          <span class="block text-[10px] text-[var(--text-muted)]">{meta.description}</span>
        </span>
      </button>
    {/each}
  </nav>

  <div class="space-y-2 border-t border-[var(--border-subtle)] p-3">
    <button
      type="button"
      class="flex w-full items-center justify-center gap-2 rounded-lg border border-[var(--accent-cyan)]/30 bg-[var(--accent-cyan)]/5 px-3 py-2.5 text-sm font-medium text-[var(--accent-cyan)] transition-all hover:border-[var(--accent-cyan)]/60 hover:shadow-[var(--glow-cyan)] disabled:opacity-40"
      onclick={openSphere}
      disabled={connectionStore.status !== "connected" || sphereBusy}
      title="Lancer Godot — Boule de Pixels Vivante (SphereDedicated)"
    >
      <span>◉</span>
      {sphereBusy ? "Lancement…" : uiStore.sphereLaunchState === "running" ? "Sphère active" : "Ouvrir Sphère"}
    </button>
    <button
      type="button"
      class="flex w-full items-center justify-center gap-2 rounded-lg border border-[var(--border-subtle)] px-3 py-2 text-xs text-[var(--text-secondary)] transition-colors hover:border-[var(--border-focus)] hover:text-[var(--text-primary)] disabled:opacity-40"
      onclick={openTerritory}
      disabled={connectionStore.status !== "connected" || sphereBusy}
      title="Lancer Godot — Territoire complet (MainTerritory)"
    >
      Territoire Godot
    </button>
    {#if uiStore.sphereLaunchMessage}
      <p
        class="px-1 text-[10px] {uiStore.sphereLaunchState === 'error'
          ? 'text-red-400'
          : 'text-[var(--text-muted)]'}"
      >
        {uiStore.sphereLaunchMessage}
      </p>
    {/if}
    {#if connectionStore.serverHealth?.connected_windows}
      {@const w = connectionStore.serverHealth.connected_windows}
      <p class="px-1 text-[10px] text-[var(--text-muted)]">
        WS : desktop {w.desktop} · sphère {w.sphere} · main {w.main} · ext {w.extension}
      </p>
    {/if}
  </div>
</aside>