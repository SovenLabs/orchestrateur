<script lang="ts">
  import CosmicRailButton from "$lib/cosmic/CosmicRailButton.svelte";
  import { agentsStore } from "$lib/stores/agents.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { isAgentAwake } from "$lib/ws/bridge";

  const visibleAgents = $derived(agentsStore.agents.slice(0, 5));
  const agentCount = $derived(agentsStore.agents.length);
</script>

<aside class="cosmic-rail cosmic-rail--left" aria-label="Navigation agents">
  <CosmicRailButton
    icon="◇"
    label="Sub-Agents"
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
            {isAgentAwake(agent.status) ? 'cosmic-rail__avatar--active' : ''}"
          onclick={() => {
            agentsStore.openDetail(agent.id);
            navigationStore.leftDrawerOpen = true;
          }}
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