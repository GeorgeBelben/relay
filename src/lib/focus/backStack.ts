import { useEffect } from "react";

// Norigin Spatial Navigation has no concept of "back" -- it only ever moves focus by
// direction or fires enter-press on whatever's focused. Modals/dialogs register their own
// close handler here on mount; a "back" nav event (Escape / gamepad B) invokes whichever
// handler registered most recently, i.e. whatever's on top.
type BackHandler = () => void;

const stack: BackHandler[] = [];

export function pushBackHandler(handler: BackHandler): () => void {
  stack.push(handler);
  return () => {
    const index = stack.lastIndexOf(handler);
    if (index !== -1) stack.splice(index, 1);
  };
}

export function handleBack() {
  const handler = stack[stack.length - 1];
  handler?.();
}

// Registers `onBack` for as long as the calling component (typically a modal/dialog) is
// mounted -- e.g. `useBackHandler(closeModal)`.
export function useBackHandler(onBack: BackHandler) {
  useEffect(() => pushBackHandler(onBack), [onBack]);
}
