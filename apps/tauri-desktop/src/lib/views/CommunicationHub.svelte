<script lang="ts">
  import MessageLog from "$lib/components/agents/MessageLog.svelte";
  import { agentsStore } from "$lib/stores/agents.svelte";
  import { communicationStore } from "$lib/stores/communication.svelte";

  const nodes = $derived(agentsStore.agents);
  const edges = $derived(communicationStore.filteredEdges);
  const activeFilter = $derived(communicationStore.filterAgentId);

  function nodePos(index: number, total: number): { x: number; y: number } {
    const angle = (index / Math.max(1, total)) * Math.PI * 2 - Math.PI / 2;
    return { x: 150 + Math.cos(angle) * 100, y: 120 + Math.sin(angle) * 80 };
  }

  function findIndex(id: string): number {
    const idx = nodes.findIndex((n) => n.id === id);
    return idx >= 0 ? idx : 0;
  }

  function toggleFilter(id: string): void {
    if (activeFilter === id) {
      communicationStore.clearFilter();
    } else {
      communicationStore.setFilterAgentId(id);
    }
  }
</script>

<div class="eh-card space-y-4 p-4">
  <header class="flex items-center justify-between gap-2">
    <div>
      <button
        type="button"
        class="text-xs text-[var(--accent-cyan)] hover:underline"
        onclick={() => agentsStore.backToList()}
      >
        ← Retour sub-agents
      </button>
      <h2 class="text-sm font-medium text-[var(--accent-cyan)]">Communication Hub</h2>
      <p class="text-xs text-[var(--text-muted)]">Arcs synaptiques · filtre par agent</p>
    </div>
    <span class="text-xs text-[var(--text-muted)]">
      {edges.length} lien{edges.length === 1 ? "" : "s"}
      {#if activeFilter}
        · filtre <span class="font-mono text-[var(--accent-hot)]">{activeFilter}</span>
      {/if}
    </span>
  </header>

  <div class="comm-hub__filter" role="group" aria-label="Filtrer par agent">
    <button
      type="button"
      class="comm-hub__chip {activeFilter === null ? 'comm-hub__chip--active' : ''}"
      onclick={() => communicationStore.clearFilter()}
    >
      Tous
    </button>
    {#each nodes as node (node.id)}
      <button
        type="button"
        class="comm-hub__chip {activeFilter === node.id ? 'comm-hub__chip--active' : ''}"
        onclick={() => toggleFilter(node.id)}
      >
        {node.name || node.id}
      </button>
    {/each}
  </div>

  <div class="rounded-lg border border-[var(--border-glass)] bg-[var(--bg-deep)] p-2">
    <svg viewBox="0 0 300 240" class="h-56 w-full" role="img" aria-label="Graphe agents">
      {#each edges as edge (edge.from + edge.to)}
        {@const a = nodePos(findIndex(edge.from), nodes.length)}
        {@const b = nodePos(findIndex(edge.to), nodes.length)}
        <line
          class="comm-hub__edge"
          x1={a.x}
          y1={a.y}
          x2={b.x}
          y2={b.y}
          stroke-width={Math.min(4, 1 + edge.count * 0.4)}
        />
      {/each}
      {#each nodes as node, i (node.id)}
        {@const p = nodePos(i, nodes.length)}
        {@const highlighted = !activeFilter || activeFilter === node.id}
        <circle
          class="comm-hub__node-ring"
          cx={p.x}
          cy={p.y}
          r="18"
          opacity={highlighted ? 1 : 0.2}
        />
        <circle
          cx={p.x}
          cy={p.y}
          r="14"
          fill="var(--bg-card)"
          stroke={highlighted ? "var(--accent-cyan)" : "var(--text-muted)"}
          stroke-width="1.5"
          opacity={highlighted ? 1 : 0.35}
        />
        <text
          x={p.x}
          y={p.y + 26}
          text-anchor="middle"
          fill="var(--text-muted)"
          font-size="9"
          opacity={highlighted ? 1 : 0.4}
        >
          {node.id.slice(0, 10)}
        </text>
      {/each}
    </svg>
  </div>

  <section>
    <h3 class="mb-2 text-xs uppercase tracking-wider text-[var(--text-muted)]">Journal</h3>
    <MessageLog entries={communicationStore.filteredLog} />
  </section>
</div>