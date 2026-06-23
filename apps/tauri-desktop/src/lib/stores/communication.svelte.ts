import type { AgentMessage } from "$lib/types/ui";
import {
  filterCommunicationEdges,
  filterCommunicationLog,
} from "$lib/stores/communication-filters";
import type {
  CommunicationEdge,
  CommunicationLogEntry,
} from "$lib/stores/communication-types";

export type { CommunicationEdge, CommunicationLogEntry } from "$lib/stores/communication-types";

class CommunicationStore {
  edges = $state<CommunicationEdge[]>([]);
  log = $state<CommunicationLogEntry[]>([]);
  filterAgentId = $state<string | null>(null);

  filteredEdges = $derived.by(() =>
    filterCommunicationEdges(this.edges, this.filterAgentId),
  );

  filteredLog = $derived.by(() =>
    filterCommunicationLog(this.log, this.filterAgentId),
  );

  setFilterAgentId(id: string | null): void {
    this.filterAgentId = id;
  }

  clearFilter(): void {
    this.filterAgentId = null;
  }

  recordMessage(msg: AgentMessage): void {
    this.bumpEdge(msg.from, msg.to);
    this.pushLog({
      id: msg.id,
      kind: "message",
      from: msg.from,
      to: msg.to,
      body: msg.body,
      at: Date.now(),
    });
  }

  recordOutbound(from: string, to: string, body: string): void {
    this.bumpEdge(from, to);
    this.pushLog({
      id: crypto.randomUUID(),
      kind: "message",
      from,
      to,
      body,
      at: Date.now(),
    });
  }

  recordEvent(from: string, to: string, messageId: string): void {
    this.bumpEdge(from, to);
    this.pushLog({
      id: messageId || crypto.randomUUID(),
      kind: "event",
      from,
      to,
      body: "(événement temps réel)",
      at: Date.now(),
    });
  }

  recordTurn(agentId: string, prompt: string, reply: string): void {
    this.pushLog({
      id: crypto.randomUUID(),
      kind: "turn",
      from: "operator",
      to: agentId,
      body: prompt,
      at: Date.now(),
    });
    this.pushLog({
      id: crypto.randomUUID(),
      kind: "turn",
      from: agentId,
      to: "operator",
      body: reply.slice(0, 280),
      at: Date.now() + 1,
    });
  }

  private bumpEdge(from: string, to: string): void {
    const existing = this.edges.find((e) => e.from === from && e.to === to);
    if (existing) {
      this.edges = this.edges.map((e) =>
        e.from === from && e.to === to ? { ...e, count: e.count + 1, lastAt: Date.now() } : e,
      );
    } else {
      this.edges = [...this.edges, { from, to, count: 1, lastAt: Date.now() }].slice(-64);
    }
  }

  private pushLog(entry: CommunicationLogEntry): void {
    this.log = [entry, ...this.log].slice(0, 120);
  }
}

export const communicationStore = new CommunicationStore();