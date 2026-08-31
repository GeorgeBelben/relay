import { RiLayoutGridFill } from "@remixicon/react";
import { useNavigate } from "@tanstack/react-router";
import { useFocusable } from "@/lib/focus";
import { cn } from "@/lib/cn";
import { CAROUSEL_TILE_WIDTH } from "./carousel";
import type { RefObject } from "react";

type ViewAllTileProps = {
  onTileFocus?: () => void;
  // Called on focus with no game -- lets Home clear the hero details panel, since there's no
  // game to show details for while this tile is highlighted.
  onFocusTile?: () => void;
  // See carousel-game-tile.tsx -- the recents row stays pinned to the top of the page rather
  // than centering whichever tile is focused.
  scrollContainerRef: RefObject<HTMLElement | null>;
};

// Trailing tile at the end of the Home "recents" row -- always present, even with zero games, so
// there's always a way into the full alphabetical library. Rolls its own useFocusable rather than
// using FocusableLink so it can share CarouselGameTile's sizing (see carousel.tsx).
//
// Deliberately does *not* grow on focus the way CarouselGameTile does -- it's always the last
// slide, and Embla's scroll-range math is computed once assuming every slide stays at its resting
// width (watchResize: false, see carousel.tsx). That's a fine approximation for a game tile
// growing somewhere in the middle of the row, but for the last slide specifically, growing wider
// pushes its true right edge past where Embla thinks the maximum scroll position is -- the
// carousel can't scroll far enough to reveal it, so it's always a little cut off. A ring/color
// change is a perfectly good focus indicator for an action button anyway.
export function ViewAllTile({ onTileFocus, onFocusTile, scrollContainerRef }: ViewAllTileProps) {
  const navigate = useNavigate();
  const { ref, focused } = useFocusable({
    onEnterPress: () => navigate({ to: "/games" }),
    onFocus: () => {
      onTileFocus?.();
      onFocusTile?.();
      scrollContainerRef.current?.scrollTo({ top: 0, behavior: "smooth" });
    },
  });

  return (
    <div
      ref={ref}
      className={cn(
        "flex aspect-2/3 shrink-0 flex-col items-center justify-center gap-1 rounded-lg bg-zinc-900 transition-bounce",
        CAROUSEL_TILE_WIDTH,
        focused ? cn("bg-zinc-800 text-white scale-110") : "text-zinc-300",
      )}
    >
      <RiLayoutGridFill className={cn("transition-bounce", { "size-8": focused, "size-6": !focused })} aria-hidden="true" />
      <span className={cn("transition-bounce font-medium", { "text-lg": focused, "text-sm": !focused })}>View All</span>
    </div>
  );
}
