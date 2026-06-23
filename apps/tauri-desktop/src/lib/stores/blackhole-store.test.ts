import { describe, expect, it } from "vitest";
import {
  blackholeStateFromScroll,
  resolveBlackholeStateWithHysteresis,
  SCROLL_DOCK_PX,
  SCROLL_EXPAND_PX,
} from "$lib/cosmic/nuance";

describe("blackhole scroll hysteresis", () => {
  it("reste expanded dans la zone tampon", () => {
    const dist = SCROLL_DOCK_PX - 20;
    const scrollTop = 1000 - 400 - dist;
    expect(resolveBlackholeStateWithHysteresis(scrollTop, 1000, 400, "expanded")).toBe("expanded");
  });

  it("passe docked au-delà du seuil haut", () => {
    const dist = SCROLL_DOCK_PX + 10;
    const scrollTop = 1000 - 400 - dist;
    expect(resolveBlackholeStateWithHysteresis(scrollTop, 1000, 400, "expanded")).toBe("docked");
  });

  it("reste docked tant qu'on n'atteint pas le seuil bas", () => {
    const dist = SCROLL_EXPAND_PX + 10;
    const scrollTop = 1000 - 400 - dist;
    expect(resolveBlackholeStateWithHysteresis(scrollTop, 1000, 400, "docked")).toBe("docked");
  });

  it("repasse expanded près du bas", () => {
    const dist = SCROLL_EXPAND_PX - 5;
    const scrollTop = 1000 - 400 - dist;
    const next = blackholeStateFromScroll(scrollTop, 1000, 400, "docked");
    expect(next.state).toBe("expanded");
  });
});