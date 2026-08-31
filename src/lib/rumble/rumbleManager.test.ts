import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

function fakePad({ withActuator = true, rejects = false }: { withActuator?: boolean; rejects?: boolean } = {}) {
  const playEffect = vi.fn(() => (rejects ? Promise.reject(new Error("not supported")) : Promise.resolve("complete" as const)));
  return {
    vibrationActuator: withActuator ? { playEffect, reset: vi.fn() } : undefined,
    _playEffect: playEffect,
  } as unknown as Gamepad & { _playEffect: typeof playEffect };
}

let pads: (Gamepad | null)[] = [];

beforeEach(() => {
  vi.resetModules();
  pads = [];
  vi.stubGlobal("navigator", { getGamepads: () => pads });
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  vi.useRealTimers();
});

describe("playRumble", () => {
  it("plays dual-rumble on every connected pad with an actuator", async () => {
    const padA = fakePad();
    const padB = fakePad();
    pads = [padA, padB];

    const { playRumble } = await import("./rumbleManager");
    playRumble("confirm");

    expect(padA._playEffect).toHaveBeenCalledWith("dual-rumble", expect.objectContaining({ duration: expect.any(Number) }));
    expect(padB._playEffect).toHaveBeenCalledOnce();
  });

  it("skips a pad with no vibrationActuator, and a null slot, without throwing", async () => {
    const withActuator = fakePad();
    pads = [null, fakePad({ withActuator: false }), withActuator];

    const { playRumble } = await import("./rumbleManager");
    expect(() => playRumble("confirm")).not.toThrow();
    expect(withActuator._playEffect).toHaveBeenCalledOnce();
  });

  it("swallows a rejected playEffect (pad doesn't actually support dual-rumble)", async () => {
    const pad = fakePad({ rejects: true });
    pads = [pad];

    const { playRumble } = await import("./rumbleManager");
    expect(() => playRumble("nav")).not.toThrow();
    await Promise.resolve(); // let the rejection's .catch() run
  });

  it("debounces the nav effect within the minimum interval, but not confirm", async () => {
    const pad = fakePad();
    pads = [pad];

    const { playRumble } = await import("./rumbleManager");

    const now = vi.spyOn(performance, "now");
    now.mockReturnValue(1000);
    playRumble("nav");
    now.mockReturnValue(1020); // +20ms, inside the ~45ms debounce window
    playRumble("nav");
    expect(pad._playEffect).toHaveBeenCalledTimes(1);

    now.mockReturnValue(1046); // +46ms from the first -- past the window
    playRumble("nav");
    expect(pad._playEffect).toHaveBeenCalledTimes(2);

    playRumble("confirm");
    playRumble("confirm");
    expect(pad._playEffect).toHaveBeenCalledTimes(4); // confirm isn't debounced at all
  });

  it("plays error as two pulses, spaced apart", async () => {
    vi.useFakeTimers();
    const pad = fakePad();
    pads = [pad];

    const { playRumble } = await import("./rumbleManager");
    playRumble("error");
    expect(pad._playEffect).toHaveBeenCalledTimes(1);

    vi.advanceTimersByTime(89);
    expect(pad._playEffect).toHaveBeenCalledTimes(1);

    vi.advanceTimersByTime(1);
    expect(pad._playEffect).toHaveBeenCalledTimes(2);
  });

  it("does nothing at all once disabled", async () => {
    const pad = fakePad();
    pads = [pad];

    const { playRumble, setRumbleEnabled } = await import("./rumbleManager");
    setRumbleEnabled(false);
    playRumble("confirm");
    expect(pad._playEffect).not.toHaveBeenCalled();

    setRumbleEnabled(true);
    playRumble("confirm");
    expect(pad._playEffect).toHaveBeenCalledOnce();
  });
});
