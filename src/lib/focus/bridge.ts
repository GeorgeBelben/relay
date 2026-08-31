import { useEffect } from "react";
import { navigateByDirection, SpatialNavigation } from "@noriginmedia/norigin-spatial-navigation-core";
import { navEvents } from "@/lib/input/nav";
import type { NavEvent } from "@/lib/input/types";
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
    handleMenu();
    return;
  }

  handleBack();
}

export function useFocusBridge() {
  useEffect(() => navEvents.subscribe(handleNavEvent), []);
}
