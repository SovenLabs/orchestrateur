<script lang="ts">
  import { uiStore } from "$lib/stores/ui.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { PANEL_META, type CommandAction, type PanelId } from "$lib/types/ui";

  const canLaunch = $derived(connectionStore.status === "connected");

  let query = $state("");
  let selected = $state(0);

  const actions = $derived<CommandAction[]>([
    ...(["dashboard", "memory", "chat", "agents", "monitoring"] as PanelId[]).map((panel) => ({
      id: `nav-${panel}`,
      label: `Aller à ${PANEL_META[panel].label}`,
      panel,
    })),
    {
      id: "health",
      label: "HealthCheck daemon",
      action: () => connectionStore.healthCheck(),
    },
    {
      id: "ping",
      label: "Ping WebSocket",
      action: () => connectionStore.ping(),
    },
    {
      id: "memories",
      label: "Rafraîchir les mémoires",
      action: () => connectionStore.fetchMemories(),
    },
    {
      id: "sphere",
      label: "Ouvrir Sphère Godot",
      action: () => void uiStore.openSphere(),
      disabled: !canLaunch,
    },
    {
      id: "territory",
      label: "Ouvrir Territoire Godot",
      action: () => void uiStore.openTerritory(),
      disabled: !canLaunch,
    },
    {
      id: "launch-status",
      label: "Rafraîchir statut fenêtres Godot",
      action: () => void uiStore.refreshLaunchStatus(),
    },
  ]);

  const filtered = $derived(
    actions
      .filter((a) => !a.disabled)
      .filter((a) => a.label.toLowerCase().includes(query.toLowerCase())),
  );

  function run(action: CommandAction) {
    if (action.panel) navigationStore.navigate(action.panel);
    action.action?.();
    uiStore.closeCommandPalette();
    query = "";
    selected = 0;
  }

  function onKeydown(e: KeyboardEvent) {
    if (!uiStore.commandPaletteOpen) return;
    if (e.key === "Escape") {
      uiStore.closeCommandPalette();
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selected = Math.min(selected + 1, filtered.length - 1);
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      selected = Math.max(selected - 1, 0);
    }
    if (e.key === "Enter" && filtered[selected]) {
      e.preventDefault();
      run(filtered[selected]);
    }
  }

  $effect(() => {
    if (uiStore.commandPaletteOpen) selected = 0;
  });
</script>

<svelte:window onkeydown={onKeydown} />

{#if uiStore.commandPaletteOpen}
  <div
    class="fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-[18vh] backdrop-blur-sm"
    role="dialog"
    aria-modal="true"
    aria-label="Palette de commandes"
    onclick={(e) => e.target === e.currentTarget && uiStore.closeCommandPalette()}
  >
    <div class="w-full max-w-lg overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-card)] shadow-2xl">
      <input
        type="text"
        class="w-full border-b border-[var(--border-subtle)] bg-transparent px-4 py-3 text-sm outline-none"
        placeholder="Rechercher une commande…"
        bind:value={query}
        autofocus
      />
      <ul class="max-h-64 overflow-auto scroll-thin py-1">
        {#each filtered as action, i}
          <li>
            <button
              type="button"
              class="flex w-full px-4 py-2.5 text-left text-sm transition-colors
                {i === selected ? 'bg-[var(--accent-cyan)]/10 text-[var(--accent-cyan)]' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
              onclick={() => run(action)}
            >
              {action.label}
            </button>
          </li>
        {:else}
          <li class="px-4 py-6 text-center text-sm text-[var(--text-muted)]">Aucune commande</li>
        {/each}
      </ul>
    </div>
  </div>
{/if}