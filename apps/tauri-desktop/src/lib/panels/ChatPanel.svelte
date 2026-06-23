<script lang="ts">
  import CommandInput from "$lib/components/CommandInput.svelte";
  import GlowCard from "$lib/components/GlowCard.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";

  const disabled = $derived(connectionStore.status !== "connected" || connectionStore.chatPending);

  function sendMessage(text: string) {
    void connectionStore.sendChat(text);
  }
</script>

<div class="panel-enter flex h-[calc(100vh-11rem)] flex-col gap-4">
  <GlowCard title="Thought Stream" subtitle="Chat avec l'Esprit — bridge daemon" class="flex min-h-0 flex-1 flex-col">
    <div class="flex min-h-0 flex-1 flex-col">
      <div class="mb-3 flex-1 space-y-3 overflow-auto scroll-thin pr-1">
        {#each connectionStore.chatMessages as msg (msg.id)}
          <div
            class="rounded-lg px-3 py-2 text-sm
              {msg.role === 'user'
              ? 'ml-8 bg-[var(--accent-cyan)]/10 text-[var(--text-primary)]'
              : msg.role === 'assistant'
                ? 'mr-8 bg-[var(--bg-input)] text-[var(--text-secondary)]'
                : 'bg-amber-500/10 text-amber-200'}"
          >
            <p class="mb-1 text-[10px] uppercase tracking-wider text-[var(--text-muted)]">{msg.role}</p>
            <p class="whitespace-pre-wrap">{msg.content}</p>
          </div>
        {:else}
          <p class="py-12 text-center text-sm text-[var(--text-muted)]">
            Posez une question à l'orchestrateur. Chaque tour peut enrichir le Cortex.
          </p>
        {/each}
        {#if connectionStore.chatPending}
          <p class="text-xs text-[var(--accent-cyan)] status-pulse">L'agent réfléchit…</p>
        {/if}
      </div>
      <CommandInput
        placeholder="Message à l'Esprit… (Entrée pour envoyer)"
        disabled={disabled}
        onSubmit={sendMessage}
      />
    </div>
  </GlowCard>
</div>