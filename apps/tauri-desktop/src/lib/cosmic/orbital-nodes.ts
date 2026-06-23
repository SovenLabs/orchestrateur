import type { AgentInfo, MemoryItem } from "$lib/types/ui";

export type OrbitalNodeKind = "agent" | "memory" | "skill";

export type OrbitalNode = {
  id: string;
  kind: OrbitalNodeKind;
  label: string;
  activity: number;
  angle: number;
  orbitFactor: number;
};

export type OrbitalEdge = {
  fromId: string;
  toId: string;
  strength: number;
};

export function orbitalNodeBudget(memoryCount: number, maxCap = 16): number {
  if (memoryCount <= 0) return 8;
  return Math.min(maxCap, Math.max(10, 6 + Math.floor(memoryCount / 12)));
}

const KIND_COLORS: Record<OrbitalNodeKind, { r: number; g: number; b: number }> = {
  agent: { r: 200, g: 168, b: 208 },
  memory: { r: 160, g: 196, b: 224 },
  skill: { r: 232, g: 208, b: 176 },
};

export function buildOrbitalNodes(
  agents: AgentInfo[],
  _memories: MemoryItem[],
  skills: Array<{ name: string }>,
  maxNodes = 8,
): OrbitalNode[] {
  const nodes: OrbitalNode[] = [];
  const agentSlots = Math.min(4, Math.max(2, maxNodes - 2));

  const sortedAgents = [...agents].sort((a, b) => b.activity - a.activity);
  for (const agent of sortedAgents.slice(0, agentSlots)) {
    nodes.push({
      id: `agent-${agent.id}`,
      kind: "agent",
      label: agent.name.split(" ")[0],
      activity: agent.status === "active" ? Math.max(0.55, agent.activity) : agent.activity * 0.5,
      angle: 0,
      orbitFactor: 1,
    });
  }

  for (const skill of skills.slice(0, 2)) {
    if (nodes.length >= maxNodes) break;
    nodes.push({
      id: `skill-${skill.name}`,
      kind: "skill",
      label: skill.name.slice(0, 10),
      activity: 0.45,
      angle: 0,
      orbitFactor: 1.15,
    });
  }

  const capped = nodes.slice(0, maxNodes);
  capped.forEach((n, i) => {
    n.angle = (i / capped.length) * Math.PI * 2 - Math.PI / 2;
    n.orbitFactor += (i % 3) * 0.06;
  });
  return capped;
}

export function buildMemoryEdges(nodes: OrbitalNode[]): OrbitalEdge[] {
  const memoryIds = nodes.filter((n) => n.kind === "memory").map((n) => n.id);
  if (memoryIds.length < 2) return [];

  const edges: OrbitalEdge[] = [];
  for (let i = 0; i < memoryIds.length; i++) {
    const next = (i + 1) % memoryIds.length;
    edges.push({ fromId: memoryIds[i], toId: memoryIds[next], strength: 0.7 });
    if (memoryIds.length > 3) {
      const hop = (i + 2) % memoryIds.length;
      edges.push({ fromId: memoryIds[i], toId: memoryIds[hop], strength: 0.38 });
    }
  }
  return edges;
}

export function nodePosition(
  cx: number,
  cy: number,
  baseRadius: number,
  node: OrbitalNode,
  time: number,
  dockT: number,
): { x: number; y: number; radius: number } {
  const visibility = Math.max(0, 1 - dockT * 1.15);
  const drift = node.kind === "agent" && node.activity > 0.5 ? Math.sin(time * 1.4 + node.angle) * 0.04 : 0;
  const r = baseRadius * (0.95 + node.orbitFactor * 0.35 + drift) * (0.35 + visibility * 0.65);
  const a = node.angle + time * (0.08 + node.activity * 0.12) * visibility;
  const size = (5 + node.activity * 9) * (0.4 + visibility * 0.6);
  return {
    x: cx + Math.cos(a) * r,
    y: cy + Math.sin(a) * r * 0.88,
    radius: size,
  };
}

export function drawOrbitalEdges(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  baseRadius: number,
  nodes: OrbitalNode[],
  edges: OrbitalEdge[],
  time: number,
  dockT: number,
): void {
  const visibility = Math.max(0, 1 - dockT * 0.85);
  if (visibility < 0.08 || edges.length === 0) return;

  const byId = new Map(nodes.map((n) => [n.id, n]));
  for (const edge of edges) {
    const from = byId.get(edge.fromId);
    const to = byId.get(edge.toId);
    if (!from || !to) continue;
    const a = nodePosition(cx, cy, baseRadius, from, time, dockT);
    const b = nodePosition(cx, cy, baseRadius, to, time, dockT);
    const alpha = edge.strength * 0.42 * visibility;
    const mx = (a.x + b.x) / 2 + Math.sin(time * 0.6 + edge.strength * 4) * 8;
    const my = (a.y + b.y) / 2 + Math.cos(time * 0.5) * 6;

    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.quadraticCurveTo(mx, my, b.x, b.y);
    ctx.strokeStyle = `rgba(160,196,224,${alpha})`;
    ctx.lineWidth = 0.8 + edge.strength * 0.6;
    ctx.stroke();
  }
}

export function drawOrbitalNodes(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  baseRadius: number,
  nodes: OrbitalNode[],
  time: number,
  dockT: number,
  hoveredId: string | null = null,
  edges: OrbitalEdge[] = [],
): void {
  const memoryNodes = nodes.filter((n) => n.kind === "memory").length;
  const dockCutoff = memoryNodes > 0 ? 0.98 : 0.92;
  if (dockT > dockCutoff || nodes.length === 0) return;

  drawOrbitalEdges(ctx, cx, cy, baseRadius, nodes, edges, time, dockT);

  for (const node of nodes) {
    const pos = nodePosition(cx, cy, baseRadius, node, time, dockT);
    const c = KIND_COLORS[node.kind];
    const alpha = (0.42 + node.activity * 0.5) * (1 - dockT * (memoryNodes > 0 ? 0.55 : 0.75));
    const glow = hoveredId === node.id ? 1.35 : 1;

    ctx.beginPath();
    ctx.moveTo(cx, cy);
    ctx.lineTo(pos.x, pos.y);
    ctx.strokeStyle = `rgba(${c.r},${c.g},${c.b},${alpha * 0.22})`;
    ctx.lineWidth = 0.6;
    ctx.stroke();

    const grad = ctx.createRadialGradient(pos.x, pos.y, 0, pos.x, pos.y, pos.radius * 2.2 * glow);
    grad.addColorStop(0, `rgba(${c.r},${c.g},${c.b},${alpha})`);
    grad.addColorStop(1, `rgba(${c.r},${c.g},${c.b},0)`);
    ctx.fillStyle = grad;
    ctx.beginPath();
    ctx.arc(pos.x, pos.y, pos.radius * glow, 0, Math.PI * 2);
    ctx.fill();

    if (dockT < 0.45 && pos.radius > 5) {
      ctx.fillStyle = `rgba(240,238,244,${alpha * 0.85})`;
      ctx.font = "9px Inter, system-ui, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(node.label, pos.x, pos.y + pos.radius + 10);
    }
  }
}

export function hitTestOrbitalNode(
  cx: number,
  cy: number,
  baseRadius: number,
  nodes: OrbitalNode[],
  time: number,
  dockT: number,
  px: number,
  py: number,
): OrbitalNode | null {
  if (dockT > 0.95) return null;
  for (let i = nodes.length - 1; i >= 0; i--) {
    const node = nodes[i];
    const pos = nodePosition(cx, cy, baseRadius, node, time, dockT);
    const hitR = Math.max(12, pos.radius + 6);
    const dx = px - pos.x;
    const dy = py - pos.y;
    if (dx * dx + dy * dy <= hitR * hitR) return node;
  }
  return null;
}