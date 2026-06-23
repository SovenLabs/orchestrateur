<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    subtitle,
    class: className = "",
    children,
    header,
  }: {
    title?: string;
    subtitle?: string;
    class?: string;
    children?: Snippet;
    header?: Snippet;
  } = $props();
</script>

<section class="glow-card p-4 {className}">
  {#if header}
    {@render header()}
  {:else if title}
    <header class="mb-3 flex items-start justify-between gap-2">
      <div>
        <h3 class="text-sm font-medium text-[var(--accent-cyan)]">{title}</h3>
        {#if subtitle}
          <p class="mt-0.5 text-xs text-[var(--text-muted)]">{subtitle}</p>
        {/if}
      </div>
    </header>
  {/if}
  {#if children}
    {@render children()}
  {/if}
</section>