import { useEffect } from "react";
import { getCurrentFocusKey } from "@noriginmedia/norigin-spatial-navigation-core";

// Norigin has no built-in concept of a third per-element action beyond onEnterPress -- this is a
// small keyed registry instead, mirroring backStack.ts's approach to the same kind of gap for
// "back". Whichever focusable is currently focused (by its own focusKey) gets first refusal on
// a "menu" press; anything that hasn't registered one is silently a no-op.
type MenuHandler = () => void;

const handlers = new Map<string, MenuHandler>();

export function handleMenu() {
  const key = getCurrentFocusKey();
  if (key) handlers.get(key)?.();
}

// Registers `onMenu` for the given focusKey for as long as the calling component is mounted --
// e.g. `useMenuHandler(game.id, () => setDrawerOpen(true))`.
export function useMenuHandler(focusKey: string, onMenu: MenuHandler) {
  useEffect(() => {
    handlers.set(focusKey, onMenu);
    return () => {
      if (handlers.get(focusKey) === onMenu) handlers.delete(focusKey);
    };
  }, [focusKey, onMenu]);
}
