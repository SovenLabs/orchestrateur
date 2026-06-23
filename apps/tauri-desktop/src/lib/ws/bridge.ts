import type {
  AgentInfo,
  AgentMessage,
  DraftItem,
  HealthStatus,
  MemoryItem,
  WatcherStatus,
} from "$lib/types/ui";

export const PROTOCOL_VERSION = "1.2.0";

export function parseMemoryList(response: Record<string, unknown>): {
  items: MemoryItem[];
  total: number;
} {
  if (response.response !== "MemoryList" || !response.payload) {
    return { items: [], total: 0 };
  }
  const payload = response.payload as {
    items?: Array<Record<string, unknown>>;
    total?: number;
  };
  const items = (payload.items ?? []).map((m) => {
    const rawLinks = Array.isArray(m.backlinks) ? m.backlinks : [];
    const backlinks = rawLinks.map((bl: Record<string, unknown>) => ({
      target: String(bl.target ?? ""),
      score: Number(bl.score ?? 0),
      kind: String(bl.kind ?? "semantic"),
    }));
    return {
      id: String(m.id ?? ""),
      title: String(m.title ?? "Sans titre"),
      tags: Array.isArray(m.tags) ? m.tags.map(String) : [],
      updated_at: String(m.updated_at ?? ""),
      backlink_count: Number(m.backlink_count ?? backlinks.length),
      backlinks,
      kind: String(m.kind ?? "context"),
    };
  });
  return { items, total: Number(payload.total ?? items.length) };
}

export function parseHealth(response: Record<string, unknown>): HealthStatus | null {
  if (response.response !== "Health" || !response.payload) return null;
  const p = response.payload as Record<string, unknown>;
  return {
    status: String(p.status ?? "unknown"),
    version: String(p.version ?? ""),
    llm_available: Boolean(p.llm_available),
    embedding_available: Boolean(p.embedding_available),
  };
}

export function parseChatReply(response: Record<string, unknown>): string | null {
  if (response.response !== "ChatReply" || !response.payload) return null;
  const p = response.payload as Record<string, unknown>;
  return String(p.reply ?? "");
}

export function parseSkills(
  response: Record<string, unknown>,
): Array<{ name: string; description: string }> {
  if (response.response !== "SkillList" || !response.payload) return [];
  const p = response.payload as { skills?: Array<Record<string, unknown>> };
  return (p.skills ?? []).map((s) => ({
    name: String(s.name ?? ""),
    description: String(s.description ?? ""),
  }));
}

export type ServerHealthMetrics = {
  messages_received: number;
  messages_sent: number;
  broadcasts_sent: number;
  execute_requests: number;
  ping_requests: number;
  connections_opened: number;
  auth_failures: number;
  parse_errors: number;
};

export type ConnectedWindows = {
  main: number;
  extension: number;
  desktop: number;
  sphere: number;
  total: number;
};

export type ServerHealth = {
  status: string;
  version: string;
  protocol_version: string;
  connected_clients: number;
  connected_windows?: ConnectedWindows;
  metrics: ServerHealthMetrics;
};

export function parseDraftList(response: Record<string, unknown>): {
  items: DraftItem[];
  total: number;
} {
  if (response.response !== "DraftList" || !response.payload) {
    return { items: [], total: 0 };
  }
  const payload = response.payload as {
    items?: Array<Record<string, unknown>>;
    total?: number;
  };
  const items = (payload.items ?? []).map((d) => ({
    id: String(d.id ?? ""),
    title: String(d.title ?? ""),
    kind: String(d.kind ?? "context"),
    tags: Array.isArray(d.tags) ? d.tags.map(String) : [],
    created_at: String(d.created_at ?? ""),
    source_session: d.watcher_session
      ? String(d.watcher_session)
      : d.source_session
        ? String(d.source_session)
        : null,
  }));
  return { items, total: Number(payload.total ?? items.length) };
}

export function parseWatcherStatus(response: Record<string, unknown>): WatcherStatus | null {
  if (response.response !== "WatcherStatus" || !response.payload) return null;
  const p = response.payload as { status?: Record<string, unknown> };
  const s = p.status ?? {};
  return {
    enabled: Boolean(s.enabled),
    running: Boolean(s.running),
    watch_dirs: Array.isArray(s.watch_dirs) ? s.watch_dirs.map(String) : [],
    sessions_processed: Number(s.sessions_processed ?? 0),
    drafts_created: Number(s.drafts_created ?? 0),
    drafts_pending: Number(s.drafts_pending ?? 0),
    last_activity_at: s.last_activity_at ? String(s.last_activity_at) : null,
    last_error: s.last_error ? String(s.last_error) : null,
  };
}

export function agentStatusToActivity(status: string): number {
  switch (status) {
    case "awake":
    case "active":
      return 0.85;
    case "background":
      return 0.65;
    case "error":
      return 0.2;
    default:
      return 0.18;
  }
}

export function agentStatusIndicator(
  status: string,
): "ok" | "warn" | "error" | "idle" {
  if (status === "awake" || status === "active") return "ok";
  if (status === "background") return "warn";
  if (status === "error") return "error";
  return "idle";
}

export function mapAgentSummary(raw: Record<string, unknown>): AgentInfo {
  const status = String(raw.status ?? "sleeping");
  return {
    id: String(raw.id ?? ""),
    name: String(raw.name ?? raw.id ?? "Agent"),
    role: String(raw.role ?? ""),
    model: String(raw.model ?? ""),
    status: status as AgentInfo["status"],
    activity: agentStatusToActivity(status),
    lastHeartbeat: raw.last_heartbeat ? String(raw.last_heartbeat) : null,
    sessionKey: String(raw.session_key ?? ""),
    unreadInbox: 0,
  };
}

export function parseAgentList(response: Record<string, unknown>): AgentInfo[] {
  if (response.response !== "AgentList" || !response.payload) return [];
  const payload = response.payload as { items?: Array<Record<string, unknown>> };
  return (payload.items ?? []).map(mapAgentSummary);
}

export function parseAgentDetail(response: Record<string, unknown>): AgentInfo | null {
  if (response.response !== "AgentDetail" || !response.payload) return null;
  const payload = response.payload as { agent?: Record<string, unknown> };
  if (!payload.agent) return null;
  return mapAgentSummary(payload.agent);
}

export function parseAgentMessages(response: Record<string, unknown>): AgentMessage[] {
  if (response.response !== "AgentMessages" || !response.payload) return [];
  const payload = response.payload as { items?: Array<Record<string, unknown>> };
  return (payload.items ?? []).map((m) => ({
    id: String(m.id ?? ""),
    from: String(m.from ?? ""),
    to: String(m.to ?? ""),
    body: String(m.body ?? ""),
    sent_at: String(m.sent_at ?? ""),
    read: Boolean(m.read),
  }));
}

export function isAgentAwake(status: string): boolean {
  return status === "awake" || status === "background" || status === "active";
}

export async function fetchServerHealth(baseUrl: string): Promise<ServerHealth | null> {
  try {
    const res = await fetch(`${baseUrl}/health`);
    if (!res.ok) return null;
    return (await res.json()) as ServerHealth;
  } catch {
    return null;
  }
}