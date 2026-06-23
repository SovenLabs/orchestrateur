<script lang="ts">
  import KindBadge from "$lib/components/KindBadge.svelte";
  import { connectionStore } from "$lib/stores/connection.svelte";
  import type { DraftItem } from "$lib/types/ui";

  const disabled = $derived(connectionStore.status !== "connected");

  async function publish(draft: DraftItem) {
    await connectionStore.publishDraft(draft.id);
  }

  async function discard(draft: DraftItem) {
    await connectionStore.discardDraft(draft.id);
  }
</script>

<section class="drafts-panel space-y-3">
  <header class="flex items-center justify-between gap-2">
    <div>
      <h3 class="text-xs uppercase tracking-wider text-[var(--text-muted)]">Brouillons</h3>
      <p class="text-[10px] text-[var(--text-muted)]">
        {connectionStore.drafts.length} en attente — revue avant publication
      </p>
    </div>
    <button
      type="button"
      class="rounded-lg border border-[var(--border-subtle)] px-3 py-1.5 text-xs hover:border-[var(--border-focus)] disabled:opacity-40"
      onclick={() => connectionStore.fetchDrafts()}
      disabled={disabled}
    >
      Rafraîchir
    </button>
  </header>

  <ul class="space-y-2">
    {#each connectionStore.drafts as draft (draft.id)}
      <li class="rounded-xl border border-[var(--glass-border)] bg-[var(--bg-input)] p-3">
        <div class="mb-2 flex flex-wrap items-center gap-2">
          <KindBadge kind={draft.kind} compact />
          <p class="min-w-0 flex-1 truncate text-sm font-medium text-[var(--text-primary)]">
            {draft.title}
          </p>
        </div>
        {#if draft.source_session}
          <p class="mb-2 font-mono text-[10px] text-[var(--text-muted)]">{draft.source_session}</p>
        {/if}
        {#if draft.tags.length > 0}
          <div class="mb-2 flex flex-wrap gap-1">
            {#each draft.tags as tag}
              <span class="rounded bg-[var(--accent-cyan)]/10 px-1.5 py-0.5 text-[10px] text-[var(--accent-cyan)]">{tag}</span>
            {/each}
          </div>
        {/if}
        <div class="flex gap-2">
          <button
            type="button"
            class="rounded-lg bg-[var(--accent-cyan)]/15 px-3 py-1.5 text-xs text-[var(--accent-cyan)] hover:bg-[var(--accent-cyan)]/25 disabled:opacity-40"
            onclick={() => publish(draft)}
            disabled={disabled}
          >
            Publier
          </button>
          <button
            type="button"
            class="rounded-lg border border-[var(--border-subtle)] px-3 py-1.5 text-xs text-[var(--text-muted)] hover:border-[var(--border-focus)] disabled:opacity-40"
            onclick={() => discard(draft)}
            disabled={disabled}
          >
            Ignorer
          </button>
        </div>
      </li>
    {:else}
      <li class="rounded-xl bg-[var(--bg-input)] px-4 py-8 text-center text-sm text-[var(--text-muted)]">
        {disabled ? "Connectez le daemon pour lister les brouillons." : "Aucun brouillon — le watcher créera des drafts à la fin de session."}
      </li>
    {/each}
  </ul>
</section>