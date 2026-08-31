import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { navEvents } from "@/lib/input/nav";
import { useFocusBridge } from "./bridge";

const { navigateByDirection, onEnterPress, onEnterRelease } = vi.hoisted(() => ({
  navigateByDirection: vi.fn(),
  onEnterPress: vi.fn(),
  onEnterRelease: vi.fn(),
}));

vi.mock("@noriginmedia/norigin-spatial-navigation-core", () => ({
  navigateByDirection,
  SpatialNavigation: { onEnterPress, onEnterRelease },
}));

const { handleBack } = vi.hoisted(() => ({ handleBack: vi.fn() }));
vi.mock("./backStack", () => ({ handleBack }));

describe("useFocusBridge", () => {
  it("forwards direction events to navigateByDirection", () => {
    renderHook(() => useFocusBridge());

    navEvents.emit({ type: "direction", direction: "up" });

    expect(navigateByDirection).toHaveBeenCalledWith("up");
  });

  it("forwards confirm actions to onEnterPress/onEnterRelease", () => {
    renderHook(() => useFocusBridge());

    navEvents.emit({ type: "action", action: "confirm" });

    expect(onEnterPress).toHaveBeenCalled();
    expect(onEnterRelease).toHaveBeenCalled();
  });

  it("forwards back actions to the back-handler stack", () => {
    renderHook(() => useFocusBridge());

    navEvents.emit({ type: "action", action: "back" });

    expect(handleBack).toHaveBeenCalled();
  });

  it("stops forwarding once unmounted", () => {
    const { unmount } = renderHook(() => useFocusBridge());
    unmount();

    navigateByDirection.mockClear();
    navEvents.emit({ type: "direction", direction: "down" });

    expect(navigateByDirection).not.toHaveBeenCalled();
  });
});
