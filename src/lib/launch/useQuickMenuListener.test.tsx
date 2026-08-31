import { renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { navEvents } from "@/lib/input/nav";
import { useLaunchStore } from "./store";
import { useQuickMenuListener } from "./useQuickMenuListener";

const game = { id: "g1", title: "Chrono Trigger", system_id: "snes" } as Parameters<
  ReturnType<typeof useLaunchStore.getState>["start"]
>[0];

describe("useQuickMenuListener", () => {
  beforeEach(() => {
    useLaunchStore.setState({ phase: "idle", game: null, message: null, quickMenuOpen: false });
  });

  it("toggles the quick menu open then closed on repeated menu presses while playing", () => {
    useLaunchStore.getState().start(game);
    useLaunchStore.getState().enterPlaying();
    renderHook(() => useQuickMenuListener());

    navEvents.emit({ type: "action", action: "menu" });
    expect(useLaunchStore.getState().quickMenuOpen).toBe(true);

    navEvents.emit({ type: "action", action: "menu" });
    expect(useLaunchStore.getState().quickMenuOpen).toBe(false);
  });

  it("ignores a menu press when no game is playing", () => {
    renderHook(() => useQuickMenuListener());

    navEvents.emit({ type: "action", action: "menu" });

    expect(useLaunchStore.getState().quickMenuOpen).toBe(false);
  });

  it("ignores a menu press during the pre-launch logo phase", () => {
    useLaunchStore.getState().start(game);
    renderHook(() => useQuickMenuListener());

    navEvents.emit({ type: "action", action: "menu" });

    expect(useLaunchStore.getState().quickMenuOpen).toBe(false);
  });

  it("ignores non-menu actions while playing", () => {
    useLaunchStore.getState().start(game);
    useLaunchStore.getState().enterPlaying();
    renderHook(() => useQuickMenuListener());

    navEvents.emit({ type: "action", action: "confirm" });
    navEvents.emit({ type: "direction", direction: "up" });

    expect(useLaunchStore.getState().quickMenuOpen).toBe(false);
  });
});
