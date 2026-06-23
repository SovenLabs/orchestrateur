<script lang="ts">
  import DrawerPanelHost from "$lib/cosmic/DrawerPanelHost.svelte";
  import MetricBadge from "$lib/components/MetricBadge.svelte";
  import DraftsPanel from "$lib/components/DraftsPanel.svelte";
  import WatcherIndicator from "$lib/components/WatcherIndicator.svelte";
  import MemoryNebula from "$lib/cosmic/MemoryNebula.svelte";
  import AgentsPanel from "$lib/panels/AgentsPanel.svelte";
  import DashboardPanel from "$lib/panels/DashboardPanel.svelte";
  import MemoryPanel from "$lib/panels/MemoryPanel.svelte";
  import MonitoringPanel from "$lib/panels/MonitoringPanel.svelte";
  import { blackholeStore } from "$lib/stores/blackhole.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { PANEL_META, type PanelId } from "$lib/types/ui";

  type InsightSection = "flux" | "drafts" | PanelId;

  let section = $state<InsightSection>("flux");

  const draftCount = $derived(connectionStore.drafts.length);

  const tabs = $derived.by(() => {
    const items: { id: InsightSection; label: string; icon: string; hint: string; badge?: number }[] = [
      { id: "flux", label: "Flux", icon: "∿", hint: "Événements temps réel" },
      { id: "drafts", label: "Drafts", icon: "◫", hint: "Brouillons à publier", badge: draftCount },
      ...(["dashboard", "memory", "agents", "monitoring"] as PanelId[]).map((id) => ({
        id,
        label: PANEL_META[id].label,
        icon: PANEL_META[id].icon,
        hint: PANEL_META[id].description,
      })),
    ];
    return items;
  });
</script>

{#if navigationStore.insightsPanelOpen}
  <div
    class="cosmic-drawer-scrim"
    role="presentation"
    onclick={() => (navigationStore.insightsPanelOpen = false)}
  ></div>
  <aside class="cosmic-drawer cosmic-drawer--right drawer-slide-right" aria-label="Flux et insights">
    <header class="cosmic-drawer__header">
      <div class="min-w-0 flex-1">
        <p class="cosmic-drawer__eyebrow">Navigation · ]</p>
        <h2 class="cosmic-drawer__title">Insights</h2>
        <p class="cosmic-drawer__subtitle">
          {tabs.find((t) => t.id === section)?.hint ?? "Second brain — métriques & panneaux"}
        </p>
      </div>
      <button
        type="button"
        class="cosmic-drawer__close"
        onclick={() => (navigationStore.insightsPanelOpen = false)}
        aria-label="Fermer insights"
      >
        ✕
      </button>

      <div class="cosmic-drawer__tabs" role="tablist" aria-label="Sections insights">
        {#each tabs as tab (tab.id)}
          <button
            type="button"
            role="tab"
            aria-selected={section === tab.id}
            class="cosmic-drawer__tab {section === tab.id ? 'cosmic-drawer__tab--active' : ''}"
            onclick={() => {
              section = tab.id;
              if (tab.id === "dashboard" || tab.id === "memory" || tab.id === "agents" || tab.id === "monitoring") {
                navigationStore.navigate(tab.id);
              }
            }}
          >
            <span aria-hidden="true">{tab.icon}</span>
            <span>{tab.label}</span>
            {#if tab.badge && tab.badge > 0}
              <span class="cosmic-drawer__tab-badge">{tab.badge}</span>
            {/if}
          </button>
        {/each}
      </div>

      <div class="cosmic-drawer__metrics">
        <WatcherIndicator />
        <MetricBadge label="Nuance" value={Math.round(blackholeStore.nuanceDepth * 100)} unit="%" />
        <MetricBadge label="Drafts" value={draftCount} />
        <MetricBadge label="Messages" value={connectionStore.chatMessages.length} />
      </div>
    </header>

    <div class="cosmic-drawer__body scroll-thin">
      <DrawerPanelHost>
        {#if section === "flux"}
          <section>
            <h3 class="mb-2 text-xs uppercase tracking-wider text-[var(--text-muted)]">Orchestration live</h3>
            <ul class="space-y-1.5 text-xs text-[var(--text-secondary)]">
              {#each connectionStore.eventLog.slice(0, 32) as ev, i (i)}
                <li class="rounded-lg border border-[var(--glass-border)] bg-[var(--bg-input)] px-3 py-2 font-mono break-all">
                  {ev.event}
                </li>
              {:else}
                <li class="rounded-lg bg-[var(--bg-input)] px-3 py-4 text-center text-[var(--text-muted)]">
                  Aucun événement récent
                </li>
              {/each}
            </ul>
          </section>
        {:else if section === "drafts"}
          <DraftsPanel />
        {:else if section === "dashboard"}
          <DashboardPanel />
        {:else if section === "memory"}
          <section class="space-y-4">
            <div>
              <h3 class="mb-2 text-xs uppercase tracking-wider text-[var(--text-muted)]">Nébuleuse</h3>
              <MemoryNebula />
            </div>
            <MemoryPanel />
          </section>
        {:else if section === "agents"}
          <AgentsPanel />
        {:else if section === "monitoring"}
          <MonitoringPanel />
        {/if}
      </DrawerPanelHost>
    </div>
  </aside>
{/if}