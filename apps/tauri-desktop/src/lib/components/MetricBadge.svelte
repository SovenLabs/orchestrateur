<script lang="ts">
  let {
    label,
    value,
    unit = "",
    trend,
  }: {
    label: string;
    value: string | number;
    unit?: string;
    trend?: "up" | "down" | "flat";
  } = $props();

  const trendColor = {
    up: "text-[var(--status-green)]",
    down: "text-[var(--status-red)]",
    flat: "text-[var(--text-muted)]",
  };
</script>

<div class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] px-3 py-2">
  <p class="text-[10px] uppercase tracking-wider text-[var(--text-muted)]">{label}</p>
  <p class="mt-1 font-mono text-lg text-[var(--text-primary)]">
    {value}{#if unit}<span class="ml-0.5 text-xs text-[var(--text-muted)]">{unit}</span>{/if}
  </p>
  {#if trend}
    <p class="mt-0.5 text-[10px] {trendColor[trend]}">
      {trend === "up" ? "↑" : trend === "down" ? "↓" : "—"} live
    </p>
  {/if}
</div>