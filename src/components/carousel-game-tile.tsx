import { useState } from "react";
import { RiCheckboxCircleFill, RiGamepadFill, RiTrophyFill } from "@remixicon/react";
import { useFocusable, useMenuHandler } from "@/lib/focus";
import { startGameLaunch } from "@/lib/launch";
import { cn } from "@/lib/cn";
import { CAROUSEL_TILE_WIDTH, CAROUSEL_TILE_WIDTH_FOCUSED } from "./carousel";
import { isRecentlyAdded } from "@/lib/library/isRecentlyAdded";
import { resolveMediaUrl, useMediaRoot } from "@/lib/media";
import { GameCardDrawer } from "./game-card-drawer";
import type { LibraryGame } from "@/hooks/use-library";
import type { RefObject } from "react";
import { useAchievements } from "@/hooks/use-game-actions";
import { AchievementProgressRing } from "./achievement-progress-ring";

type CarouselGameTileProps = {
  game: LibraryGame;
  onFocusTile?: (game: LibraryGame) => void;
  // Set by Carousel when this tile is one of its slides -- lets a virtual focus change drive
  // Embla to the right slide, since Embla has no way to observe that on its own.
  onTileFocus?: () => void;
  // Unlike every other focusable tile, this one doesn't center itself on focus -- the recents
  // row stays pinned at the top of the page regardless of which tile is focused. SECTION_RECENT
  // is always the first thing in this container, so "top of the page" is just scrollTop 0 --
  // simpler and more reliable than asking scrollIntoView to work that out via an anchor element.
  scrollContainerRef: RefObject<HTMLElement | null>;
  // Fires true on focus, false on blur -- see game-tile.tsx for why.
  onFocusChange?: (focused: boolean) => void;
};

// Home's recents row -- see game-tile.tsx for the plain grid version this was split from, and
// carousel.tsx for why growth here is a real width change rather than a transform.
export function CarouselGameTile({ game, onFocusTile, onTileFocus, scrollContainerRef, onFocusChange }: CarouselGameTileProps) {
  const [drawerOpen, setDrawerOpen] = useState(false);
  const mediaRoot = useMediaRoot();
  const boxartUrl = mediaRoot && game.boxart_path ? resolveMediaUrl(mediaRoot, game.boxart_path) : null;

  const { ref, focused, focusSelf } = useFocusable({
    focusKey: game.id,
    onFocus: () => {
      onFocusTile?.(game);
      onTileFocus?.();
      onFocusChange?.(true);
      scrollContainerRef.current?.scrollTo({ top: 0, behavior: "smooth" });
    },
    onBlur: () => onFocusChange?.(false),
    onEnterPress: () => startGameLaunch(game),
  });

  useMenuHandler(game.id, () => setDrawerOpen(true));

  const progress = useAchievements(game.id);

  // See game-tile.tsx for why this matters: Drawer's autoRestoreFocus doesn't actually restore
  // anything on close, so without this, virtual focus is left stuck on the now-hidden drawer
  // instead of coming back to this tile.
  const closeDrawer = () => {
    setDrawerOpen(false);
    focusSelf();
  };

  return (
    <>
      <div
        ref={ref}
        className={cn(
          "relative flex aspect-2/3 shrink-0 flex-col justify-end overflow-hidden rounded-lg p-3 transition-bounce",
          focused ? cn(CAROUSEL_TILE_WIDTH_FOCUSED) : CAROUSEL_TILE_WIDTH,
          boxartUrl ? "" : "bg-gray-800 flex items-center justify-center ",
        )}
      >
        {!boxartUrl && <RiGamepadFill className={cn("text-gray-500 transition-bounce", focused ? "size-24" : "size-16")} />}
        {boxartUrl && <img src={boxartUrl} alt="" className="absolute inset-0 h-full w-full object-cover" />}
        {game.beaten && (
          <div className="bg-white size-6.5 p-px rounded-full absolute top-2 right-2 shadow-sm">
            <RiCheckboxCircleFill className="size-6 text-green-600" aria-label="Beaten" />
          </div>
        )}
        {isRecentlyAdded(game.added_at) && (
          <span className="absolute bottom-2 left-2 rounded-full bg-sky-500 px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide text-white shadow-sm">
            New
          </span>
        )}
        {focused && progress.data && (
          <div
            className="flex size-6.5 items-center justify-center rounded-full bg-black/60 backdrop-blur-md absolute top-1.5 left-1.5 shadow-sm p-0.5"
            aria-label={`${progress.data.num_awarded_to_user}/${progress.data.num_achievements} achievements`}
          >
            <AchievementProgressRing
              percent={(progress.data.num_awarded_to_user / progress.data.num_achievements) * 100}
              className="size-full text-amber-400"
            />
          </div>
        )}
        {progress.data && (
          <div
            className={cn(
              "absolute transition-bounce left-1/2 -translate-x-1/2 bg-[#111111]/60 backdrop-blur-md rounded-xl flex items-center px-3 py-1.5 gap-1.5",
              { "bottom-2 opacity-100": focused, "bottom-0 opacity-0": !focused },
            )}
          >
            <RiTrophyFill className="size-4 text-amber-400" aria-hidden="true" />
            <span className="text-sm font-semibold text-white font-space-mono">
              {progress.data.num_awarded_to_user}/{progress.data.num_achievements}
            </span>
          </div>
        )}
      </div>
      <GameCardDrawer game={game} open={drawerOpen} onClose={closeDrawer} />
    </>
  );
}
