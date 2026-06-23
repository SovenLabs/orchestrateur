import { describe, expect, it } from "vitest";
import {
  agentStatusIndicator,
  agentStatusToActivity,
  isAgentAwake,
  mapAgentSummary,
  parseAgentList,
  parseAgentMessages,
} from "./bridge";

describe("agent bridge parsers", () => {
  it("parseAgentList mappe les champs AgentSummary", () => {
    const items = parseAgentList({
      response: "AgentList",
      payload: {
        items: [
          {
            id: "researcher",
            name: "Chercheur",
            role: "analyste",
            model: "grok-4.3",
            status: "awake",
            session_key: "agent:researcher",
            last_heartbeat: "2026-06-23T10:00:00Z",
          },
        ],
      },
    });
    expect(items).toHaveLength(1);
    expect(items[0].id).toBe("researcher");
    expect(items[0].activity).toBeGreaterThan(0.5);
    expect(items[0].sessionKey).toBe("agent:researcher");
  });

  it("parseAgentMessages compte les non lus", () => {
    const msgs = parseAgentMessages({
      response: "AgentMessages",
      payload: {
        items: [
          { id: "1", from: "a", to: "b", body: "hi", sent_at: "t", read: false },
          { id: "2", from: "a", to: "b", body: "ok", sent_at: "t", read: true },
        ],
      },
    });
    expect(msgs.filter((m) => !m.read)).toHaveLength(1);
  });

  it("isAgentAwake reconnaît awake et background", () => {
    expect(isAgentAwake("awake")).toBe(true);
    expect(isAgentAwake("background")).toBe(true);
    expect(isAgentAwake("sleeping")).toBe(false);
  });

  it("agentStatusIndicator classe les statuts", () => {
    expect(agentStatusIndicator("awake")).toBe("ok");
    expect(agentStatusIndicator("background")).toBe("warn");
    expect(agentStatusIndicator("sleeping")).toBe("idle");
  });

  it("mapAgentSummary défaut sleeping", () => {
    const a = mapAgentSummary({ id: "x", name: "X" });
    expect(a.status).toBe("sleeping");
    expect(agentStatusToActivity(a.status)).toBeLessThan(0.3);
  });
});