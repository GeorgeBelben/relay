import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { startGamepadListener, stickToDirection } from "./gamepad";

function fakeGamepad({
  pressed = [],
  axes = [0, 0],
}: { pressed?: number[]; axes?: number[] } = {}): Gamepad {
  const buttons = Array.from(
    { length: 17 },
    (_, i) => ({ pressed: pressed.includes(i) }) as GamepadButton,
  );
  return { buttons, axes } as unknown as Gamepad;
}

describe("stickToDirection", () => {
  it("returns null inside the deadzone", () => {
    expect(stickToDirection(0.1, 0.1)).toBeNull();
  });

  it("picks the dominant axis and its sign", () => {
    expect(stickToDirection(0.9, 0.1)).toBe("right");
    expect(stickToDirection(-0.9, 0.1)).toBe("left");
    expect(stickToDirection(0.1, 0.9)).toBe("down");
    expect(stickToDirection(0.1, -0.9)).toBe("up");
  });
});

describe("startGamepadListener", () => {
  let rafCallback: FrameRequestCallback | null = null;
  let cancel: ReturnType<typeof vi.fn>;
  let pads: (Gamepad | null)[] = [];

  beforeEach(() => {
    rafCallback = null;
    pads = [];
    cancel = vi.fn();
    vi.stubGlobal("requestAnimationFrame", (cb: FrameRequestCallback) => {
      rafCallback = cb;
      return 1;
    });
    vi.stubGlobal("cancelAnimationFrame", cancel);
    vi.stubGlobal("navigator", { getGamepads: () => pads });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  function tick(now: number) {
    const cb = rafCallback;
    rafCallback = null;
    cb?.(now);
  }

  it("fires a direction event once on press, not every frame while held", () => {
    const onEvent = vi.fn();
    startGamepadListener(onEvent, vi.fn());

    pads = [fakeGamepad({ pressed: [12] })]; // up
    tick(0);
    tick(50);
    tick(100);

    expect(onEvent).toHaveBeenCalledTimes(1);
    expect(onEvent).toHaveBeenCalledWith({ type: "direction", direction: "up" });
  });

  it("repeats a held direction after the initial delay, then at the repeat interval", () => {
    const onEvent = vi.fn();
    startGamepadListener(onEvent, vi.fn());

    pads = [fakeGamepad({ pressed: [12] })];
    tick(0); // initial press
    tick(500); // past the 400ms initial delay -> repeat
    tick(650); // past the 120ms repeat interval since the last repeat -> repeat again

    expect(onEvent).toHaveBeenCalledTimes(3);
  });

  it("fires a confirm/back action once on press, not on release", () => {
    const onEvent = vi.fn();
    startGamepadListener(onEvent, vi.fn());

    pads = [fakeGamepad({ pressed: [0] })]; // confirm
    tick(0);
    pads = [fakeGamepad()];
    tick(16);

    expect(onEvent).toHaveBeenCalledTimes(1);
    expect(onEvent).toHaveBeenCalledWith({ type: "action", action: "confirm" });
  });

  it("fires a home action for the Guide/PS/Xbox button", () => {
    const onEvent = vi.fn();
    startGamepadListener(onEvent, vi.fn());

    pads = [fakeGamepad({ pressed: [16] })]; // home
    tick(0);

    expect(onEvent).toHaveBeenCalledWith({ type: "action", action: "home" });
  });

  it("allows a released and re-pressed action to fire again", () => {
    const onEvent = vi.fn();
    startGamepadListener(onEvent, vi.fn());

    pads = [fakeGamepad({ pressed: [1] })]; // back
    tick(0);
    pads = [fakeGamepad()];
    tick(16);
    pads = [fakeGamepad({ pressed: [1] })];
    tick(32);

    expect(onEvent).toHaveBeenCalledTimes(2);
    expect(onEvent).toHaveBeenNthCalledWith(2, { type: "action", action: "back" });
  });

  it("calls onUsed only on frames where input actually happened", () => {
    const onUsed = vi.fn();
    startGamepadListener(vi.fn(), onUsed);

    pads = [fakeGamepad()];
    tick(0);
    expect(onUsed).not.toHaveBeenCalled();

    pads = [fakeGamepad({ pressed: [0] })];
    tick(16);
    expect(onUsed).toHaveBeenCalledOnce();
  });

  it("cancels the animation frame when torn down", () => {
    const stop = startGamepadListener(vi.fn(), vi.fn());
    stop();
    expect(cancel).toHaveBeenCalledWith(1);
  });
});
