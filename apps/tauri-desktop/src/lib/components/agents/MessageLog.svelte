<script lang="ts">
  import type { CommunicationLogEntry } from "$lib/stores/communication.svelte";
  import { communicationStore } from "$lib/stores/communication.svelte";

  let {
    entries,
  }: {
    entries?: CommunicationLogEntry[];
  } = $props();

  const rows = $derived(entries ?? communicationStore.log);
</script>

<ul class="max-h-56 space-y-1.5 overflow-auto scroll-thin text-xs">
  {#each rows as entry (entry.id + String(entry.at))}
    <li class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] px-3 py-2">
      <span class="font-mono text-[var(--text-muted)]">
        [{entry.kind}] {entry.from} → {entry.to}
      </span>
      <p class="mt-0.5 text-[var(--text-secondary)]">{entry.body}</p>
    </li>
  {:else}
    <li class="py-4 text-center text-[var(--text-muted)]">Aucun échange enregistré.</li>
  {/each}
</ul>