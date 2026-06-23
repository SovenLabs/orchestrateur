import type {
  BackendEvent,
  ConnectionConfig,
  DaemonClientMessage,
  DaemonServerMessage,
} from "$lib/generated/types";
import { DEFAULT_CONNECTION_CONFIG, mapBroadcastToBackendEvent } from "$lib/generated/types";
import { PROTOCOL_VERSION } from "$lib/ws/bridge";
import { OutboundMessageQueue } from "$lib/ws/message-queue";
import { PendingRequestRegistry } from "$lib/ws/pending-requests";
import { computeReconnectDelay } from "$lib/ws/reconnect";

type EventHandler = (event: BackendEvent) => void;
type StatusHandler = (
  status: "connecting" | "connected" | "reconnecting" | "disconnected",
  detail?: string,
) => void;
type ResultHandler = (requestId: string, response: Record<string, unknown>) => void;

let requestCounter = 0;

export function nextRequestId(): string {
  requestCounter += 1;
  return `desktop-${Date.now()}-${requestCounter}`;
}

export type DaemonClientOptions = {
  config?: ConnectionConfig;
  onEvent: EventHandler;
  onStatus: StatusHandler;
  onResult?: ResultHandler;
  /** Injecté en tests pour simuler WebSocket. */
  webSocketFactory?: (url: string) => WebSocket;
};

/**
 * Client WebSocket résilient — queue, pending requests, backoff, heartbeat.
 */
export class DaemonWebSocketClient {
  private socket: WebSocket | null = null;
  private config: ConnectionConfig;
  private reconnectAttempt = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  private intentionalClose = false;
  private outbound = new OutboundMessageQueue();
  private pending = new PendingRequestRegistry();
  private onEvent: EventHandler;
  private onStatus: StatusHandler;
  private onResult: ResultHandler;
  private wsFactory: (url: string) => WebSocket;

  constructor(options: DaemonClientOptions) {
    this.config = options.config ?? DEFAULT_CONNECTION_CONFIG;
    this.onEvent = options.onEvent;
    this.onStatus = options.onStatus;
    this.onResult = options.onResult ?? (() => {});
    this.wsFactory = options.webSocketFactory ?? ((url) => new WebSocket(url));
  }

  get queuedMessages(): number {
    return this.outbound.size;
  }

  get pendingRequests(): number {
    return this.pending.size;
  }

  connect(): void {
    this.intentionalClose = false;
    this.clearReconnectTimer();
    this.onStatus("connecting");
    this.socket = this.wsFactory(this.config.ws_url);

    this.socket.addEventListener("open", () => {
      this.reconnectAttempt = 0;
      this.sendConnect();
    });

    this.socket.addEventListener("message", (ev) => {
      if (typeof ev.data !== "string") return;
      this.handleMessage(ev.data);
    });

    this.socket.addEventListener("close", () => {
      this.stopHeartbeat();
      this.pending.rejectAll("connexion fermée");
      if (this.intentionalClose) {
        this.onStatus("disconnected", "fermeture volontaire");
        this.onEvent({ event: "disconnected", reason: "fermeture volontaire" });
        return;
      }
      this.scheduleReconnect();
    });

    this.socket.addEventListener("error", () => {
      this.onStatus("reconnecting", "erreur réseau");
    });
  }

  disconnect(): void {
    this.intentionalClose = true;
    this.clearReconnectTimer();
    this.stopHeartbeat();
    this.pending.rejectAll("déconnexion volontaire");
    this.socket?.close();
    this.socket = null;
    this.onStatus("disconnected");
  }

  ping(): void {
    this.send({ type: "ping", nonce: Date.now() });
  }

  execute(command: Record<string, unknown>, requestId = nextRequestId()): string {
    this.send({ type: "execute", request_id: requestId, command });
    return requestId;
  }

  executeAsync(
    command: Record<string, unknown>,
    timeoutMs = 30_000,
  ): Promise<Record<string, unknown>> {
    const requestId = nextRequestId();
    return new Promise((resolve, reject) => {
      this.pending.register(requestId, resolve, reject, timeoutMs);
      this.execute(command, requestId);
    });
  }

  healthCheck(): string {
    return this.execute({ command: "HealthCheck" });
  }

  listMemories(limit = 200): string {
    return this.execute({
      command: "List",
      payload: { filter: null, offset: 0, limit },
    });
  }

  sendChat(message: string): string {
    return this.execute({ command: "Chat", payload: { message } });
  }

  listSkills(): string {
    return this.execute({ command: "ListSkills" });
  }

  private sendConnect(): void {
    this.send({
      type: "connect",
      token: this.config.token,
      protocol_version: PROTOCOL_VERSION,
      client: {
        window_kind: "desktop",
        window_id: "tauri-main",
        panels: ["dashboard", "memory", "chat", "agents", "monitoring"],
        subscriptions: ["activity", "memories", "brain_pulse", "chat", "visual"],
      },
    });
  }

  private send(message: DaemonClientMessage): void {
    const payload = JSON.stringify(message);
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
      this.outbound.enqueue(payload);
      return;
    }
    this.socket.send(payload);
  }

  private flushQueue(): void {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) return;
    for (const msg of this.outbound.drain()) {
      this.socket.send(msg);
    }
  }

  private handleMessage(raw: string): void {
    let parsed: DaemonServerMessage;
    try {
      parsed = JSON.parse(raw) as DaemonServerMessage;
    } catch {
      this.onEvent({ event: "daemon_broadcast", name: "parse_error", payload: { raw } });
      return;
    }

    switch (parsed.type) {
      case "connect_ok":
        this.onStatus("connected", parsed.version);
        this.onEvent({
          event: "connected",
          version: parsed.version,
          session_id: parsed.session_id,
          territory_session_id: parsed.territory_session_id,
        });
        this.flushQueue();
        this.startHeartbeat();
        break;
      case "broadcast":
        this.onEvent(mapBroadcastToBackendEvent(parsed.event, parsed.payload ?? {}));
        break;
      case "pong":
        this.onEvent({
          event: "daemon_broadcast",
          name: "pong",
          payload: { nonce: parsed.nonce },
        });
        break;
      case "error":
        this.onEvent({
          event: "system_status",
          status: `error: ${parsed.message}`,
        });
        if (parsed.request_id) {
          this.pending.reject(parsed.request_id, new Error(parsed.message));
        }
        break;
      case "result":
        this.pending.resolve(parsed.request_id, parsed.response ?? {});
        this.onResult(parsed.request_id, parsed.response ?? {});
        this.onEvent({
          event: "daemon_broadcast",
          name: "result",
          payload: { request_id: parsed.request_id, response: parsed.response },
        });
        break;
    }
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();
    this.heartbeatTimer = setInterval(() => this.ping(), this.config.heartbeat_ms);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  private scheduleReconnect(): void {
    this.onStatus("reconnecting");
    this.onEvent({ event: "disconnected", reason: "reconnexion en cours" });
    const delay = computeReconnectDelay(
      this.reconnectAttempt,
      this.config.reconnect_base_ms,
      this.config.reconnect_max_ms,
    );
    this.reconnectAttempt += 1;
    this.clearReconnectTimer();
    this.reconnectTimer = setTimeout(() => this.connect(), delay);
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }
}