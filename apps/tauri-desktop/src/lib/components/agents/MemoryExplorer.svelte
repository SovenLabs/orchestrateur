<script lang="ts">
  import { connectionStore } from "$lib/stores/connection.svelte";

  let { agentId }: { agentId: string } = $props();

  const scoped = $derived(
    connectionStore.memories.filter(
      (m) =>
        m.tags.some((t) => t.toLowerCase().includes(agentId.toLowerCase())) ||
        m.title.toLowerCase().includes(agentId.toLowerCase()),
    ),
  );
</script>

<div>
  <h4 class="mb-2 text-xs uppercase tracking-wider text-[var(--text-muted)]">
    Mémoires liées · {agentId}
  </h4>
  <ul class="max-h-40 space-y-2 overflow-auto scroll-thin text-sm">
    {#each scoped as mem (mem.id)}
      <li class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] px-3 py-2">
        <p class="font-medium text-[var(--accent-cyan)]">{mem.title}</p>
        <p class="text-xs text-[var(--text-muted)]">
          {mem.tags.join(", ") || "sans tag"} · {mem.backlink_count} backlinks
        </p>
      </li>
    {:else}
      <li class="text-[var(--text-muted)]">
        Aucune mémoire Cortex taguée pour cet agent — assimilez via tour agent ou CLI.
      </li>
    {/each}
  </ul>
</div>