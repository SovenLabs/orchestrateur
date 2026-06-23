<script lang="ts">
  import { onMount } from "svelte";
  import AgentPresenceStrip from "$lib/cosmic/AgentPresenceStrip.svelte";
  import CosmicTickerHeader from "$lib/cosmic/CosmicTickerHeader.svelte";
  import ConstellationDrawer from "$lib/cosmic/ConstellationDrawer.svelte";
  import ConversationPane from "$lib/cosmic/ConversationPane.svelte";
  import EscMenu from "$lib/cosmic/EscMenu.svelte";
  import InsightsPanel from "$lib/cosmic/InsightsPanel.svelte";
  import InsightsTriggerBar from "$lib/cosmic/InsightsTriggerBar.svelte";
  import { cosmicStore } from "$lib/stores/cosmic-store.svelte";
  import { cosmicCameraStore } from "$lib/stores/cosmic-camera.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { harnessStore } from "$lib/stores/harness.svelte";
  import SetupOverlay from "$lib/harness/SetupOverlay.svelte";
  import MessagingDrawer from "$lib/harness/MessagingDrawer.svelte";

  onMount(() => {
    void harnessStore.init();
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    const apply = () => document.documentElement.classList.toggle("reduce-motion", mq.matches);
    apply();
    mq.addEventListener("change", apply);
    return () => mq.removeEventListener("change", apply);
  });

  function onGlobalKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement | null;
    const typing =
      target?.tagName === "INPUT" ||
      target?.tagName === "TEXTAREA" ||
      target?.isContentEditable;

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      navigationStore.openEscMenu();
      return;
    }

    if (e.key === "Escape" || e.key === "Esc") {
      e.preventDefault();
      if (typing && target) target.blur();
      if (cosmicStore.zoomLevel !== "cosmos") {
        cosmicStore.zoomOut();
        cosmicCameraStore.reset();
        return;
      }
      if (cosmicCameraStore.cam.zoom !== 1 || cosmicCameraStore.cam.panX !== 0) {
        cosmicCameraStore.reset();
        return;
      }
      if (navigationStore.escMenuOpen) {
        navigationStore.closeEscMenu();
      } else if (navigationStore.leftDrawerOpen || navigationStore.insightsPanelOpen) {
        navigationStore.closeAllOverlays();
      } else {
        navigationStore.openEscMenu();
      }
      return;
    }

    if (!typing && e.key === "[") {
      e.preventDefault();
      navigationStore.toggleLeftDrawer();
    }
    if (!typing && e.key === "]") {
      e.preventDefault();
      navigationStore.toggleInsightsPanel();
    }
  }
</script>

<svelte:window onkeydowncapture={onGlobalKeydown} />

<div class="relative flex h-screen overflow-hidden bg-[var(--bg-base)] text-[var(--text-primary)]">
  <AgentPresenceStrip />

  <div class="relative flex min-w-0 flex-1 flex-col">
    <CosmicTickerHeader />
    <main class="relative min-h-0 flex-1">
      <ConversationPane />
    </main>
  </div>

  <InsightsTriggerBar />
</div>

<ConstellationDrawer />
<InsightsPanel />
<EscMenu />
<SetupOverlay />
<MessagingDrawer />