import { useEffect } from "react";
import {
  navigateByDirection,
  SpatialNavigation,
} from "@noriginmedia/norigin-spatial-navigation-core";
import { navEvents } from "@/lib/input/nav";
import type { NavEvent } from "@/lib/input/types";
import { useLaunchStore } from "@/lib/launch/store";
import { handleBack } from "./backStack";
import { handleMenu } from "./menuHandler";

// Keyboard arrows/enter are handled natively by the library's own keydown listener (see
// lib/input/keyboard.ts) -- by construction, the only "direction" and "confirm" events this
// emitter ever carries come from the gamepad poller, since the keyboard listener only emits
// for Escape ("back") and M ("menu"). This bridge drives the library programmatically for that
// gamepad path, exactly as the library's own docs recommend for non-native input sources.
function handleNavEvent(event: NavEvent) {
  if (event.type === "direction") {
    void navigateByDirection(event.direction);
    return;
  }

  if (event.action === "confirm") {
    SpatialNavigation.onEnterPress({ pressedKeys: {} });
    SpatialNavigation.onEnterRelease();
    return;
  }

  if (event.action === "menu") {
    // While a game is playing, "menu" belongs to the in-game quick menu (REL-23, see
    // useQuickMenuListener) exclusively -- the tile that was last focused before launch is still
    // registered with handleMenu() underneath (nothing unmounts it, the screen's just covered by
    // the emulator), and would otherwise open its GameCardDrawer invisibly behind the game.
    if (useLaunchStore.getState().phase === "playing") return;
    handleMenu();
    return;
  }

  handleBack();
}

export function useFocusBridge() {
  useEffect(() => navEvents.subscribe(handleNavEvent), []);
}
