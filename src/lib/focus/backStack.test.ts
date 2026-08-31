import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { handleBack, pushBackHandler, useBackHandler } from "./backStack";

describe("pushBackHandler / handleBack", () => {
  it("invokes the most recently pushed handler", () => {
    const first = vi.fn();
    const second = vi.fn();
    const popFirst = pushBackHandler(first);
    const popSecond = pushBackHandler(second);

    handleBack();

    expect(second).toHaveBeenCalledOnce();
    expect(first).not.toHaveBeenCalled();

    popSecond();
    popFirst();
  });

  it("falls back to the next handler once the top one is popped", () => {
    const first = vi.fn();
    const second = vi.fn();
    const popFirst = pushBackHandler(first);
    const popSecond = pushBackHandler(second);

    popSecond();
    handleBack();

    expect(first).toHaveBeenCalledOnce();
    popFirst();
  });

  it("does nothing when the stack is empty", () => {
    expect(() => handleBack()).not.toThrow();
  });
});

describe("useBackHandler", () => {
  it("registers on mount and deregisters on unmount", () => {
    const onBack = vi.fn();
    const { unmount } = renderHook(() => useBackHandler(onBack));

    handleBack();
    expect(onBack).toHaveBeenCalledOnce();

    unmount();
    handleBack();
    expect(onBack).toHaveBeenCalledOnce();
  });
});
