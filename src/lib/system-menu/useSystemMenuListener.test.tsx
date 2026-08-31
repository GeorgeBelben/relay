import { renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { navEvents } from "@/lib/input/nav";
import { useLaunchStore } from "@/lib/launch/store";
import { useSystemMenuStore } from "./store";
import { useSystemMenuListener } from "./useSystemMenuListener";

const game = { id: "g1", title: "Chrono Trigger", system_id: "snes" } as Parameters<
  ReturnType<typeof useLaunchStore.getState>["start"]
>[0];

describe("useSystemMenuListener", () => {
  beforeEach(() => {
    useLaunchStore.setState({ phase: "idle", game: null, message: null, quickMenuOpen: false });
    useSystemMenuStore.setState({ open: false });
  });

  it("toggles the system menu open then closed on repeated home presses while browsing", () => {
    renderHook(() => useSystemMenuListener());

    navEvents.emit({ type: "action", action: "home" });
    expect(useSystemMenuStore.getState().open).toBe(true);

    navEvents.emit({ type: "action", action: "home" });
    expect(useSystemMenuStore.getState().open).toBe(false);
  });

  it("ignores a home press while a game is playing (REL-137's quick menu owns that context instead)", () => {
    useLaunchStore.getState().start(game);
    useLaunchStore.getState().enterPlaying();
    renderHook(() => useSystemMenuListener());

    navEvents.emit({ type: "action", action: "home" });

    expect(useSystemMenuStore.getState().open).toBe(false);
  });

  it("ignores non-home actions", () => {
    renderHook(() => useSystemMenuListener());

    navEvents.emit({ type: "action", action: "menu" });
    navEvents.emit({ type: "action", action: "confirm" });
    navEvents.emit({ type: "direction", direction: "up" });

    expect(useSystemMenuStore.getState().open).toBe(false);
  });
});
