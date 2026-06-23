import type { BackendEvent } from "$lib/generated/types";
import type { ChatMessage } from "$lib/types/ui";

export type BlackholeState = "expanded" | "docked";

const MESSAGE_DEPTH_SCALE = 40;
const ASSIMILATION_BONUS = 0.05;
const MAX_ASSIMILATION_BONUSES = 10;
const MEMORY_DEPTH_SCALE = 80;
const LINK_DEPTH_SCALE = 400;

export type MemoryDensity = {
  total: number;
  linkTotal: number;
};

export function computeNuanceDepth(
  messages: ChatMessage[],
  eventLog: BackendEvent[],
  memory: MemoryDensity = { total: 0, linkTotal: 0 },
): number {
  const chatBase = Math.min(0.45, messages.length / MESSAGE_DEPTH_SCALE);
  const memoryBase = Math.min(0.5, memory.total / MEMORY_DEPTH_SCALE);
  const linkBase = Math.min(0.25, memory.linkTotal / LINK_DEPTH_SCALE);
  const assimilations = eventLog.filter((e) => e.event === "memory_assimilated").length;
  const assimBonus = Math.min(MAX_ASSIMILATION_BONUSES, assimilations) * ASSIMILATION_BONUS;
  const raw = chatBase + memoryBase + linkBase + assimBonus;
  const floor = memory.total > 0 ? 0.28 + Math.min(0.22, memory.total / 200) : 0;
  return Math.min(1, Math.max(floor, raw));
}

export function isThinking(chatPending: boolean, agentActivity: number): boolean {
  return chatPending || agentActivity > 0.6;
}

export type ScrollAnchor = { top: number; height: number; client: number };

/** Distance du bas (px) pour repasser en expanded depuis docked. */
export const SCROLL_EXPAND_PX = 56;
/** Distance du bas (px) pour passer en docked depuis expanded. */
export const SCROLL_DOCK_PX = 148;

export function scrollDistanceFromBottom(
  scrollTop: number,
  scrollHeight: number,
  clientHeight: number,
): number {
  return scrollHeight - scrollTop - clientHeight;
}

export function resolveBlackholeState(
  scrollTop: number,
  scrollHeight: number,
  clientHeight: number,
  threshold = SCROLL_DOCK_PX,
): BlackholeState {
  return scrollDistanceFromBottom(scrollTop, scrollHeight, clientHeight) <= threshold
    ? "expanded"
    : "docked";
}

export function resolveBlackholeStateWithHysteresis(
  scrollTop: number,
  scrollHeight: number,
  clientHeight: number,
  current: BlackholeState,
): BlackholeState {
  const dist = scrollDistanceFromBottom(scrollTop, scrollHeight, clientHeight);
  if (current === "expanded") {
    return dist > SCROLL_DOCK_PX ? "docked" : "expanded";
  }
  return dist <= SCROLL_EXPAND_PX ? "expanded" : "docked";
}

export function blackholeStateFromScroll(
  scrollTop: number,
  scrollHeight: number,
  clientHeight: number,
  current: BlackholeState = "expanded",
): { state: BlackholeState; anchor: ScrollAnchor } {
  return {
    state: resolveBlackholeStateWithHysteresis(scrollTop, scrollHeight, clientHeight, current),
    anchor: { top: scrollTop, height: scrollHeight, client: clientHeight },
  };
}

export function ringCountForDepth(depth: number): number {
  return Math.max(3, Math.min(6, Math.round(3 + depth * 3)));
}