<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import CosmicShell from "$lib/cosmic/CosmicShell.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";

  onMount(() => connectionStore.connect());
  onDestroy(() => connectionStore.disconnect());
</script>

<CosmicShell />