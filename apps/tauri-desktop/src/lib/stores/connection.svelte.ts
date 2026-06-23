import { DaemonWebSocketClient } from "$lib/ws/client";
import type { BackendEvent, ConnectionConfig } from "$lib/generated/types";
import { DEFAULT_CONNECTION_CONFIG } from "$lib/generated/types";
import {
  fetchServerHealth,
  parseChatReply,
  parseDraftList,
  parseHealth,
  parseMemoryList,
  parseSkills,
  parseWatcherStatus,
  type ServerHealth,
  type ServerHealthMetrics,
} from "$lib/ws/bridge";
import type {
  ChatMessage,
  DraftItem,
  HealthStatus,
  MemoryItem,
  WatcherStatus,
} from "$lib/types/ui";
import { agentsStore } from "$lib/stores/agents.svelte";
import { notificationsStore } from "$lib/stores/notifications.svelte";

export type UiConnectionStatus = "disconnected" | "connecting" | "connected" | "reconnecting";

function resolveConfig(): ConnectionConfig {
  return {
    ...DEFAULT_CONNECTION_CONFIG,
    ws_url: import.meta.env.VITE_ORCHESTRATEUR_WS_URL ?? DEFAULT_CONNECTION_CONFIG.ws_url,
    token: import.meta.env.VITE_ORCHESTRATEUR_DAEMON_TOKEN ?? DEFAULT_CONNECTION_CONFIG.token,
  };
}

function httpBaseFromWs(wsUrl: string): string {
  return wsUrl.replace(/^ws/, "http").replace(/\/ws$/, "");
}

class ConnectionStore {
  status = $state<UiConnectionStatus>("disconnected");
  version = $state<string | null>(null);
  protocolVersion = $state<string | null>(null);
  sessionId = $state<string | null>(null);
  lastEvent = $state<BackendEvent | null>(null);
  eventLog = $state<BackendEvent[]>([]);
  latencyMs = $state<number | null>(null);
  agentActivity = $state(0.35);
  eventRate = $state(0);
  health = $state<HealthStatus | null>(null);
  serverHealth = $state<ServerHealth | null>(null);
  serverMetrics = $state<ServerHealthMetrics | null>(null);
  memories = $state<MemoryItem[]>([]);
  memoryTotal = $state(0);
  drafts = $state<DraftItem[]>([]);
  watcher = $state<WatcherStatus | null>(null);
  chatMessages = $state<ChatMessage[]>([]);
  chatPending = $state(false);
  skills = $state<Array<{ name: string; description: string }>>([]);
  queuedMessages = $state(0);
  pendingRequests = $state(0);

  private client: DaemonWebSocketClient | null = null;
  private lastPingAt: number | null = null;
  private eventTimestamps: number[] = [];
  private healthPollTimer: ReturnType<typeof setInterval> | null = null;
  private config = resolveConfig();

  constructor() {
    agentsStore.bindClient(() => this.client);
  }

  connect(): void {
    if (this.client) return;
    this.config = resolveConfig();
    this.client = new DaemonWebSocketClient({
      config: this.config,
      onEvent: (event) => this.handleEvent(event),
      onStatus: (status, detail) => {
        this.status = status;
        if (detail && status === "connected") this.version = detail;
        if (status === "connected") this.onConnected();
        if (status !== "connected") {
          this.stopHealthPolling();
          agentsStore.stopPolling();
        }
      },
      onResult: (requestId, response) => this.handleResult(requestId, response),
    });
    this.client.connect();
    this.syncClientCounters();
  }

  disconnect(): void {
    this.stopHealthPolling();
    agentsStore.stopPolling();
    this.client?.disconnect();
    this.client = null;
    this.status = "disconnected";
  }

  ping(): void {
    this.lastPingAt = Date.now();
    this.client?.ping();
    this.syncClientCounters();
  }

  healthCheck(): void {
    this.client?.healthCheck();
    this.syncClientCounters();
  }

  fetchMemories(): void {
    this.client?.listMemories();
    this.syncClientCounters();
  }

  fetchDrafts(): void {
    this.client?.execute({ command: "ListDrafts", payload: null });
    this.syncClientCounters();
  }

  refreshWatcher(): void {
    this.client?.execute({ command: "WatcherStatus", payload: null });
    this.syncClientCounters();
  }

  /** @deprecated Utiliser agentsStore.fetchAll() */
  fetchAgents(): void {
    agentsStore.fetchAll();
  }

  /** @deprecated Utiliser agentsStore.select() */
  selectAgent(id: string | null): void {
    agentsStore.select(id);
  }

  /** @deprecated Utiliser agentsStore.wake() */
  async wakeAgent(id: string): Promise<void> {
    await agentsStore.wake(id);
  }

  /** @deprecated Utiliser agentsStore.sleep() */
  async sleepAgent(id: string): Promise<void> {
    await agentsStore.sleep(id);
  }

  /** @deprecated Utiliser agentsStore.refreshSelected() */
  refreshSelectedAgent(): void {
    agentsStore.refreshSelected();
  }

  async publishDraft(id: string): Promise<void> {
    if (!this.client) return;
    await this.client.executeAsync({ command: "PublishDraft", payload: { id } });
    this.fetchDrafts();
    this.fetchMemories();
    this.syncClientCounters();
  }

  async discardDraft(id: string): Promise<void> {
    if (!this.client) return;
    await this.client.executeAsync({ command: "DiscardDraft", payload: { id } });
    this.fetchDrafts();
    this.syncClientCounters();
  }

  async sendChat(message: string): Promise<void> {
    if (!this.client || this.chatPending) return;
    const id = crypto.randomUUID();
    this.chatMessages = [
      ...this.chatMessages,
      { id, role: "user", content: message, timestamp: Date.now() },
    ];
    this.chatPending = true;
    this.bumpActivity(`Chat: ${message.slice(0, 48)}…`);
    try {
      const response = await this.client.executeAsync({
        command: "Chat",
        payload: { message },
      });
      this.applyChatResponse(response);
    } catch (err) {
      this.chatPending = false;
      notificationsStore.push("error", err instanceof Error ? err.message : "Erreur chat");
      this.chatMessages = [
        ...this.chatMessages,
        {
          id: crypto.randomUUID(),
          role: "system",
          content: err instanceof Error ? err.message : "Erreur chat",
          timestamp: Date.now(),
        },
      ];
    }
    this.syncClientCounters();
  }

  async pollServerHealth(): Promise<void> {
    const base = httpBaseFromWs(this.config.ws_url);
    const health = await fetchServerHealth(base);
    if (health) {
      this.serverHealth = health;
      this.serverMetrics = health.metrics;
      if (!this.protocolVersion) {
        this.protocolVersion = health.protocol_version;
      }
    }
  }

  private onConnected(): void {
    this.healthCheck();
    this.fetchMemories();
    this.fetchDrafts();
    this.refreshWatcher();
    this.client?.listSkills();
    agentsStore.fetchAll();
    agentsStore.startPolling();
    this.pollServerHealth();
    this.startHealthPolling();
    this.syncClientCounters();
    window.setTimeout(() => {
      if (this.status === "connected" && this.memoryTotal === 0) {
        this.fetchMemories();
      }
    }, 1500);
  }

  private startHealthPolling(): void {
    this.stopHealthPolling();
    this.healthPollTimer = setInterval(() => {
      void this.pollServerHealth();
      this.syncClientCounters();
    }, 10_000);
  }

  private stopHealthPolling(): void {
    if (this.healthPollTimer) {
      clearInterval(this.healthPollTimer);
      this.healthPollTimer = null;
    }
  }

  private syncClientCounters(): void {
    if (!this.client) {
      this.queuedMessages = 0;
      this.pendingRequests = 0;
      return;
    }
    this.queuedMessages = this.client.queuedMessages;
    this.pendingRequests = this.client.pendingRequests;
  }

  private handleResult(_requestId: string, response: Record<string, unknown>): void {
    const health = parseHealth(response);
    if (health) this.health = health;

    const memories = parseMemoryList(response);
    if (memories.items.length > 0 || response.response === "MemoryList") {
      this.memories = memories.items;
      this.memoryTotal = memories.total;
    }

    const skills = parseSkills(response);
    if (skills.length > 0) this.skills = skills;

    const drafts = parseDraftList(response);
    if (drafts.items.length > 0 || response.response === "DraftList") {
      this.drafts = drafts.items;
    }

    const watcher = parseWatcherStatus(response);
    if (watcher) this.watcher = watcher;

    agentsStore.handleBridgeResponse(response);

    if (response.response === "DraftPublished" || response.response === "DraftDiscarded") {
      this.fetchDrafts();
      if (response.response === "DraftPublished") this.fetchMemories();
    }

    if (response.response === "ChatReply") {
      this.applyChatResponse(response);
    }

    if (response.response === "Error") {
      this.chatPending = false;
      const p = response.payload as Record<string, unknown> | undefined;
      const msg = String(p?.message ?? "Erreur bridge");
      notificationsStore.push("error", msg);
      this.chatMessages = [
        ...this.chatMessages,
        {
          id: crypto.randomUUID(),
          role: "system",
          content: msg,
          timestamp: Date.now(),
        },
      ];
    }
    this.syncClientCounters();
  }

  private applyChatResponse(response: Record<string, unknown>): void {
    const reply = parseChatReply(response);
    if (reply === null) return;
    this.chatPending = false;
    this.chatMessages = [
      ...this.chatMessages,
      {
        id: crypto.randomUUID(),
        role: "assistant",
        content: reply,
        timestamp: Date.now(),
      },
    ];
    this.bumpActivity("Réponse chat reçue");
  }

  private bumpActivity(action: string): void {
    this.agentActivity = Math.min(1, this.agentActivity + 0.22);
    agentsStore.bumpActivity(action);
    setTimeout(() => {
      this.agentActivity = Math.max(0.12, this.agentActivity - 0.18);
    }, 2400);
  }

  private handleEvent(event: BackendEvent): void {
    this.lastEvent = event;
    this.eventLog = [event, ...this.eventLog].slice(0, 80);
    this.trackEventRate();

    if (event.event === "connected") {
      this.sessionId = event.session_id;
      this.version = event.version;
    }

    if (event.event === "agent_activity") {
      this.agentActivity = event.level;
    }

    if (event.event === "agent_status_changed") {
      agentsStore.applyStatus(event.agent_id, event.status);
    }

    if (event.event === "agent_message_received") {
      agentsStore.onMessageReceived(event.from, event.to, event.message_id);
    }

    if (event.event === "memory_assimilated") {
      this.bumpActivity(`Mémoire assimilée: ${event.memory_id}`);
      this.fetchMemories();
    }

    if (event.event === "draft_created") {
      this.bumpActivity(`Brouillon prêt: ${event.title}`);
      this.fetchDrafts();
      this.refreshWatcher();
    }

    if (event.event === "draft_published") {
      this.fetchDrafts();
      this.fetchMemories();
    }

    if (event.event === "draft_discarded") {
      this.fetchDrafts();
    }

    if (event.event === "thought_propagation") {
      this.agentActivity = Math.min(1, 0.4 + event.path.length * 0.05);
    }

    if (event.event === "neuron_stimulated") {
      this.agentActivity = Math.max(this.agentActivity, event.intensity);
    }

    if (event.event === "system_status" && event.status.includes("error")) {
      notificationsStore.push("error", event.status);
    }

    if (event.event === "daemon_broadcast") {
      if (event.name === "agent_message") {
        const p = event.payload;
        agentsStore.onMessageReceived(
          String(p.from ?? ""),
          String(p.to ?? ""),
          String(p.message_id ?? ""),
        );
      }
      if (event.name === "pong" && this.lastPingAt) {
        this.latencyMs = Date.now() - this.lastPingAt;
        this.lastPingAt = null;
      }
    }

    this.syncClientCounters();
  }

  private trackEventRate(): void {
    const now = Date.now();
    this.eventTimestamps = [...this.eventTimestamps, now].filter((t) => now - t < 60_000);
    this.eventRate = this.eventTimestamps.length;
  }
}

export const connectionStore = new ConnectionStore();