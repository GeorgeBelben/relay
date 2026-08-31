import { useState } from "react";
import { RiCheckboxCircleFill, RiTrophyFill } from "@remixicon/react";
import { FOCUS_RING_BASE, scrollFocusedIntoView, useFocusable, useMenuHandler } from "@/lib/focus";
import { startGameLaunch } from "@/lib/launch";
import { placeholderGradient } from "@/lib/library/placeholderArt";
import { isRecentlyAdded } from "@/lib/library/isRecentlyAdded";
import { resolveMediaUrl, useMediaRoot } from "@/lib/media";
import { cn } from "@/lib/cn";
import { useAchievements } from "@/hooks/use-game-actions";
import { GameCardDrawer } from "./game-card-drawer";
import { AchievementProgressRing } from "./achievement-progress-ring";
import type { LibraryGame } from "@/hooks/use-library";

type GameTileProps = {
  game: LibraryGame;
  onFocusTile?: (game: LibraryGame) => void;
  // Fires true on focus, false on blur -- lets the screen know whether *a game card specifically*
  // is currently focused (as opposed to some other tile type), so it can show/hide the "Options"
  // hint accordingly instead of it being shown unconditionally everywhere.
  onFocusChange?: (focused: boolean) => void;
};

// Used in the systems/all-games CSS grids -- see carousel-game-tile.tsx for Home's recents row,
// which needs a different sizing/growth mechanism (fixed carousel width vs. this tile's own
// aspect-derived width) and got split out into its own component rather than sharing this one via
// a variant switch.
export function GameTile({ game, onFocusTile, onFocusChange }: GameTileProps) {
  const [drawerOpen, setDrawerOpen] = useState(false);
  const mediaRoot = useMediaRoot();
  const boxartUrl = mediaRoot && game.boxart_path ? resolveMediaUrl(mediaRoot, game.boxart_path) : null;

  const { ref, focused, focusSelf } = useFocusable({
    focusKey: game.id,
    onFocus: () => {
      onFocusTile?.(game);
      onFocusChange?.(true);
      scrollFocusedIntoView(ref.current);
    },
    onBlur: () => onFocusChange?.(false),
    onEnterPress: () => startGameLaunch(game),
  });

  useMenuHandler(game.id, () => setDrawerOpen(true));

  // Gated on `focused`, not just mounted -- this tile can render hundreds of times in the All
  // Games grid, and most will never be focused. Shares its query key with the drawer's own
  // Achievements view, so focusing a tile that was already the drawer's target is a cache hit, not
  // a second fetch.
  const progress = useAchievements(game.id, { enabled: focused });

  // Drawer's own autoRestoreFocus doesn't actually restore anything on close -- it only fires on
  // real unmount, and Drawer never unmounts. Reclaiming focus explicitly here is what makes the
  // tile visibly focused again (and gamepad input do anything at all) once the drawer closes,
  // instead of leaving virtual focus stuck on whatever row was last focused inside it, now
  // invisible.
  const closeDrawer = () => {
    setDrawerOpen(false);
    focusSelf();
  };

  return (
    <>
      <div
        ref={ref}
        className={cn(
          "relative flex w-full opacity-70 outline-transparent aspect-2/3 shrink-0 flex-col justify-end overflow-hidden rounded-lg p-3 transition-bounce",
          FOCUS_RING_BASE,
          focused && "outline-white scale-105 opacity-100",
        )}
        style={boxartUrl ? undefined : { backgroundImage: placeholderGradient(game.id) }}
      >
        {boxartUrl && <img src={boxartUrl} alt="" className="absolute inset-0 h-full w-full object-cover" />}
        {game.beaten && (
          <div className="bg-white size-5.5 p-px rounded-full absolute top-1.5 right-1.5 shadow-sm">
            <RiCheckboxCircleFill className="size-5 text-green-600" aria-label="Beaten" />
          </div>
        )}
        {isRecentlyAdded(game.added_at) && (
          <span className="absolute bottom-1.5 left-1.5 rounded-full bg-sky-500 px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide text-white shadow-sm">
            New
          </span>
        )}
        {focused && progress.data && (
          <div
            className="flex size-5.5 items-center justify-center rounded-full bg-black/60 backdrop-blur-md absolute top-1.5 left-1.5 shadow-sm p-0.5"
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
