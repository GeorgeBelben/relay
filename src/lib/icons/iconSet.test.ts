import { describe, expect, it } from "vitest";
import { getIconUrl } from "./iconSet";

describe("getIconUrl", () => {
  it("uses the selected controller's icon set for gamepad input", () => {
    const xbox = getIconUrl("gamepad", "xbox", "confirm");
    const playstation = getIconUrl("gamepad", "playstation", "confirm");
    const generic = getIconUrl("gamepad", "generic", "confirm");

    expect(xbox).not.toBe(playstation);
    expect(xbox).not.toBe(generic);
    expect(playstation).not.toBe(generic);
  });

  it("uses the keyboard icon set regardless of the selected controller type", () => {
    const withXbox = getIconUrl("keyboard", "xbox", "back");
    const withPlaystation = getIconUrl("keyboard", "playstation", "back");

    expect(withXbox).toBe(withPlaystation);
  });

  it("returns a distinct icon for every direction and action", () => {
    const keys = ["up", "down", "left", "right", "confirm", "back"] as const;
    const urls = keys.map((key) => getIconUrl("gamepad", "switch", key));

    expect(new Set(urls).size).toBe(keys.length);
  });
});
