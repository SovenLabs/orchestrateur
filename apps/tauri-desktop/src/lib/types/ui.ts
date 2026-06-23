export type PanelId = "dashboard" | "memory" | "chat" | "agents" | "monitoring";

export type MemoryBacklink = {
  target: string;
  score: number;
  kind: "semantic" | "explicit_wikilink" | string;
};

export type MemoryKind =
  | "decision"
  | "dead_end"
  | "pattern"
  | "context"
  | "progress"
  | "business";

export type MemoryItem = {
  id: string;
  title: string;
  tags: string[];
  updated_at: string;
  backlink_count: number;
  backlinks: MemoryBacklink[];
  kind?: MemoryKind | string;
};

export type DraftItem = {
  id: string;
  title: string;
  kind: MemoryKind | string;
  tags: string[];
  created_at: string;
  source_session?: string | null;
};

export type WatcherStatus = {
  enabled: boolean;
  running: boolean;
  watch_dirs: string[];
  sessions_processed: number;
  drafts_created: number;
  drafts_pending: number;
  last_activity_at?: string | null;
  last_error?: string | null;
};

export type HealthStatus = {
  status: string;
  version: string;
  llm_available: boolean;
  embedding_available: boolean;
};

export type ChatMessage = {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  /** Version enrichie (prompt preprocessor) si disponible. */
  enrichedContent?: string | null;
  timestamp: number;
};

export type AgentInfo = {
  id: string;
  name: string;
  status: "idle" | "active" | "error";
  activity: number;
  lastAction?: string;
};

export type CommandAction = {
  id: string;
  label: string;
  shortcut?: string;
  panel?: PanelId;
  action?: () => void;
  disabled?: boolean;
};

export const PANEL_META: Record<
  PanelId,
  { label: string; description: string; icon: string }
> = {
  dashboard: {
    label: "Dashboard",
    description: "Vue d'ensemble Cortex & activité",
    icon: "◈",
  },
  memory: {
    label: "Memory",
    description: "Explorateur de mémoires",
    icon: "⬡",
  },
  chat: {
    label: "Thoughts",
    description: "Chat & flux de pensées",
    icon: "◎",
  },
  agents: {
    label: "Agents",
    description: "Registre & activité agents",
    icon: "◇",
  },
  monitoring: {
    label: "Monitoring",
    description: "Santé système & latence WS",
    icon: "▣",
  },
};