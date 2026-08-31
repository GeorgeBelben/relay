import { describe, expect, it } from "vitest";
import { useLaunchStore } from "./store";

const game = { id: "g1", title: "Chrono Trigger", system_id: "snes" } as Parameters<
  ReturnType<typeof useLaunchStore.getState>["start"]
>[0];

describe("useLaunchStore", () => {
  it("moves idle -> logo -> playing on a normal launch, keeping game populated throughout", () => {
    const store = useLaunchStore.getState();
    store.start(game);
    expect(useLaunchStore.getState()).toMatchObject({ phase: "logo", game, message: null });

    useLaunchStore.getState().enterPlaying();
    expect(useLaunchStore.getState()).toMatchObject({ phase: "playing", game, message: null });
  });

  it("dismiss() fully resets phase, game, and the quick menu together", () => {
    const store = useLaunchStore.getState();
    store.start(game);
    store.enterPlaying();
    store.openQuickMenu();
    expect(useLaunchStore.getState().quickMenuOpen).toBe(true);

    store.dismiss();

    expect(useLaunchStore.getState()).toMatchObject({
      phase: "idle",
      game: null,
      message: null,
      quickMenuOpen: false,
    });
  });

  it("a fresh start() while a quick menu happens to be open closes it", () => {
    const store = useLaunchStore.getState();
    store.start(game);
    store.enterPlaying();
    store.openQuickMenu();

    store.start(game);

    expect(useLaunchStore.getState().quickMenuOpen).toBe(false);
  });

  it("toggleQuickMenu flips open/closed each call", () => {
    const store = useLaunchStore.getState();
    expect(useLaunchStore.getState().quickMenuOpen).toBe(false);

    store.toggleQuickMenu();
    expect(useLaunchStore.getState().quickMenuOpen).toBe(true);

    store.toggleQuickMenu();
    expect(useLaunchStore.getState().quickMenuOpen).toBe(false);
  });

  it("setPhase(error) surfaces a message without touching game", () => {
    const store = useLaunchStore.getState();
    store.start(game);

    store.setPhase("error", "dolphin-emu: No such file or directory");

    expect(useLaunchStore.getState()).toMatchObject({
      phase: "error",
      game,
      message: "dolphin-emu: No such file or directory",
    });
  });
});
