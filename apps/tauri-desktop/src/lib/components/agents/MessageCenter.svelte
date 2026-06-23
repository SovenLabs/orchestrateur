<script lang="ts">
  import { agentsStore } from "$lib/stores/agents.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";

  let {
    agentId,
    agentName = agentId,
  }: {
    agentId: string;
    agentName?: string;
  } = $props();

  let tab = $state<"inbox" | "send" | "talk">("inbox");
  let fromId = $state("");
  let toId = $state("");
  let body = $state("");
  let talkPrompt = $state("");

  const disabled = $derived(
    connectionStore.status !== "connected" || agentsStore.actionPending,
  );

  const peers = $derived(agentsStore.agents.filter((a) => a.id !== agentId));

  $effect(() => {
    toId = agentId;
    if (!fromId && peers.length > 0) fromId = peers[0].id;
  });
</script>

<div class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] p-3">
  <div class="mb-3 flex gap-1" role="tablist">
    {#each [["inbox", "Inbox"], ["send", "Envoyer"], ["talk", "Parler"]] as [id, label] (id)}
      <button
        type="button"
        role="tab"
        aria-selected={tab === id}
        class="rounded px-2 py-1 text-xs
          {tab === id ? 'bg-[var(--accent-cyan)]/15 text-[var(--accent-cyan)]' : 'text-[var(--text-muted)]'}"
        onclick={() => (tab = id as typeof tab)}
      >
        {label}
      </button>
    {/each}
  </div>

  {#if tab === "inbox"}
    <ul class="max-h-44 space-y-2 overflow-auto scroll-thin text-sm">
      {#each agentsStore.inbox as msg (msg.id)}
        <li class="rounded border border-[var(--border-subtle)] px-2 py-1.5">
          <p class="text-xs text-[var(--text-muted)]">{msg.from} → {msg.to}</p>
          <p>{msg.body}</p>
        </li>
      {:else}
        <li class="text-[var(--text-muted)]">Inbox vide.</li>
      {/each}
    </ul>
  {:else if tab === "send"}
    <div class="space-y-2 text-sm">
      <label class="block text-xs text-[var(--text-muted)]">
        De
        <select class="mt-1 w-full rounded border border-[var(--border-subtle)] bg-transparent px-2 py-1" bind:value={fromId}>
          {#each agentsStore.agents as a (a.id)}
            <option value={a.id}>{a.name}</option>
          {/each}
        </select>
      </label>
      <label class="block text-xs text-[var(--text-muted)]">
        Vers
        <select class="mt-1 w-full rounded border border-[var(--border-subtle)] bg-transparent px-2 py-1" bind:value={toId}>
          {#each agentsStore.agents as a (a.id)}
            <option value={a.id}>{a.name}</option>
          {/each}
        </select>
      </label>
      <textarea
        class="w-full rounded border border-[var(--border-subtle)] bg-transparent px-2 py-1.5 text-sm"
        rows="3"
        placeholder="Message inter-agent…"
        bind:value={body}
      ></textarea>
      <button
        type="button"
        class="rounded border border-[var(--border-subtle)] px-3 py-1 text-xs disabled:opacity-50"
        disabled={disabled || !body.trim()}
        onclick={() => void agentsStore.sendMessage(fromId, toId, body).then(() => (body = ""))}
      >
        Envoyer
      </button>
    </div>
  {:else}
    <div class="space-y-2 text-sm">
      <p class="text-xs text-[var(--text-muted)]">Tour LLM avec {agentName}</p>
      <textarea
        class="w-full rounded border border-[var(--border-subtle)] bg-transparent px-2 py-1.5 text-sm"
        rows="3"
        placeholder="Consigne pour l'agent…"
        bind:value={talkPrompt}
      ></textarea>
      <button
        type="button"
        class="rounded border border-[var(--border-subtle)] px-3 py-1 text-xs disabled:opacity-50"
        disabled={disabled || !talkPrompt.trim()}
        onclick={() => void agentsStore.talkToAgent(agentId, talkPrompt).then(() => (talkPrompt = ""))}
      >
        Parler
      </button>
    </div>
  {/if}
</div>