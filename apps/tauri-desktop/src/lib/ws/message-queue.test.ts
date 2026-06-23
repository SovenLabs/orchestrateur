import { describe, expect, it } from "vitest";
import { OutboundMessageQueue } from "./message-queue";

describe("OutboundMessageQueue", () => {
  it("drain vide la file", () => {
    const q = new OutboundMessageQueue();
    q.enqueue("a");
    q.enqueue("b");
    expect(q.drain()).toEqual(["a", "b"]);
    expect(q.size).toBe(0);
  });

  it("éjecte le plus ancien au-delà de maxSize", () => {
    const q = new OutboundMessageQueue(2);
    q.enqueue("1");
    q.enqueue("2");
    q.enqueue("3");
    expect(q.drain()).toEqual(["2", "3"]);
  });
});