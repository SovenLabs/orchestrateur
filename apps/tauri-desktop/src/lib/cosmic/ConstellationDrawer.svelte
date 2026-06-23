<script lang="ts">
  import DrawerPanelHost from "$lib/cosmic/DrawerPanelHost.svelte";
  import AgentsPanel from "$lib/panels/AgentsPanel.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
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
        <h2 class="cosmic-drawer__title">Agents</h2>
        <p class="cosmic-drawer__subtitle">
          {connectionStore.agents.length} opérateur{connectionStore.agents.length === 1 ? "" : "s"} · registre & activité
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
    <div class="cosmic-drawer__body scroll-thin">
      <DrawerPanelHost>
        <AgentsPanel />
      </DrawerPanelHost>
    </div>
  </aside>
{/if}