import { describe, expect, it } from "vitest";
import type { BackendEvent } from "$lib/generated/types";
import type { ChatMessage } from "$lib/types/ui";
import {
  computeNuanceDepth,
  isThinking,
  resolveBlackholeState,
  resolveBlackholeStateWithHysteresis,
  ringCountForDepth,
  SCROLL_DOCK_PX,
} from "./nuance";

function msg(id: string): ChatMessage {
  return { id, role: "user", content: "x", timestamp: 0 };
}

function assimilated(id: string): BackendEvent {
  return { event: "memory_assimilated", memory_id: id, intensity: 0.5 };
}

describe("computeNuanceDepth", () => {
  it("scale avec le nombre de messages", () => {
    expect(computeNuanceDepth([], [])).toBe(0);
    expect(computeNuanceDepth(Array.from({ length: 20 }, (_, i) => msg(String(i))), [])).toBe(0.45);
    expect(computeNuanceDepth(Array.from({ length: 40 }, (_, i) => msg(String(i))), [])).toBe(0.45);
  });

  it("ajoute un bonus par assimilation mémoire", () => {
    const events = Array.from({ length: 3 }, (_, i) => assimilated(String(i)));
    expect(computeNuanceDepth([], events)).toBeCloseTo(0.15);
  });

  it("plafonne à 1", () => {
    const messages = Array.from({ length: 40 }, (_, i) => msg(String(i)));
    const events = Array.from({ length: 20 }, (_, i) => assimilated(String(i)));
    expect(computeNuanceDepth(messages, events)).toBeCloseTo(0.95);
  });

  it("donne une profondeur visible avec des mémoires sans chat", () => {
    const depth = computeNuanceDepth([], [], { total: 100, linkTotal: 400 });
    expect(depth).toBeGreaterThanOrEqual(0.5);
    expect(depth).toBeLessThanOrEqual(1);
  });
});

describe("isThinking", () => {
  it("détecte chat pending ou activité agent élevée", () => {
    expect(isThinking(false, 0)).toBe(false);
    expect(isThinking(true, 0)).toBe(true);
    expect(isThinking(false, 0.7)).toBe(true);
    expect(isThinking(false, 0.5)).toBe(false);
  });
});

describe("resolveBlackholeState", () => {
  it("expanded près du bas, docked sinon", () => {
    const expandedTop = 1000 - 400 - (SCROLL_DOCK_PX - 10);
    expect(resolveBlackholeState(expandedTop, 1000, 400)).toBe("expanded");
    expect(resolveBlackholeState(100, 1000, 400)).toBe("docked");
  });
});

describe("resolveBlackholeStateWithHysteresis", () => {
  it("évite le flip-flop autour du seuil", () => {
    const borderTop = 1000 - 400 - SCROLL_DOCK_PX;
    expect(resolveBlackholeStateWithHysteresis(borderTop, 1000, 400, "expanded")).toBe("expanded");
    expect(resolveBlackholeStateWithHysteresis(borderTop - 20, 1000, 400, "expanded")).toBe("docked");
  });
});

describe("ringCountForDepth", () => {
  it("varie entre 3 et 6 anneaux", () => {
    expect(ringCountForDepth(0)).toBe(3);
    expect(ringCountForDepth(1)).toBe(6);
    expect(ringCountForDepth(0.5)).toBe(5);
  });
});