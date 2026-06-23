<script lang="ts">
  import CosmicRailButton from "$lib/cosmic/CosmicRailButton.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";

  const eventCount = $derived(Math.min(99, connectionStore.eventLog.length));
  const memoryBadge = $derived(
    connectionStore.memoryTotal > 0 ? connectionStore.memoryTotal : undefined,
  );
</script>

<aside class="cosmic-rail cosmic-rail--right" aria-label="Navigation insights">
  <CosmicRailButton
    icon="∿"
    label="Insights"
    shortcut="]"
    active={navigationStore.insightsPanelOpen}
    badge={memoryBadge ?? (eventCount || undefined)}
    onclick={() => navigationStore.toggleInsightsPanel()}
  />
</aside>