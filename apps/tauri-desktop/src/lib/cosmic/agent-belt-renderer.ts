import type { AgentInfo } from "$lib/types/ui";

export type AgentBeltParams = {
  cx: number;
  cy: number;
  baseRadius: number;
  time: number;
  dockT: number;
  scale: number;
  agents: AgentInfo[];
};

export function drawAgentBelt(ctx: CanvasRenderingContext2D, p: AgentBeltParams): void {
  if (p.dockT > 0.92 || p.agents.length === 0) return;

  const visibility = Math.max(0, 1 - p.dockT * 0.9);
  const beltR = p.baseRadius * 0.22 * p.scale;

  for (let i = 0; i < p.agents.length; i++) {
    const agent = p.agents[i];
    const speed = 0.06 + agent.activity * 0.14;
    const a = (i / p.agents.length) * Math.PI * 2 - Math.PI / 2 + p.time * speed;
    const x = p.cx + Math.cos(a) * beltR;
    const y = p.cy + Math.sin(a) * beltR * 0.88;
    const size = 4 + agent.activity * 5;
    const alpha = (0.45 + agent.activity * 0.4) * visibility;
    const color =
      agent.id === "esprit"
        ? { r: 200, g: 168, b: 208 }
        : { r: 160, g: 196, b: 224 };

    const grad = ctx.createRadialGradient(x, y, 0, x, y, size * 2.2);
    grad.addColorStop(0, `rgba(${color.r},${color.g},${color.b},${alpha})`);
    grad.addColorStop(1, "rgba(120,160,200,0)");
    ctx.fillStyle = grad;
    ctx.beginPath();
    ctx.arc(x, y, size * 2.2, 0, Math.PI * 2);
    ctx.fill();

    ctx.beginPath();
    ctx.arc(x, y, size, 0, Math.PI * 2);
    ctx.fillStyle = `rgba(230,236,244,${alpha})`;
    ctx.fill();

    if (p.dockT < 0.25 && agent.activity > 0.3) {
      ctx.fillStyle = `rgba(230,236,244,${alpha * 0.5})`;
      ctx.font = "7px Inter, system-ui, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(agent.name.split(" ")[0], x, y + size + 8);
    }
  }
}