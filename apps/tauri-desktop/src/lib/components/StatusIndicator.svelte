<script lang="ts">
  type Status = "ok" | "warn" | "error" | "idle";

  let {
    status = "idle",
    label,
    pulse = false,
  }: {
    status?: Status;
    label: string;
    pulse?: boolean;
  } = $props();

  const colors: Record<Status, string> = {
    ok: "var(--status-green)",
    warn: "var(--status-orange)",
    error: "var(--status-red)",
    idle: "var(--text-muted)",
  };
</script>

<span
  class="inline-flex items-center gap-2 rounded-full border border-[var(--border-subtle)] px-3 py-1 text-xs text-[var(--text-secondary)]"
  role="status"
  aria-label={label}
>
  <span
    class="h-2 w-2 rounded-full {pulse ? 'status-pulse' : ''}"
    style="background: {colors[status]}"
  ></span>
  {label}
</span>