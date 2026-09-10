import { useEffect, useId } from "react";
import { useHintsStore } from "./store";
import type { Hint } from "./types";

// A component's `hints` array is usually written inline (a fresh array/objects each render),
// so we can't use it directly as an effect dependency without pushing/popping on every render --
// stringifying it gives a stable, cheap-enough comparison for hint lists, which stay tiny.
//
// `null` means "not participating right now" (e.g. a permanently-mounted modal while it's
// closed) and skips registration entirely, so whatever's underneath in the stack shows through.
// This is deliberately different from `[]`, which registers as the topmost entry but with
// nothing to show -- correct for e.g. a dialog mid-download that wants to suppress every hint,
// including the screen behind it, without offering any of its own.
export function useActionHints(hints: Hint[] | null) {
  const id = useId();
  const push = useHintsStore((state) => state.push);
  const remove = useHintsStore((state) => state.remove);
  const hintsKey = hints ? JSON.stringify(hints) : null;

  useEffect(() => {
    if (hintsKey === null) return;
    push({ id, hints: JSON.parse(hintsKey) });
    return () => remove(id);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id, push, remove, hintsKey]);
}
