import { Dialog } from "@base-ui/react/dialog";
import { useEffect, useRef, type ReactNode } from "react";
import { FocusContext, handleBack, pushBackHandler, useFocusable } from "@/lib/focus";
import { playSound } from "@/lib/sound";
import { cn } from "@/lib/cn";

type DrawerProps = {
  open: boolean;
  onClose: () => void;
  focusKey?: string;
  className?: string;
  children: ReactNode;
};

// Right-hand, full-height panel -- see modal.tsx for the full rationale on the Base UI setup
// (same story here: our own virtual focus system owns everything about what's focused and how
// this gets dismissed, Base UI just supplies the portal/backdrop/viewport plumbing). Used for the
// game card context menu (see game-card-drawer.tsx).
export function Drawer({ open, onClose, focusKey = "DRAWER", className, children }: DrawerProps) {
  // saveLastFocusedChild is what makes focusSelf() descend into the first row instead of
  // focusing this container itself as a leaf -- Carousel/Grid rely on this same flag for their
  // own focusSelf() calls.
  const { ref, focusKey: resolvedFocusKey, focusSelf } = useFocusable({
    focusKey,
    focusable: open,
    isFocusBoundary: true,
    trackChildren: true,
    saveLastFocusedChild: true,
    autoRestoreFocus: true,
  });

  // The rAF matters: on the render where `open` flips true, this component's own effect runs
  // before its children's -- rows haven't registered with the focus service yet, so calling
  // focusSelf() synchronously here finds zero children and parks focus on the drawer container
  // itself. Deferring one frame lets the children's mount effects land first.
  useEffect(() => {
    if (!open) return;
    const raf = requestAnimationFrame(() => focusSelf());
    return () => cancelAnimationFrame(raf);
  }, [open, focusSelf]);

  useEffect(() => {
    if (!open) return;
    return pushBackHandler(onClose);
  }, [open, onClose]);

  // A sound on each real transition, mirroring modal.tsx's own (see its longer comment on why
  // this needs to be explicit rather than relying solely on the input-event stream, and why the
  // two sounds are debounced against themselves in soundManager.ts). Deliberately doesn't also
  // take on modal.tsx's focus-restore-on-close job -- game-card-drawer.tsx's closeDrawer already
  // reclaims focus itself, more precisely (back to the exact tile that opened this), than the
  // generic page-fallback modal.tsx falls back to.
  const wasOpen = useRef(open);
  useEffect(() => {
    if (!wasOpen.current && open) playSound("confirm");
    if (wasOpen.current && !open) playSound("back");
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
        // escape-key: cancelled too, then re-driven through our own back-stack instead of
        // calling onClose() directly -- Base UI's own Escape listener stopPropagation()s the
        // keydown unconditionally (see modal.tsx's longer note on this), so our window-level
        // listener never sees it and this onOpenChange callback is the *only* signal physical
        // Escape produces while a Base UI dialog is open. Calling onClose() straight from here
        // would mean Escape always fully closed the drawer, even from a sub-view (Change Box
        // Art / Achievements) that's supposed to back out to the menu first, the same as
        // gamepad B already correctly does via pushBackHandler's LIFO stack. Routing through
        // handleBack() instead makes physical Escape consult that same stack -- it closes the
        // drawer exactly when nothing else is layered on top of it, and otherwise defers to
        // whichever sub-view's own back-handler is currently on top.
        if (eventDetails.reason === "escape-key") {
          eventDetails.cancel();
          handleBack();
          return;
        }
        if (!nextOpen) onClose();
      }}
    >
      <Dialog.Portal>
        <Dialog.Backdrop className="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs transition-opacity duration-200 data-[ending-style]:opacity-0 data-[starting-style]:opacity-0" />
        <Dialog.Viewport className="fixed inset-0 z-50 flex justify-end">
          <Dialog.Popup
            ref={ref}
            initialFocus={false}
            finalFocus={false}
            className={cn(
              "h-full w-full max-w-sm space-y-1 overflow-y-auto bg-zinc-900 p-4 text-card-foreground shadow-2xl",
              "transition-[transform,max-width] duration-200 ease-bounce",
              "data-[ending-style]:translate-x-full data-[starting-style]:translate-x-full",
              className,
            )}
          >
            <FocusContext.Provider value={resolvedFocusKey}>{children}</FocusContext.Provider>
          </Dialog.Popup>
        </Dialog.Viewport>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
