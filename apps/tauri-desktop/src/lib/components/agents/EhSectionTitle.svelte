<script lang="ts">
  let {
    title,
    subtitle,
  }: {
    title: string;
    subtitle?: string;
  } = $props();
</script>

<header class="eh-section-title">
  <div class="eh-section-title__pulse" aria-hidden="true"></div>
  <h3 class="eh-section-title__text">{title}</h3>
  {#if subtitle}
    <p class="eh-section-title__sub">{subtitle}</p>
  {/if}
</header>