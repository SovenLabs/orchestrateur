<script lang="ts">
  import CommandInput from "$lib/components/CommandInput.svelte";
  import BlackHoleCanvas from "$lib/cosmic/BlackHoleCanvas.svelte";
  import CosmicHitOverlay from "$lib/cosmic/CosmicHitOverlay.svelte";
  import CosmicLegend from "$lib/cosmic/CosmicLegend.svelte";
  import type { CosmicHit } from "$lib/cosmic/cosmic-model";
  import NuanceOrb from "$lib/cosmic/NuanceOrb.svelte";
  import { blackholeStore } from "$lib/stores/blackhole.svelte";
  import { cosmicStore } from "$lib/stores/cosmic-store.svelte";
  import EnrichedMessageToggle from "$lib/components/EnrichedMessageToggle.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";

  let scrollEl: HTMLDivElement | undefined = $state();
  let inputValue = $state("");
  let cosmicHits = $state<CosmicHit[]>([]);
  let exportCanvas = $state<HTMLCanvasElement | undefined>();
  let stageWidth = $state(0);
  let stageHeight = $state(0);

  const disabled = $derived(connectionStore.status !== "connected" || connectionStore.chatPending);
  const orbVisible = $derived(blackholeStore.state === "docked");

  function onScroll() {
    if (!scrollEl) return;
    blackholeStore.updateFromScroll(scrollEl.scrollTop, scrollEl.scrollHeight, scrollEl.clientHeight);
  }

  function scrollToBottom() {
    if (!scrollEl) return;
    scrollEl.scrollTop = scrollEl.scrollHeight;
    blackholeStore.expand();
  }

  async function sendMessage(text: string) {
    blackholeStore.triggerFeedBurst();
    await connectionStore.sendChat(text);
    requestAnimationFrame(scrollToBottom);
  }

  $effect(() => {
    if (connectionStore.chatMessages.length && scrollEl) {
      requestAnimationFrame(scrollToBottom);
    }
  });
</script>

<div class="relative flex h-full min-h-0 flex-col" bind:clientWidth={stageWidth} bind:clientHeight={stageHeight}>
  <BlackHoleCanvas bind:exportCanvasRef={exportCanvas} onHits={(hits) => (cosmicHits = hits)} />
  <CosmicHitOverlay
    hits={cosmicHits}
    stageWidth={stageWidth}
    stageHeight={stageHeight}
    dockT={blackholeStore.state === "docked" ? 1 : 0}
  />
  <CosmicLegend
    {exportCanvas}
    stageCenterX={stageWidth * 0.5}
    stageCenterY={stageHeight * 0.5}
  />

  <NuanceOrb visible={orbVisible} onactivate={scrollToBottom} />

  <div
    bind:this={scrollEl}
    class="chat-stream pointer-events-none relative z-10 mx-auto flex w-full max-w-2xl flex-1 flex-col gap-3 overflow-auto scroll-thin px-5 pb-36 pt-28"
    onscroll={onScroll}
  >
    {#each connectionStore.chatMessages as msg (msg.id)}
      <article
        class="glass-message pointer-events-auto rounded-xl px-4 py-3 text-sm
          {msg.role === 'user' ? 'ml-8' : msg.role === 'assistant' ? 'mr-8' : ''}"
      >
        <p class="mb-1.5 text-[10px] uppercase tracking-wider text-[var(--text-muted)]">{msg.role}</p>
        {#if msg.role === "user" && msg.enrichedContent}
          <EnrichedMessageToggle original={msg.content} enriched={msg.enrichedContent} />
        {:else}
          <p class="whitespace-pre-wrap leading-relaxed text-[var(--text-primary)]">{msg.content}</p>
        {/if}
      </article>
    {:else}
      <p class="chat-stream__empty">
        Parlez au centre — les neurones autour du trou noir réagissent à chaque pensée.
      </p>
    {/each}
    {#if connectionStore.chatPending}
      <p class="text-xs text-[var(--sky)] status-pulse">L'agent réfléchit…</p>
    {/if}
  </div>

  <div class="smart-input-dock pointer-events-none absolute inset-x-0 bottom-0 z-20 flex justify-center pb-8 pt-20">
    <div class="pointer-events-auto w-full max-w-xl px-5">
      <div class="glass-panel rounded-2xl p-2 shadow-2xl">
        <CommandInput
          bind:value={inputValue}
          placeholder="Nourrir le second brain…"
          {disabled}
          onSubmit={sendMessage}
        />
      </div>
    </div>
  </div>
</div>