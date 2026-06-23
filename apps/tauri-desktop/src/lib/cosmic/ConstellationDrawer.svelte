<script lang="ts">
  import DrawerPanelHost from "$lib/cosmic/DrawerPanelHost.svelte";
  import AgentsView from "$lib/views/Agents.svelte";
  import AgentsSidebar from "$lib/components/AgentsSidebar.svelte";
  import { agentsStore } from "$lib/stores/agents.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
</script>

{#if navigationStore.leftDrawerOpen}
  <div
    class="cosmic-drawer-scrim"
    role="presentation"
    onclick={() => (navigationStore.leftDrawerOpen = false)}
  ></div>
  <aside
    class="cosmic-drawer cosmic-drawer--left drawer-slide-left"
    aria-label="Constellation d'agents"
  >
    <header class="cosmic-drawer__header">
      <div class="min-w-0">
        <p class="cosmic-drawer__eyebrow">Navigation · [</p>
        <h2 class="cosmic-drawer__title">Sub-Agents</h2>
        <p class="cosmic-drawer__subtitle">
          {agentsStore.agents.length} opérateur{agentsStore.agents.length === 1 ? "" : "s"} · registre & activité
        </p>
      </div>
      <button
        type="button"
        class="cosmic-drawer__close"
        onclick={() => (navigationStore.leftDrawerOpen = false)}
        aria-label="Fermer agents"
      >
        ✕
      </button>
    </header>
    <div class="cosmic-drawer__body scroll-thin flex min-h-0">
      <AgentsSidebar />
      <div class="min-w-0 flex-1 overflow-auto p-3">
        <DrawerPanelHost>
          <AgentsView />
        </DrawerPanelHost>
      </div>
    </div>
  </aside>
{/if}