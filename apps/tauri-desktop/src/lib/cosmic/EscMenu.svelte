<script lang="ts">
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";
  import { blackholeStore } from "$lib/stores/blackhole.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { PANEL_META, type CommandAction, type PanelId } from "$lib/types/ui";

  const canLaunch = $derived(connectionStore.status === "connected");

  let query = $state("");
  let selected = $state(0);
  let searchInput: HTMLInputElement | undefined = $state();

  const statusKind = $derived(
    connectionStore.status === "connected"
      ? "ok"
      : connectionStore.status === "reconnecting"
        ? "warn"
        : "error",
  );

  const navigationActions = $derived<CommandAction[]>([
    ...(["dashboard", "memory", "agents", "monitoring"] as PanelId[]).map((panel) => ({
      id: `nav-${panel}`,
      label: `${PANEL_META[panel].icon} ${PANEL_META[panel].label}`,
      action: () => {
        navigationStore.navigate(panel);
        navigationStore.insightsPanelOpen = true;
      },
    })),
    {
      id: "agents-drawer",
      label: "◇ Constellation d'agents",
      action: () => {
        navigationStore.leftDrawerOpen = true;
        navigationStore.insightsPanelOpen = false;
      },
    },
    {
      id: "insights",
      label: "∿ Flux & Insights",
      action: () => {
        navigationStore.insightsPanelOpen = true;
      },
    },
  ]);

  const toolActions = $derived<CommandAction[]>([
    { id: "health", label: "HealthCheck daemon", action: () => connectionStore.healthCheck() },
    { id: "ping", label: "Ping WebSocket", action: () => connectionStore.ping() },
    { id: "memories", label: "Rafraîchir les mémoires", action: () => connectionStore.fetchMemories() },
  ]);

  const godotActions = $derived<CommandAction[]>([
    {
      id: "sphere",
      label: "Sphère Godot",
      action: () => void uiStore.openSphere(),
      disabled: !canLaunch,
    },
    {
      id: "territory",
      label: "Territoire Godot",
      action: () => void uiStore.openTerritory(),
      disabled: !canLaunch,
    },
  ]);

  const allActions = $derived([...navigationActions, ...toolActions, ...godotActions]);

  const filtered = $derived(
    allActions.filter((a) => !a.disabled).filter((a) => a.label.toLowerCase().includes(query.toLowerCase())),
  );

  function run(action: CommandAction) {
    action.action?.();
    navigationStore.closeEscMenu();
    query = "";
    selected = 0;
  }

  function onKeydown(e: KeyboardEvent) {
    if (!navigationStore.escMenuOpen) return;
    if (e.key === "Escape") {
      navigationStore.closeEscMenu();
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
    if (navigationStore.escMenuOpen) {
      selected = 0;
      requestAnimationFrame(() => searchInput?.focus());
    }
  });
</script>

<svelte:window onkeydown={onKeydown} />

{#if navigationStore.escMenuOpen}
  <div
    class="fixed inset-0 z-[60] flex items-start justify-center bg-black/65 pt-[8vh] backdrop-blur-md"
    role="dialog"
    aria-modal="true"
    aria-label="Command Center"
    tabindex="-1"
    onclick={(e) => e.target === e.currentTarget && navigationStore.closeEscMenu()}
    onkeydown={(e) => e.key === "Escape" && navigationStore.closeEscMenu()}
  >
    <div class="glass-panel w-full max-w-2xl overflow-hidden rounded-2xl shadow-2xl">
      <div class="border-b border-[var(--glass-border)] px-6 py-5">
        <p class="text-[10px] uppercase tracking-[0.25em] text-[var(--mauve)]">Orchestrateur</p>
        <h2 class="mt-1 text-xl font-semibold">Command Center</h2>
        <input
          bind:this={searchInput}
          type="search"
          class="mt-4 w-full rounded-lg border border-[var(--glass-border)] bg-[var(--bg-input)] px-4 py-3 text-sm outline-none focus:border-[var(--border-focus)]"
          placeholder="Rechercher navigation, outils, Godot…"
          bind:value={query}
        />
      </div>

      <div class="grid gap-0 md:grid-cols-2">
        <section class="border-b border-[var(--glass-border)] p-4 md:border-b-0 md:border-r">
          <h3 class="mb-2 text-xs uppercase tracking-wider text-[var(--text-muted)]">Nuance</h3>
          <p class="text-sm text-[var(--text-secondary)]">
            Profondeur : <strong class="text-[var(--text-primary)]">{Math.round(blackholeStore.nuanceDepth * 100)}%</strong>
            · état <strong class="text-[var(--mauve)]">{blackholeStore.state}</strong>
          </p>
          <div class="mt-2 h-2 overflow-hidden rounded-full bg-[var(--bg-input)]">
            <div
              class="h-full rounded-full bg-gradient-to-r from-[var(--mauve)] via-[var(--sky)] to-[var(--apricot)] transition-all duration-500"
              style="width: {blackholeStore.nuanceDepth * 100}%"
            ></div>
          </div>
          <p class="mt-3 text-xs text-[var(--text-muted)]">
            {connectionStore.memoryTotal} mémoires · {connectionStore.chatMessages.length} messages
          </p>
        </section>

        <section class="border-b border-[var(--glass-border)] p-4">
          <h3 class="mb-2 text-xs uppercase tracking-wider text-[var(--text-muted)]">Agents</h3>
          <ul class="max-h-40 space-y-2 overflow-auto scroll-thin text-sm">
            {#each connectionStore.agents as agent (agent.id)}
              <li class="flex items-center justify-between rounded-lg bg-[var(--bg-input)] px-3 py-2">
                <span>{agent.name}</span>
                <StatusIndicator
                  status={agent.status === "active" ? "ok" : agent.status === "error" ? "error" : "idle"}
                  label={agent.status}
                  pulse={agent.status === "active"}
                />
              </li>
            {:else}
              <li class="text-xs text-[var(--text-muted)]">Aucun agent enregistré</li>
            {/each}
          </ul>
        </section>

        <section class="border-t border-[var(--glass-border)] p-4 md:col-span-2">
          <h3 class="mb-2 text-xs uppercase tracking-wider text-[var(--text-muted)]">Actions</h3>
          <ul class="max-h-52 overflow-auto scroll-thin">
            {#each filtered as action, i}
              <li>
                <button
                  type="button"
                  class="flex w-full px-3 py-2 text-left text-sm transition-colors
                    {i === selected ? 'bg-[var(--mauve)]/15 text-[var(--mauve)]' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
                  onclick={() => run(action)}
                >
                  {action.label}
                </button>
              </li>
            {:else}
              <li class="px-3 py-4 text-sm text-[var(--text-muted)]">Aucune action</li>
            {/each}
          </ul>
        </section>

        <section class="border-t border-[var(--glass-border)] p-4 md:col-span-2">
          <h3 class="mb-2 text-xs uppercase tracking-wider text-[var(--text-muted)]">Système</h3>
          <div class="flex flex-wrap items-center gap-3 text-sm">
            <StatusIndicator status={statusKind} label={connectionStore.status} pulse={connectionStore.status === "connected"} />
            <span class="text-[var(--text-muted)]">v{connectionStore.version ?? "—"}</span>
            <span class="text-[var(--text-muted)]">↑↓ · Entrée · Échap</span>
          </div>
        </section>
      </div>
    </div>
  </div>
{/if}