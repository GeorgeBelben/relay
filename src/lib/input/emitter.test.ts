import { describe, expect, it, vi } from "vitest";
import { createEmitter } from "./emitter";

describe("createEmitter", () => {
  it("notifies subscribed listeners", () => {
    const emitter = createEmitter<number>();
    const listener = vi.fn();
    emitter.subscribe(listener);

    emitter.emit(42);

    expect(listener).toHaveBeenCalledWith(42);
  });

  it("supports multiple listeners", () => {
    const emitter = createEmitter<number>();
    const a = vi.fn();
    const b = vi.fn();
    emitter.subscribe(a);
    emitter.subscribe(b);

    emitter.emit(1);

    expect(a).toHaveBeenCalledWith(1);
    expect(b).toHaveBeenCalledWith(1);
  });

  it("stops notifying after unsubscribe", () => {
    const emitter = createEmitter<number>();
    const listener = vi.fn();
    const unsubscribe = emitter.subscribe(listener);

    unsubscribe();
    emitter.emit(1);

    expect(listener).not.toHaveBeenCalled();
  });
});
