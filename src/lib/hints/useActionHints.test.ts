import { renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { useActionHints } from "./useActionHints";
import { useCurrentHints, useHintsStore } from "./store";

afterEach(() => {
  useHintsStore.setState({ entries: [] });
});

describe("useActionHints", () => {
  it("exposes its hints as the current hints while mounted", () => {
    const hints = [{ action: "confirm" as const, label: "Select" }];
    renderHook(() => useActionHints(hints));

    const { result } = renderHook(() => useCurrentHints());
    expect(result.current).toEqual(hints);
  });

  it("clears its hints once unmounted", () => {
    const hints = [{ action: "confirm" as const, label: "Select" }];
    const { unmount } = renderHook(() => useActionHints(hints));
    unmount();

    const { result } = renderHook(() => useCurrentHints());
    expect(result.current).toEqual([]);
  });

  it("shows a later-mounted (e.g. modal) hint set instead of the earlier one, and reverts on its unmount", () => {
    const screenHints = [{ action: "back" as const, label: "Exit" }];
    const modalHints = [{ action: "confirm" as const, label: "Install" }];

    renderHook(() => useActionHints(screenHints));
    const { unmount: unmountModal } = renderHook(() => useActionHints(modalHints));

    const { result } = renderHook(() => useCurrentHints());
    expect(result.current).toEqual(modalHints);

    unmountModal();

    const { result: afterClose } = renderHook(() => useCurrentHints());
    expect(afterClose.current).toEqual(screenHints);
  });

  it("an empty array registers as the topmost entry, hiding what's underneath", () => {
    const screenHints = [{ action: "back" as const, label: "Exit" }];

    renderHook(() => useActionHints(screenHints));
    renderHook(() => useActionHints([])); // e.g. a permanently-mounted dialog mid-download

    const { result } = renderHook(() => useCurrentHints());
    expect(result.current).toEqual([]);
  });

  it("null does not register at all, letting what's underneath show through", () => {
    const screenHints = [{ action: "back" as const, label: "Exit" }];

    renderHook(() => useActionHints(screenHints));
    renderHook(() => useActionHints(null)); // e.g. that same dialog while closed

    const { result } = renderHook(() => useCurrentHints());
    expect(result.current).toEqual(screenHints);
  });

  it("switching from null to a real array registers it, and back to null un-registers it", () => {
    const dialogHints = [{ action: "confirm" as const, label: "Install" }];
    const { rerender } = renderHook(({ open }) => useActionHints(open ? dialogHints : null), {
      initialProps: { open: false },
    });

    let { result } = renderHook(() => useCurrentHints());
    expect(result.current).toEqual([]);

    rerender({ open: true });
    ({ result } = renderHook(() => useCurrentHints()));
    expect(result.current).toEqual(dialogHints);

    rerender({ open: false });
    ({ result } = renderHook(() => useCurrentHints()));
    expect(result.current).toEqual([]);
  });
});
