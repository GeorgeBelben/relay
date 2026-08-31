import { renderHook } from "@testing-library/react";
import { act } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useConnectedControllers } from "./hooks";

function fakeGamepad(index: number): Gamepad {
  return { index } as Gamepad;
}

describe("useConnectedControllers", () => {
  let pads: (Gamepad | null)[] = [];

  beforeEach(() => {
    pads = [];
    vi.stubGlobal("navigator", { getGamepads: () => pads });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("reads whatever's already connected on mount, without waiting for a connect event", () => {
    pads = [fakeGamepad(0), null, fakeGamepad(2)];
    const { result } = renderHook(() => useConnectedControllers());
    expect(result.current).toEqual([0, 2]);
  });

  it("picks up a hot-plugged controller via gamepadconnected", () => {
    const { result } = renderHook(() => useConnectedControllers());
    expect(result.current).toEqual([]);

    pads = [fakeGamepad(0)];
    act(() => window.dispatchEvent(new Event("gamepadconnected")));

    expect(result.current).toEqual([0]);
  });

  it("drops a controller via gamepaddisconnected", () => {
    pads = [fakeGamepad(0), fakeGamepad(1)];
    const { result } = renderHook(() => useConnectedControllers());
    expect(result.current).toEqual([0, 1]);

    pads = [fakeGamepad(0)];
    act(() => window.dispatchEvent(new Event("gamepaddisconnected")));

    expect(result.current).toEqual([0]);
  });

  it("sorts indexes low to high regardless of connect order", () => {
    pads = [null, fakeGamepad(3), null, fakeGamepad(1)];
    const { result } = renderHook(() => useConnectedControllers());
    expect(result.current).toEqual([1, 3]);
  });
});
