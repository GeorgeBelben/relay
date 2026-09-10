import {
  doesFocusableExist,
  setFocus,
} from "@noriginmedia/norigin-spatial-navigation-core";

// A route change fully unmounts a screen's component tree, which tears down its focusable
// nodes with it -- by the time the screen remounts there's nothing left for the library to
// restore focus from (contrast with the `saveLastFocusedChild` option, which only helps while
// the parent stays mounted across a re-render). This is a plain module-level map, the same
// pattern as back-stack.ts, since the value needs to survive that unmount.
const lastFocusedByScreen = new Map<string, string>();

export function rememberFocus(screenKey: string, focusKey: string) {
  lastFocusedByScreen.set(screenKey, focusKey);
}

// Restores the screen's last-focused item if it still exists (e.g. the game is still in the
// list), otherwise falls back -- typically to focusSelf() -- for a first visit or a focus key
// that no longer resolves to anything.
export function restoreFocus(screenKey: string, fallback: () => void) {
  const focusKey = lastFocusedByScreen.get(screenKey);
  if (focusKey && doesFocusableExist(focusKey)) {
    void setFocus(focusKey);
  } else {
    fallback();
  }
}
