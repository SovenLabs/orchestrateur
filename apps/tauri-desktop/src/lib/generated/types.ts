/** Configuration client WebSocket. */
export type ConnectionConfig = {
  ws_url: string;
  token: string;
  heartbeat_ms: number;
  reconnect_base_ms: number;
  reconnect_max_ms: number;
};

export const DEFAULT_CONNECTION_CONFIG: ConnectionConfig = {
  ws_url: "ws://127.0.0.1:28790/ws",
  token: "dev",
  heartbeat_ms: 15_000,
  reconnect_base_ms: 500,
  reconnect_max_ms: 30_000,
};

export type HarnessCapabilities = {
  can_write_cortex: boolean;
  can_run_skills: boolean;
};

export type ClientInfo = {
  window_kind: string;
  window_id?: string;
  panels: string[];
  subscriptions: string[];
  harness?: HarnessCapabilities;
};

export const PROTOCOL_VERSION = "1.2.0";

export type DaemonClientMessage =
  | { type: "connect"; token: string; protocol_version?: string; client?: ClientInfo }
  | { type: "execute"; request_id: string; command: Record<string, unknown> }
  | { type: "ping"; nonce: number };

export type DaemonServerMessage =
  | {
      type: "connect_ok";
      version: string;
      protocol_version?: string;
      session_id: string;
      territory_session_id: string;
    }
  | { type: "result"; request_id: string; response: Record<string, unknown> }
  | {
      type: "broadcast";
      territory_session_id: string;
      event: string;
      source_session_id: string;
      payload: Record<string, unknown>;
    }
  | { type: "pong"; nonce: number }
  | { type: "error"; request_id?: string; message: string };

export type BackendEvent =
  | { event: "agent_activity"; level: number }
  | { event: "memory_assimilated"; memory_id: string; intensity: number }
  | { event: "draft_created"; draft_id: string; title: string; kind: string }
  | { event: "draft_published"; draft_id: string; memory_id: string }
  | { event: "draft_discarded"; draft_id: string }
  | { event: "thought_propagation"; path: number[] }
  | { event: "system_status"; status: string }
  | { event: "neuron_stimulated"; id: number; intensity: number }
  | { event: "daemon_broadcast"; name: string; payload: Record<string, unknown> }
  | {
      event: "connected";
      version: string;
      session_id: string;
      territory_session_id: string;
    }
  | { event: "disconnected"; reason: string };

export type FrontendCommand =
  | { command: "request_memory_snapshot" }
  | { command: "trigger_thought"; intensity: number }
  | { command: "heartbeat"; nonce: number };

export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "reconnecting";

export function mapBroadcastToBackendEvent(
  event: string,
  payload: Record<string, unknown>,
): BackendEvent {
  switch (event) {
    case "brain_pulse":
      return {
        event: "agent_activity",
        level:
          typeof payload.boost === "number"
            ? payload.boost
            : typeof payload.level === "number"
              ? payload.level
              : 0.5,
      };
    case "memory_assimilated":
      return {
        event: "memory_assimilated",
        memory_id: String(payload.memory_id ?? payload.id ?? ""),
        intensity: typeof payload.intensity === "number" ? payload.intensity : 0.7,
      };
    case "draft_created":
      return {
        event: "draft_created",
        draft_id: String(payload.draft_id ?? ""),
        title: String(payload.title ?? ""),
        kind: String(payload.kind ?? "context"),
      };
    case "draft_published":
      return {
        event: "draft_published",
        draft_id: String(payload.draft_id ?? ""),
        memory_id: String(payload.memory_id ?? ""),
      };
    case "draft_discarded":
      return {
        event: "draft_discarded",
        draft_id: String(payload.draft_id ?? ""),
      };
    case "memory_draft_ready":
      return {
        event: "draft_created",
        draft_id: String(payload.draft_id ?? ""),
        title: String(payload.title ?? ""),
        kind: String(payload.kind ?? "context"),
      };
    case "degraded_mode":
      return { event: "system_status", status: "degraded" };
    case "system_error":
      return { event: "system_status", status: "error" };
    default:
      return { event: "daemon_broadcast", name: event, payload };
  }
}