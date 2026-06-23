<script lang="ts">
  import GlowCard from "$lib/components/GlowCard.svelte";
  import KindBadge from "$lib/components/KindBadge.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import { MEMORY_KIND_LABELS, type MemoryKindId } from "$lib/cosmic/cosmic-palette";

  let search = $state("");
  let kindFilter = $state<string>("all");

  const kindOptions = Object.keys(MEMORY_KIND_LABELS) as MemoryKindId[];

  const filtered = $derived(
    connectionStore.memories.filter((m) => {
      const q = search.toLowerCase();
      const textMatch =
        m.title.toLowerCase().includes(q) ||
        m.tags.some((t) => t.toLowerCase().includes(q));
      const kindMatch = kindFilter === "all" || (m.kind ?? "context") === kindFilter;
      return textMatch && kindMatch;
    }),
  );

  const disabled = $derived(connectionStore.status !== "connected");
</script>

<div class="panel-enter space-y-4">
  <div class="flex flex-wrap items-center gap-3">
    <input
      type="search"
      class="min-w-[200px] flex-1 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] px-3 py-2 text-sm outline-none focus:border-[var(--border-focus)] disabled:opacity-50"
      placeholder="Rechercher mémoires (titre, tags)…"
      bind:value={search}
      disabled={disabled}
    />
    <select
      class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] px-3 py-2 text-sm disabled:opacity-50"
      bind:value={kindFilter}
      disabled={disabled}
      aria-label="Filtrer par kind"
    >
      <option value="all">Tous kinds</option>
      {#each kindOptions as k (k)}
        <option value={k}>{MEMORY_KIND_LABELS[k]}</option>
      {/each}
    </select>
    <button
      type="button"
      class="rounded-lg border border-[var(--border-subtle)] px-4 py-2 text-sm hover:border-[var(--border-focus)] disabled:opacity-40"
      onclick={() => connectionStore.fetchMemories()}
      disabled={disabled}
    >
      Rafraîchir
    </button>
  </div>

  <GlowCard title="Memory Explorer" subtitle="{connectionStore.memoryTotal} mémoires dans le Cortex">
    <ul class="divide-y divide-[var(--border-subtle)]">
      {#each filtered as memory (memory.id)}
        <li class="flex items-start justify-between gap-4 py-3 first:pt-0 last:pb-0">
          <div class="min-w-0">
            <div class="mb-1 flex flex-wrap items-center gap-2">
              <KindBadge kind={memory.kind ?? "context"} compact />
              <p class="truncate font-medium text-[var(--text-primary)]">{memory.title}</p>
            </div>
            <p class="mt-0.5 font-mono text-[10px] text-[var(--text-muted)]">{memory.id}</p>
            {#if memory.tags.length > 0}
              <div class="mt-2 flex flex-wrap gap-1">
                {#each memory.tags as tag}
                  <span class="rounded bg-[var(--accent-cyan)]/10 px-1.5 py-0.5 text-[10px] text-[var(--accent-cyan)]">{tag}</span>
                {/each}
              </div>
            {/if}
          </div>
          <div class="shrink-0 text-right text-xs text-[var(--text-muted)]">
            <p>{memory.backlink_count} links</p>
            <p class="mt-1">{new Date(memory.updated_at).toLocaleDateString("fr-FR")}</p>
          </div>
        </li>
      {:else}
        <li class="py-8 text-center text-sm text-[var(--text-muted)]">
          {disabled ? "Connectez le daemon pour lister les mémoires." : "Aucune mémoire — assimilez du contenu via Chat."}
        </li>
      {/each}
    </ul>
  </GlowCard>
</div>