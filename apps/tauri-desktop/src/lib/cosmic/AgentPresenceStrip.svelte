<script lang="ts">
  import CosmicRailButton from "$lib/cosmic/CosmicRailButton.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";

  const visibleAgents = $derived(connectionStore.agents.slice(0, 5));
  const agentCount = $derived(connectionStore.agents.length);
</script>

<aside class="cosmic-rail cosmic-rail--left" aria-label="Navigation agents">
  <CosmicRailButton
    icon="◇"
    label="Agents"
    shortcut="["
    active={navigationStore.leftDrawerOpen}
    badge={agentCount || undefined}
    onclick={() => navigationStore.toggleLeftDrawer()}
  />

  {#if visibleAgents.length > 0}
    <div class="cosmic-rail__stack" aria-hidden="true">
      {#each visibleAgents as agent (agent.id)}
        <button
          type="button"
          class="cosmic-rail__avatar
            {agent.status === 'active' ? 'cosmic-rail__avatar--active' : ''}"
          onclick={() => navigationStore.toggleLeftDrawer()}
          title={agent.name}
          aria-label="Ouvrir {agent.name}"
        >
          {agent.name.charAt(0).toUpperCase()}
        </button>
      {/each}
    </div>
  {:else}
    <p class="cosmic-rail__hint">Aucun<br />agent</p>
  {/if}
</aside>