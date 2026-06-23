import { describe, expect, it } from "vitest";
import {
  filterCommunicationEdges,
  filterCommunicationLog,
} from "./communication-filters";
import type { CommunicationEdge, CommunicationLogEntry } from "./communication-types";

const edges: CommunicationEdge[] = [
  { from: "analyst", to: "trader", count: 3, lastAt: 1000 },
  { from: "trader", to: "writer", count: 1, lastAt: 2000 },
];

const log: CommunicationLogEntry[] = [
  {
    id: "1",
    kind: "message",
    from: "analyst",
    to: "trader",
    body: "signal",
    at: 1000,
  },
  {
    id: "2",
    kind: "message",
    from: "trader",
    to: "writer",
    body: "relay",
    at: 2000,
  },
];

describe("communication filters", () => {
  it("filterCommunicationEdges retourne toutes les arêtes sans filtre", () => {
    expect(filterCommunicationEdges(edges, null)).toHaveLength(2);
  });

  it("filterCommunicationEdges ne garde que les arêtes liées à l'agent", () => {
    const filtered = filterCommunicationEdges(edges, "analyst");
    expect(filtered).toHaveLength(1);
    expect(filtered[0].from).toBe("analyst");
  });

  it("filterCommunicationLog filtre les entrées par agent", () => {
    expect(filterCommunicationLog(log, "trader")).toHaveLength(2);
    expect(filterCommunicationLog(log, "writer")).toHaveLength(1);
    expect(filterCommunicationLog(log, "writer")[0].to).toBe("writer");
  });
});