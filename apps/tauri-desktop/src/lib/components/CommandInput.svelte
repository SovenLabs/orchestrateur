<script lang="ts">
  let {
    value = $bindable(""),
    placeholder = "Commander l'orchestrateur…",
    disabled = false,
    onSubmit,
  }: {
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    onSubmit?: (text: string) => void;
  } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" || e.key === "Esc") {
      e.preventDefault();
      (e.currentTarget as HTMLInputElement).blur();
      return;
    }
    if (e.key === "Enter" && !e.shiftKey && value.trim() && onSubmit) {
      e.preventDefault();
      onSubmit(value.trim());
      value = "";
    }
  }
</script>

<div class="relative">
  <span class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-[var(--accent-cyan-dim)]">›</span>
  <input
    type="text"
    class="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] py-2.5 pl-8 pr-4 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] transition-colors focus:border-[var(--border-focus)] disabled:cursor-not-allowed disabled:opacity-50"
    {placeholder}
    bind:value
    {disabled}
    onkeydown={handleKeydown}
    aria-label="Commande"
  />
</div>