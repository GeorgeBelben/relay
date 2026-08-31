import { useEffect, useRef } from "react";
import { RiErrorWarningLine } from "@remixicon/react";
import { getCurrentFocusKey, pushBackHandler, setFocus, useFocusable } from "@/lib/focus";
import { useActionHints } from "@/lib/hints";
import { useLaunchStore } from "@/lib/launch";
import { cn } from "@/lib/cn";
import { Logo } from "./logo";

// Mounted once at the app root (see __root.tsx), driven by useLaunchStore rather than local
// per-tile state -- GameTile/CarouselGameTile just call startGameLaunch(game) on confirm and this
// owns the whole sequence from there, same reasoning as UpdateDialog: this needs to keep playing
// regardless of which specific tile triggered it or whether that tile is even still mounted. The
// "logo" -> "error"/dismiss transition itself is driven by useLauncherListener.ts reacting to the
// real "launcher:status" push, not a fixed timeout -- this component only renders whatever
// phase/message that hook has already decided on.
//
// Always rendered (visibility purely via opacity/pointer-events, not mount/unmount) so the fade
// in/out has something to transition from -- see modal.tsx's siblings for why a
// conditionally-mounted overlay can't animate its own entrance the same way.
export function LaunchOverlay() {
  const phase = useLaunchStore((state) => state.phase);
  const game = useLaunchStore((state) => state.game);
  const message = useLaunchStore((state) => state.message);
  const dismiss = useLaunchStore((state) => state.dismiss);

  const open = phase !== "idle";

  const { ref, focusSelf } = useFocusable({
    focusKey: "LAUNCH_OVERLAY",
    focusable: open,
    isFocusBoundary: true,
    onEnterPress: () => {
      if (phase === "error") dismiss();
    },
  });

  // This div is always mounted (see the comment above on why), so norigin's own autoRestoreFocus
  // -- which only fires on real unmount -- never kicks in here. Without this, dismissing just
  // leaves the current focus key pointed at an element that's now focusable: false, with nothing
  // to redirect it: every tile's `focused` check comes up false, so nothing looks focused and
  // directional/confirm presses stop doing anything until you navigate manually. Capture whatever
  // had focus right before taking it, restore it on close.
  const previousFocusKeyRef = useRef<string | null>(null);
  useEffect(() => {
    if (open) {
      previousFocusKeyRef.current = getCurrentFocusKey();
      focusSelf();
    } else if (previousFocusKeyRef.current) {
      setFocus(previousFocusKeyRef.current);
      previousFocusKeyRef.current = null;
    }
  }, [open, focusSelf]);

  useEffect(() => {
    if (!open) return;
    return pushBackHandler(dismiss);
  }, [open, dismiss]);

  useActionHints(!open ? null : phase === "logo" ? [] : [{ action: "back", label: "Dismiss" }]);

  return (
    <div
      ref={ref}
      className={cn(
        // pt-16: the hint bar's visual weight sits at the bottom of the viewport, so centering
        // on the full viewport height reads as sitting slightly too high against it -- padding
        // the top shifts the centered content down to balance against it.
        "fixed inset-0 z-[60] flex flex-col items-center justify-center gap-6 bg-black pt-16 transition-opacity duration-300",
        open ? "opacity-100" : "pointer-events-none opacity-0",
      )}
    >
      <div
        className={cn(
          "absolute transition-all duration-500 ease-bounce",
          phase === "logo" ? "scale-100 opacity-100" : "scale-75 opacity-0",
        )}
      >
        <Logo className="h-20 w-20 animate-pulse text-white" />
      </div>

      <div
        className={cn(
          "absolute flex flex-col items-center gap-2 px-8 text-center transition-all duration-500 ease-bounce",
          phase === "error" ? "translate-y-0 opacity-100" : "-translate-y-4 opacity-0",
        )}
      >
        <RiErrorWarningLine className="h-10 w-10 text-destructive" aria-hidden="true" />
        <p className="text-lg font-semibold">Couldn't launch {game?.title}</p>
        {message && <p className="text-sm text-muted-foreground">{message}</p>}
      </div>
    </div>
  );
}
