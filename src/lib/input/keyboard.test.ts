import { describe, expect, it, vi } from "vitest";
import { mapKeyToNavEvent, startKeyboardListener } from "./keyboard";

describe("mapKeyToNavEvent", () => {
  it("maps arrow keys to direction events", () => {
    expect(mapKeyToNavEvent("ArrowUp")).toEqual({ type: "direction", direction: "up" });
    expect(mapKeyToNavEvent("ArrowDown")).toEqual({ type: "direction", direction: "down" });
    expect(mapKeyToNavEvent("ArrowLeft")).toEqual({ type: "direction", direction: "left" });
    expect(mapKeyToNavEvent("ArrowRight")).toEqual({ type: "direction", direction: "right" });
  });

  it("maps Enter to confirm and Escape to back", () => {
    expect(mapKeyToNavEvent("Enter")).toEqual({ type: "action", action: "confirm" });
    expect(mapKeyToNavEvent("Escape")).toEqual({ type: "action", action: "back" });
  });

  it("returns null for unmapped keys", () => {
    expect(mapKeyToNavEvent("a")).toBeNull();
    expect(mapKeyToNavEvent("Tab")).toBeNull();
  });
});

describe("startKeyboardListener", () => {
  it("fires onEvent and onUsed for Escape (back)", () => {
    const onEvent = vi.fn();
    const onUsed = vi.fn();
    const stop = startKeyboardListener(onEvent, onUsed);

    const event = new KeyboardEvent("keydown", { key: "Escape", cancelable: true });
    window.dispatchEvent(event);

    expect(onEvent).toHaveBeenCalledWith({ type: "action", action: "back" });
    expect(onUsed).toHaveBeenCalledOnce();
    expect(event.defaultPrevented).toBe(true);
    stop();
  });

  it("tracks usage for arrows/Enter but leaves them to the focus engine's native listener", () => {
    const onEvent = vi.fn();
    const onUsed = vi.fn();
    const stop = startKeyboardListener(onEvent, onUsed);

    const event = new KeyboardEvent("keydown", { key: "ArrowUp", cancelable: true });
    window.dispatchEvent(event);

    expect(onUsed).toHaveBeenCalledOnce();
    expect(onEvent).not.toHaveBeenCalled();
    expect(event.defaultPrevented).toBe(false);
    stop();
  });

  it("fires onAnyRecognizedKey for arrows/Enter too, without those reaching onEvent", () => {
    const onAnyRecognizedKey = vi.fn();
    const stop = startKeyboardListener(vi.fn(), vi.fn(), onAnyRecognizedKey);

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowUp", cancelable: true }));
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", cancelable: true }));

    expect(onAnyRecognizedKey).toHaveBeenNthCalledWith(1, { type: "direction", direction: "up" });
    expect(onAnyRecognizedKey).toHaveBeenNthCalledWith(2, { type: "action", action: "confirm" });
    stop();
  });

  it("also fires onAnyRecognizedKey for Escape/menu, alongside onEvent", () => {
    const onEvent = vi.fn();
    const onAnyRecognizedKey = vi.fn();
    const stop = startKeyboardListener(onEvent, vi.fn(), onAnyRecognizedKey);

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", cancelable: true }));

    expect(onEvent).toHaveBeenCalledWith({ type: "action", action: "back" });
    expect(onAnyRecognizedKey).toHaveBeenCalledWith({ type: "action", action: "back" });
    stop();
  });

  it("does not fire for unmapped keys", () => {
    const onEvent = vi.fn();
    const onUsed = vi.fn();
    const stop = startKeyboardListener(onEvent, onUsed);

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "a" }));

    expect(onEvent).not.toHaveBeenCalled();
    expect(onUsed).not.toHaveBeenCalled();
    stop();
  });

  it("prevents Tab's default behavior without firing a nav event", () => {
    const onEvent = vi.fn();
    const onUsed = vi.fn();
    const stop = startKeyboardListener(onEvent, onUsed);

    const event = new KeyboardEvent("keydown", { key: "Tab", cancelable: true });
    window.dispatchEvent(event);

    expect(event.defaultPrevented).toBe(true);
    expect(onEvent).not.toHaveBeenCalled();
    stop();
  });

  it("stops listening once torn down", () => {
    const onEvent = vi.fn();
    const stop = startKeyboardListener(onEvent, vi.fn());
    stop();

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));

    expect(onEvent).not.toHaveBeenCalled();
  });
});
