import type { CommunicationEdge, CommunicationLogEntry } from "./communication-types";

export function filterCommunicationEdges(
  edges: CommunicationEdge[],
  agentId: string | null,
): CommunicationEdge[] {
  if (!agentId) return edges;
  return edges.filter((e) => e.from === agentId || e.to === agentId);
}

export function filterCommunicationLog(
  log: CommunicationLogEntry[],
  agentId: string | null,
): CommunicationLogEntry[] {
  if (!agentId) return log;
  return log.filter((e) => e.from === agentId || e.to === agentId);
}