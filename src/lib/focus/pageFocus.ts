import { useEffect } from "react";
import { useFocusable } from "@noriginmedia/norigin-spatial-navigation-react";

// The currently-mounted page's designated "home" focus key -- what Modal (modal.tsx) restores
// focus to when it closes. Norigin's own autoRestoreFocus only fires on a real unmount, never on
// a `focusable` prop toggling false (which is how Modal stays mounted across open/close -- see its
// own comment), so without an explicit fallback, closing a modal leaves focus pointing at a key
// that still technically "exists" in norigin's registry (doesFocusableExist doesn't distinguish
// "unfocusable" from "gone") but is unreachable by nav or Enter: nothing visibly focused, gamepad
// input dead until the user happens to trigger something else.
let currentPageFocusKey: string | null = null;

export function getPageFocusKey(): string | null {
  return currentPageFocusKey;
}

// Registers `focusKey` as the page-level fallback above, for the duration this component stays
// mounted. Split out from usePageFocus below for Home (routes/index.tsx), which drives its own
// initial focus via Carousel's autoFocus (data-dependent -- waits on a query) rather than the
// focus-immediately-on-mount behavior every other route wants.
export function useRegisterPageFocus(focusKey: string) {
  useEffect(() => {
    currentPageFocusKey = focusKey;
    return () => {
      if (currentPageFocusKey === focusKey) currentPageFocusKey = null;
    };
  }, [focusKey]);
}

// Drop-in replacement for the useFocusable({ focusKey }) + focusSelf-on-mount boilerplate most
// routes repeat: registers the fallback above *and* focuses this page on mount, same behavior as
// before. saveLastFocusedChild defaults to true on every useFocusable call (including this one),
// so restoring to this key -- whether on mount or later, from Modal -- naturally lands back on
// whatever was last focused within the page, not just a fixed first element.
export function usePageFocus(focusKey: string) {
  const { ref, focusKey: resolvedFocusKey, focusSelf } = useFocusable({ focusKey });
  useRegisterPageFocus(resolvedFocusKey);

  useEffect(() => {
    focusSelf();
  }, [focusSelf]);

  return { ref, focusKey: resolvedFocusKey };
}
