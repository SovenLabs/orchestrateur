<script lang="ts">
  import { onMount } from "svelte";
  import { kindColor } from "$lib/cosmic/cosmic-palette";
  import { connectionStore } from "$lib/stores/connection.svelte";

  let canvas: HTMLCanvasElement | undefined = $state();
  let container: HTMLDivElement | undefined = $state();

  function draw() {
    if (!canvas || !container) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const { width, height } = container.getBoundingClientRect();
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);

    const memories = connectionStore.memories;
    const count = Math.min(40, Math.max(8, memories.length || connectionStore.memoryTotal));
    const t = Date.now() / 1000;

    for (let i = 0; i < count; i++) {
      const memory = memories[i % Math.max(memories.length, 1)];
      const kind = memory?.kind ?? "context";
      const hex = kindColor(String(kind));
      const angle = (i / count) * Math.PI * 2 + t * 0.1;
      const r = 30 + (i % 5) * 18 + Math.sin(t + i) * 6;
      const x = width / 2 + Math.cos(angle) * r;
      const y = height / 2 + Math.sin(angle) * r * 0.6;
      const grad = ctx.createRadialGradient(x, y, 0, x, y, 14 + (i % 3) * 4);
      grad.addColorStop(0, hexToRgba(hex, 0.35));
      grad.addColorStop(1, hexToRgba(hex, 0));
      ctx.fillStyle = grad;
      ctx.beginPath();
      ctx.arc(x, y, 16, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  function hexToRgba(hex: string, alpha: number): string {
    const h = hex.replace("#", "");
    const r = parseInt(h.slice(0, 2), 16);
    const g = parseInt(h.slice(2, 4), 16);
    const b = parseInt(h.slice(4, 6), 16);
    return `rgba(${r},${g},${b},${alpha})`;
  }

  onMount(() => {
    const ro = new ResizeObserver(draw);
    if (container) ro.observe(container);
    const id = setInterval(draw, 120);
    draw();
    return () => {
      ro.disconnect();
      clearInterval(id);
    };
  });

  $effect(() => {
    connectionStore.memoryTotal;
    connectionStore.memories;
    draw();
  });
</script>

<div bind:this={container} class="h-40 w-full rounded-xl border border-[var(--glass-border)] bg-[var(--bg-input)]">
  <canvas bind:this={canvas} class="h-full w-full"></canvas>
</div>