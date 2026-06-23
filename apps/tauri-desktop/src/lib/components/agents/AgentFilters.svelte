<script lang="ts">
  import { agentsStore } from "$lib/stores/agents.svelte";

  const roles = $derived(
    [...new Set(agentsStore.agents.map((a) => a.role).filter(Boolean))] as string[],
  );
</script>

<div class="flex flex-wrap items-end gap-2">
  <label class="flex min-w-[140px] flex-1 flex-col gap-1 text-xs text-[var(--text-muted)]">
    Recherche
    <input
      type="search"
      class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] px-2 py-1.5 text-sm text-[var(--text-primary)]"
      placeholder="id, nom, rôle…"
      bind:value={agentsStore.filters.query}
    />
  </label>
  <label class="flex flex-col gap-1 text-xs text-[var(--text-muted)]">
    Statut
    <select
      class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] px-2 py-1.5 text-sm"
      bind:value={agentsStore.filters.status}
    >
      <option value="all">Tous</option>
      <option value="awake">awake</option>
      <option value="background">background</option>
      <option value="sleeping">sleeping</option>
    </select>
  </label>
  <label class="flex min-w-[120px] flex-col gap-1 text-xs text-[var(--text-muted)]">
    Rôle
    <input
      list="agent-roles"
      class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] px-2 py-1.5 text-sm"
      placeholder="filtrer…"
      bind:value={agentsStore.filters.role}
    />
    <datalist id="agent-roles">
      {#each roles as role (role)}
        <option value={role}></option>
      {/each}
    </datalist>
  </label>
</div>