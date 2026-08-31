import { Dialog } from "@base-ui/react/dialog";
import { useEffect, useRef, type ReactNode } from "react";
import { FocusContext, getPageFocusKey, handleBack, pushBackHandler, setFocus, useFocusable } from "@/lib/focus";
import { playSound } from "@/lib/sound";
import { cn } from "@/lib/cn";

type ModalProps = {
  open: boolean;
  onClose: () => void;
  focusKey?: string;
  className?: string;
  children: ReactNode;
};

// Base UI supplies the portal/backdrop/viewport plumbing; our own virtual focus system (not
// native DOM focus) drives everything about what's focused and how the modal gets dismissed:
//
// - modal={false} + initialFocus/finalFocus={false} turn off Base UI's own native focus trap and
//   auto-focus/restore-focus, which would otherwise fight useFocusable's isFocusBoundary +
//   autoRestoreFocus (two different focus models trying to own the same thing).
// - Back (gamepad B, physical Escape, this component's own on-screen equivalent) always goes
//   through our own pushBackHandler stack below, never straight to onClose -- dismiss has to work
//   from a gamepad B press, which Base UI has no way to know about, so routing both through one
//   stack keeps "what does Back do" consistent with the rest of the app, including for any
//   caller that layers its own back-handler on top while this stays open (see drawer.tsx's
//   multi-view usage). Escape-key is cancelled here too, the same as outside-press, and re-driven
//   through handleBack() -- Base UI's own Escape listener calls stopPropagation() on the keydown
//   unconditionally, whether or not the resulting close gets cancelled, so our window-level
//   listener never sees the keypress at all; this onOpenChange callback is the *only* signal
//   physical Escape produces while a Base UI dialog is open. Calling onClose() straight from here
//   would mean Escape always closed immediately, bypassing anything layered on top of this
//   modal's own back-handler -- see drawer.tsx for the multi-view case that depends on this.
//
// Callers are expected to keep this mounted with `open` toggling (Base UI's close lifecycle
// wants that), so the focus registration and back-handler are both gated on `open` themselves --
// otherwise a permanently-mounted, currently-closed modal would still register as focusable and
// still intercept Back presses meant for whatever's actually on screen.
export function Modal({ open, onClose, focusKey = "MODAL", className, children }: ModalProps) {
  // saveLastFocusedChild: true is what makes focusSelf() below actually descend into the first
  // focusable child instead of just focusing this container as a leaf -- matters for any modal
  // with more than one row/button.
  const { ref, focusKey: resolvedFocusKey, focusSelf } = useFocusable({
    focusKey,
    focusable: open,
    isFocusBoundary: true,
    trackChildren: true,
    saveLastFocusedChild: true,
    autoRestoreFocus: true,
  });

  // The rAF defer matters: on the render where `open` flips true, this component's own effect
  // runs before its children's -- rows haven't registered with the focus service yet, so calling
  // focusSelf() synchronously here finds zero children and parks focus on the modal container
  // itself instead of its first row/button.
  useEffect(() => {
    if (!open) return;
    const raf = requestAnimationFrame(() => focusSelf());
    return () => cancelAnimationFrame(raf);
  }, [open, focusSelf]);

  useEffect(() => {
    if (!open) return;
    return pushBackHandler(onClose);
  }, [open, onClose]);

  // Both directions of a real open <-> closed transition, on the same ref so mount never fires
  // either one (see each branch's own comment for why that matters):
  const wasOpen = useRef(open);
  useEffect(() => {
    // Closed -> open: every Modal gets a sound when it appears, regardless of what opened it --
    // some open from a push with no confirm press behind them at all, which would otherwise be
    // silent. playSound("confirm") is debounced against itself (soundManager.ts) specifically so
    // this doesn't double up with the generic confirm blip the input-event stream also plays for
    // the (very common) case where a confirm press is what opened this modal in the first place --
    // whichever of the two fires first wins.
    if (!wasOpen.current && open) {
      playSound("confirm");
    }

    // Open -> closed: a sound, same "every panel gets one regardless of what caused the
    // transition" reasoning as the open branch above -- plus restoring focus to the current
    // page's own fallback (see lib/focus/pageFocus.ts). autoRestoreFocus above doesn't cover this
    // (see its own comment), so without the latter, focus is left pointing at a row inside the
    // now-closed modal that's unreachable by nav or Enter: no visible focus ring, gamepad input
    // effectively dead.
    if (wasOpen.current && !open) {
      playSound("back");
      const fallback = getPageFocusKey();
      if (fallback) setFocus(fallback);
    }

    wasOpen.current = open;
  }, [open]);

  return (
    <Dialog.Root
      open={open}
      modal={false}
      onOpenChange={(nextOpen, eventDetails) => {
        if (eventDetails.reason === "outside-press") {
          eventDetails.cancel();
          return;
        }
        if (eventDetails.reason === "escape-key") {
          eventDetails.cancel();
          handleBack();
          return;
        }
        if (!nextOpen) onClose();
      }}
    >
      <Dialog.Portal>
        <Dialog.Backdrop className="fixed inset-0 z-50 bg-black/60" />
        <Dialog.Viewport className="fixed inset-0 z-50 flex items-center justify-center">
          <Dialog.Popup
            ref={ref}
            initialFocus={false}
            finalFocus={false}
            className={cn("w-80 space-y-4 rounded-lg bg-card p-6 text-card-foreground shadow-lg", className)}
          >
            <FocusContext.Provider value={resolvedFocusKey}>{children}</FocusContext.Provider>
          </Dialog.Popup>
        </Dialog.Viewport>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
