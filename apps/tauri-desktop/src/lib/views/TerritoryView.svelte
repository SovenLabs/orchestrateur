<script lang="ts">
  import { onMount } from "svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { navigationStore } from "$lib/stores/navigation.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { DEFAULT_CONNECTION_CONFIG } from "$lib/generated/types";

  const GODOT_EMBED_PATH = "/godot/index.html";
  const TOKEN_KEY = "orchestrateur_daemon_token";

  let iframeEl = $state<HTMLIFrameElement | undefined>();
  let embedReady = $state(false);
  let embedMissing = $state(false);
  let launchingNative = $state(false);

  const daemonToken = $derived(
    import.meta.env.VITE_ORCHESTRATEUR_DAEMON_TOKEN ?? DEFAULT_CONNECTION_CONFIG.token,
  );

  function close(): void {
    navigationStore.closeTerritoryOverlay();
  }

  async function openNativeFallback(): Promise<void> {
    launchingNative = true;
    try {
      await uiStore.openTerritory();
    } finally {
      launchingNative = false;
    }
  }

  onMount(() => {
    localStorage.setItem(TOKEN_KEY, daemonToken);
    fetch(GODOT_EMBED_PATH, { method: "HEAD" })
      .then((res) => {
        embedMissing = !res.ok;
        embedReady = res.ok;
      })
      .catch(() => {
        embedMissing = true;
        embedReady = false;
      });
  });
</script>

{#if navigationStore.territoryOverlayOpen}
  <div class="territory-view" role="dialog" aria-label="Territoire Godot">
    <header class="territory-view__header">
      <div>
        <h2 class="text-sm font-medium text-[var(--accent-cyan)]">Territoire 3D</h2>
        <p class="text-xs text-[var(--text-muted)]">
          Embed HTML5/WASM · fallback fenêtre native
        </p>
      </div>
      <div class="flex flex-wrap gap-2">
        <button
          type="button"
          class="rounded-md border border-[var(--border-glass)] px-3 py-1.5 text-xs hover:border-[var(--accent-cyan)] disabled:opacity-40"
          disabled={connectionStore.status !== "connected" || launchingNative}
          onclick={() => void openNativeFallback()}
        >
          {launchingNative ? "Lancement…" : "Fenêtre native"}
        </button>
        <button
          type="button"
          class="rounded-md border border-[var(--border-subtle)] px-3 py-1.5 text-xs hover:border-[var(--border-focus)]"
          onclick={close}
        >
          Fermer
        </button>
      </div>
    </header>

    {#if embedReady && !embedMissing}
      <iframe
        bind:this={iframeEl}
        class="territory-view__frame"
        title="Territoire Godot"
        src="{GODOT_EMBED_PATH}?t={Date.now()}"
        allow="fullscreen"
      ></iframe>
    {:else}
      <div class="territory-view__fallback">
        <p class="text-sm">
          Export Godot non détecté dans <code class="text-xs">public/godot/</code>.
        </p>
        <p class="max-w-md text-xs">
          Lancez <code class="text-xs">scripts/export-godot-web.ps1</code> depuis la racine du
          projet pour générer l'embed WASM, ou utilisez la fenêtre native.
        </p>
        <button
          type="button"
          class="rounded-md border border-[var(--accent-cyan)]/40 px-4 py-2 text-xs text-[var(--accent-cyan)] hover:border-[var(--accent-cyan)] disabled:opacity-40"
          disabled={connectionStore.status !== "connected" || launchingNative}
          onclick={() => void openNativeFallback()}
        >
          Ouvrir Territoire (processus Godot)
        </button>
        {#if uiStore.territoryLaunchState === "running"}
          <span class="text-xs text-[var(--status-green)]">Territoire natif actif</span>
        {/if}
      </div>
    {/if}
  </div>
{/if}