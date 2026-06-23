import { describe, expect, it } from "vitest";
import { mapBroadcastToBackendEvent } from "./types";

describe("mapBroadcastToBackendEvent", () => {
  it("maps agent_status_changed", () => {
    const event = mapBroadcastToBackendEvent("agent_status_changed", {
      agent_id: "worker",
      status: "awake",
    });
    expect(event).toEqual({ event: "agent_status_changed", agent_id: "worker", status: "awake" });
  });

  it("maps brain_pulse boost to agent_activity level", () => {
    const event = mapBroadcastToBackendEvent("brain_pulse", { boost: 0.75, duration: 0.5 });
    expect(event.event).toBe("agent_activity");
    if (event.event === "agent_activity") {
      expect(event.level).toBe(0.75);
    }
  });
});