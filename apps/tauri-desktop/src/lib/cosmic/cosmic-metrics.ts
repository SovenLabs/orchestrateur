import type { AgentInfo } from "$lib/types/ui";
import { isAgentAwake } from "$lib/ws/bridge";

export function computeAgentsSync(agents: AgentInfo[]): { active: number; total: number; label: string } {
  const total = agents.length;
  const active = agents.filter((a) => isAgentAwake(a.status)).length;
  return { active, total, label: `${active}/${total}` };
}

/** Cohérence globale — heuristique locale (connexion + santé + activité agents). */
export function computeCoherence(params: {
  connected: boolean;
  llmAvailable: boolean;
  embeddingAvailable: boolean;
  agentActivity: number;
  nuanceDepth: number;
}): number {
  if (!params.connected) return 0;
  let score = 72;
  if (params.llmAvailable) score += 10;
  if (params.embeddingAvailable) score += 8;
  score += params.agentActivity * 6;
  score += params.nuanceDepth * 4;
  return Math.min(99.9, Math.round(score * 10) / 10);
}