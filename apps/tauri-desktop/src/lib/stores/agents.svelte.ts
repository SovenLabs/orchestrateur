import type { DaemonWebSocketClient } from "$lib/ws/client";
import {
  agentStatusToActivity,
  parseAgentDetail,
  parseAgentList,
  parseAgentMessages,
} from "$lib/ws/bridge";
import type { AgentInfo, AgentMessage } from "$lib/types/ui";
import { communicationStore } from "$lib/stores/communication.svelte";
import { notificationsStore } from "$lib/stores/notifications.svelte";

export type AgentsViewMode = "list" | "detail" | "communication";

export type AgentFilters = {
  query: string;
  status: "all" | "awake" | "sleeping" | "background";
  role: string;
};

class AgentsStore {
  agents = $state<AgentInfo[]>([]);
  loading = $state(false);
  selectedId = $state<string | null>(null);
  selected = $state<AgentInfo | null>(null);
  inbox = $state<AgentMessage[]>([]);
  actionPending = $state(false);
  viewMode = $state<AgentsViewMode>("list");
  filters = $state<AgentFilters>({ query: "", status: "all", role: "" });

  private client: (() => DaemonWebSocketClient | null) | null = null;
  private pollTimer: ReturnType<typeof setInterval> | null = null;

  bindClient(getter: () => DaemonWebSocketClient | null): void {
    this.client = getter;
  }

  filteredAgents = $derived.by(() => {
    const q = this.filters.query.trim().toLowerCase();
    const role = this.filters.role.trim().toLowerCase();
    return this.agents.filter((a) => {
      if (this.filters.status !== "all" && a.status !== this.filters.status) return false;
      if (role && !(a.role ?? "").toLowerCase().includes(role)) return false;
      if (!q) return true;
      return (
        a.id.toLowerCase().includes(q) ||
        a.name.toLowerCase().includes(q) ||
        (a.role ?? "").toLowerCase().includes(q) ||
        (a.model ?? "").toLowerCase().includes(q)
      );
    });
  });

  startPolling(): void {
    this.stopPolling();
    this.pollTimer = setInterval(() => this.fetchAll(), 15_000);
  }

  stopPolling(): void {
    if (this.pollTimer) {
      clearInterval(this.pollTimer);
      this.pollTimer = null;
    }
  }

  fetchAll(): void {
    const c = this.client?.();
    if (!c) return;
    this.loading = true;
    c.listAgents();
  }

  select(id: string | null): void {
    this.selectedId = id;
    this.selected = id ? (this.agents.find((a) => a.id === id) ?? null) : null;
    this.inbox = [];
    this.viewMode = id ? "detail" : "list";
    if (id) {
      const c = this.client?.();
      c?.getAgent(id);
      c?.agentMessages(id);
    }
  }

  openDetail(id: string): void {
    this.select(id);
    this.viewMode = "detail";
  }

  openCommunicationGraph(): void {
    this.viewMode = "communication";
  }

  backToList(): void {
    this.viewMode = "list";
  }

  refreshSelected(): void {
    const id = this.selectedId;
    const c = this.client?.();
    if (!id || !c) return;
    c.getAgent(id);
    c.agentMessages(id);
  }

  async wake(id: string): Promise<void> {
    const c = this.client?.();
    if (!c || this.actionPending) return;
    this.actionPending = true;
    try {
      await c.executeAsync({ command: "AgentWake", payload: { id } });
      this.fetchAll();
      if (this.selectedId === id) c.getAgent(id);
      notificationsStore.push("info", `Agent ${id} réveillé`);
    } catch (err) {
      notificationsStore.push("error", err instanceof Error ? err.message : "Échec réveil");
    } finally {
      this.actionPending = false;
    }
  }

  async sleep(id: string): Promise<void> {
    const c = this.client?.();
    if (!c || this.actionPending) return;
    this.actionPending = true;
    try {
      await c.executeAsync({ command: "AgentSleep", payload: { id } });
      this.fetchAll();
      if (this.selectedId === id) c.getAgent(id);
      notificationsStore.push("info", `Agent ${id} en veille`);
    } catch (err) {
      notificationsStore.push("error", err instanceof Error ? err.message : "Échec veille");
    } finally {
      this.actionPending = false;
    }
  }

  async deleteAgent(id: string): Promise<void> {
    const c = this.client?.();
    if (!c || this.actionPending) return;
    this.actionPending = true;
    try {
      await c.executeAsync({ command: "AgentDelete", payload: { id } });
      this.agents = this.agents.filter((a) => a.id !== id);
      if (this.selectedId === id) this.select(null);
      notificationsStore.push("warn", `Agent ${id} supprimé`);
    } catch (err) {
      notificationsStore.push("error", err instanceof Error ? err.message : "Échec suppression");
    } finally {
      this.actionPending = false;
    }
  }

  async sendMessage(from: string, to: string, body: string): Promise<void> {
    const c = this.client?.();
    if (!c || this.actionPending || !body.trim()) return;
    this.actionPending = true;
    try {
      await c.executeAsync({
        command: "AgentSendMessage",
        payload: { from, to, body: body.trim() },
      });
      communicationStore.recordOutbound(from, to, body.trim());
      if (this.selectedId === to) this.refreshSelected();
      notificationsStore.push("info", `Message ${from} → ${to}`);
    } catch (err) {
      notificationsStore.push("error", err instanceof Error ? err.message : "Échec envoi");
    } finally {
      this.actionPending = false;
    }
  }

  async talkToAgent(id: string, message: string): Promise<void> {
    const c = this.client?.();
    if (!c || this.actionPending || !message.trim()) return;
    this.actionPending = true;
    try {
      const response = await c.executeAsync({
        command: "AgentTurn",
        payload: { id, message: message.trim() },
      });
      const payload = response.payload as Record<string, unknown> | undefined;
      const reply = String(payload?.reply ?? "");
      if (reply) {
        communicationStore.recordTurn(id, message.trim(), reply);
      }
      notificationsStore.push("info", `Tour agent ${id} terminé`);
    } catch (err) {
      notificationsStore.push("error", err instanceof Error ? err.message : "Échec tour agent");
    } finally {
      this.actionPending = false;
    }
  }

  handleBridgeResponse(response: Record<string, unknown>): void {
    if (response.response === "AgentList") {
      this.agents = parseAgentList(response);
      this.loading = false;
      if (this.selectedId) {
        this.selected =
          this.agents.find((a) => a.id === this.selectedId) ?? this.selected;
      }
    }

    const detail = parseAgentDetail(response);
    if (detail) {
      this.agents = this.agents.map((a) => (a.id === detail.id ? { ...a, ...detail } : a));
      if (this.selectedId === detail.id) this.selected = detail;
    }

    if (response.response === "AgentMessages") {
      this.inbox = parseAgentMessages(response);
      const unread = this.inbox.filter((m) => !m.read).length;
      if (this.selectedId) {
        this.agents = this.agents.map((a) =>
          a.id === this.selectedId ? { ...a, unreadInbox: unread } : a,
        );
        if (this.selected?.id === this.selectedId) {
          this.selected = { ...this.selected, unreadInbox: unread };
        }
      }
      for (const msg of this.inbox) {
        communicationStore.recordMessage(msg);
      }
    }

    if (response.response === "AgentDeleted") {
      const payload = response.payload as { id?: string } | undefined;
      const id = String(payload?.id ?? "");
      if (id) {
        this.agents = this.agents.filter((a) => a.id !== id);
        if (this.selectedId === id) this.select(null);
      }
    }
  }

  applyStatus(agentId: string, status: string): void {
    const activity = agentStatusToActivity(status);
    this.agents = this.agents.map((a) =>
      a.id === agentId ? { ...a, status: status as AgentInfo["status"], activity } : a,
    );
    if (this.selectedId === agentId && this.selected) {
      this.selected = { ...this.selected, status: status as AgentInfo["status"], activity };
    }
  }

  onMessageReceived(from: string, to: string, messageId: string): void {
    communicationStore.recordEvent(from, to, messageId);
    if (this.selectedId === to) {
      this.refreshSelected();
    } else {
      this.agents = this.agents.map((a) =>
        a.id === to
          ? {
              ...a,
              unreadInbox: (a.unreadInbox ?? 0) + 1,
              lastAction: `Message de ${from}`,
            }
          : a,
      );
    }
    notificationsStore.push("info", `Message ${from} → ${to}`);
  }

  bumpActivity(action: string): void {
    if (this.agents.length === 0) return;
    const target =
      this.agents.find((a) => a.status === "awake") ??
      this.agents.find((a) => a.status === "background") ??
      this.agents[0];
    this.agents = this.agents.map((a) =>
      a.id === target.id
        ? { ...a, activity: Math.min(1, a.activity + 0.2), lastAction: action }
        : a,
    );
  }
}

export const agentsStore = new AgentsStore();