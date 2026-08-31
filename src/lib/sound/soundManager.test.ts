import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

class FakeGainNode {
  gain = { value: 1 };
  connect = vi.fn();
}

class FakeBufferSourceNode {
  buffer: unknown = null;
  connect = vi.fn();
  start = vi.fn();
}

class FakeAudioContext {
  state: "running" | "suspended" = "running";
  destination = {};
  resume = vi.fn(async () => {
    this.state = "running";
  });
  createGain = vi.fn(() => new FakeGainNode());
  createBufferSource = vi.fn(() => new FakeBufferSourceNode());
  decodeAudioData = vi.fn(async () => ({}) as AudioBuffer);
}

let fakeContext: FakeAudioContext;

beforeEach(() => {
  vi.resetModules();
  fakeContext = new FakeAudioContext();
  // A plain function, not an arrow function -- `new AudioContext()` in the module under test
  // requires this stub to be usable as a constructor, which an arrow function can never be.
  vi.stubGlobal(
    "AudioContext",
    vi.fn(function () {
      return fakeContext;
    }),
  );
  vi.stubGlobal("fetch", vi.fn(async () => ({ arrayBuffer: async () => new ArrayBuffer(0) }) as Response));
  vi.spyOn(console, "warn").mockImplementation(() => {});
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("preloadSounds", () => {
  it("decodes all five sounds so every one is playable afterward", async () => {
    const { preloadSounds, playSound } = await import("./soundManager");
    await preloadSounds();

    for (const name of ["nav", "confirm", "back", "launch", "error"] as const) playSound(name);

    expect(fakeContext.createBufferSource).toHaveBeenCalledTimes(5);
    expect(console.warn).not.toHaveBeenCalled();
  });

  it("logs a warning and leaves the rest playable when one sound fails to decode", async () => {
    fakeContext.decodeAudioData.mockRejectedValueOnce(new Error("corrupt audio data"));
    const { preloadSounds, playSound } = await import("./soundManager");

    await expect(preloadSounds()).resolves.toBeUndefined(); // doesn't throw/reject overall
    expect(console.warn).toHaveBeenCalledOnce();

    for (const name of ["nav", "confirm", "back", "launch", "error"] as const) playSound(name);
    // Exactly one of the five never loaded a buffer, so playSound is a silent no-op for it --
    // the other four still played.
    expect(fakeContext.createBufferSource).toHaveBeenCalledTimes(4);
  });
});

describe("playSound", () => {
  it("does nothing for a sound that was never loaded", async () => {
    const { playSound } = await import("./soundManager");
    playSound("nav"); // preloadSounds() never called -- buffers map is empty
    expect(fakeContext.createBufferSource).not.toHaveBeenCalled();
  });

  it("debounces the nav sound within the minimum interval, but not launch/error", async () => {
    const { preloadSounds, playSound } = await import("./soundManager");
    await preloadSounds();

    const now = vi.spyOn(performance, "now");
    now.mockReturnValue(1000);
    playSound("nav");
    now.mockReturnValue(1020); // +20ms, inside the ~45ms debounce window
    playSound("nav");
    expect(fakeContext.createBufferSource).toHaveBeenCalledTimes(1);

    now.mockReturnValue(1046); // +46ms from the first play -- past the window
    playSound("nav");
    expect(fakeContext.createBufferSource).toHaveBeenCalledTimes(2);

    // launch/error aren't debounced at all -- back-to-back calls of each all play. (confirm/back
    // *are* debounced against themselves -- see the dedicated tests below.)
    playSound("launch");
    playSound("error");
    expect(fakeContext.createBufferSource).toHaveBeenCalledTimes(4);
  });

  it.each(["confirm", "back"] as const)(
    "debounces %s against itself, so a Modal/Drawer open or close doesn't double up with its own trigger press",
    async (name) => {
      // Modal/Drawer (modal.tsx, drawer.tsx) play "confirm" on their own open transition and
      // "back" on their own close transition, on top of whatever the input-event stream already
      // played for the physical press that caused it (see useSoundEvents.ts) -- this debounce is
      // what collapses that pair into a single play, regardless of which of the two call sites
      // happens to fire first.
      const { preloadSounds, playSound } = await import("./soundManager");
      await preloadSounds();

      const now = vi.spyOn(performance, "now");
      now.mockReturnValue(2000);
      playSound(name); // the generic, input-stream-driven play
      now.mockReturnValue(2010); // +10ms -- well within the window, e.g. the panel's own next-frame effect
      playSound(name); // the panel's own open/close-transition play
      expect(fakeContext.createBufferSource).toHaveBeenCalledTimes(1);

      now.mockReturnValue(2090); // +90ms from the first -- a genuinely separate later press
      playSound(name);
      expect(fakeContext.createBufferSource).toHaveBeenCalledTimes(2);
    },
  );

  it("suppresses a confirm sound that immediately follows a launch, from the same press", async () => {
    // Launching a game *is* the outcome of a confirm press on a tile -- without this, the generic
    // confirm blip (useSoundEvents.ts) plays right on top of the launch stinger for that same
    // physical press (startGameLaunch.ts).
    const { preloadSounds, playSound } = await import("./soundManager");
    await preloadSounds();

    const now = vi.spyOn(performance, "now");
    now.mockReturnValue(3000);
    playSound("launch");
    now.mockReturnValue(3005); // +5ms -- same input-event tick
    playSound("confirm");
    expect(fakeContext.createBufferSource).toHaveBeenCalledTimes(1); // only the launch stinger

    now.mockReturnValue(3200); // +200ms later -- an unrelated, later confirm press plays normally
    playSound("confirm");
    expect(fakeContext.createBufferSource).toHaveBeenCalledTimes(2);
  });

  it("resumes a suspended context on the next play (autoplay-policy compliance)", async () => {
    const { preloadSounds, playSound } = await import("./soundManager");
    await preloadSounds();

    fakeContext.state = "suspended";
    playSound("confirm");

    expect(fakeContext.resume).toHaveBeenCalledOnce();
  });
});

describe("setVolume", () => {
  it("clamps to [0, 1] and applies to the shared gain node", async () => {
    const { preloadSounds, setVolume } = await import("./soundManager");
    await preloadSounds();
    const gainNode = fakeContext.createGain.mock.results[0]?.value as FakeGainNode;

    setVolume(0.5);
    expect(gainNode.gain.value).toBe(0.5);

    setVolume(5);
    expect(gainNode.gain.value).toBe(1);

    setVolume(-1);
    expect(gainNode.gain.value).toBe(0);
  });
});
