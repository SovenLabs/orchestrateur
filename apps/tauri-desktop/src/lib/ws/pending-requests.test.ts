import { describe, expect, it, vi } from "vitest";
import { PendingRequestRegistry } from "./pending-requests";

describe("PendingRequestRegistry", () => {
  it("résout une requête enregistrée", () => {
    const reg = new PendingRequestRegistry();
    const resolve = vi.fn();
    reg.register("r1", resolve, () => {}, 5000);
    reg.resolve("r1", { response: "Health" });
    expect(resolve).toHaveBeenCalledWith({ response: "Health" });
    expect(reg.size).toBe(0);
  });

  it("rejectAll vide le registre", () => {
    const reg = new PendingRequestRegistry();
    const reject = vi.fn();
    reg.register("r1", () => {}, reject, 5000);
    reg.rejectAll("offline");
    expect(reject).toHaveBeenCalled();
    expect(reg.size).toBe(0);
  });
});