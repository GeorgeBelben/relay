import { describe, expect, it } from "vitest";
import { useSystemMenuStore } from "./store";

describe("useSystemMenuStore", () => {
  it("starts closed", () => {
    expect(useSystemMenuStore.getState().open).toBe(false);
  });

  it("openMenu/closeMenu set open directly", () => {
    useSystemMenuStore.getState().openMenu();
    expect(useSystemMenuStore.getState().open).toBe(true);

    useSystemMenuStore.getState().closeMenu();
    expect(useSystemMenuStore.getState().open).toBe(false);
  });

  it("toggleMenu flips open/closed each call", () => {
    useSystemMenuStore.setState({ open: false });

    useSystemMenuStore.getState().toggleMenu();
    expect(useSystemMenuStore.getState().open).toBe(true);

    useSystemMenuStore.getState().toggleMenu();
    expect(useSystemMenuStore.getState().open).toBe(false);
  });
});
