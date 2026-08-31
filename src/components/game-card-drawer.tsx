import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { RiImageEditLine, RiLoader4Line } from "@remixicon/react";
import { toast } from "sonner";
import { Drawer } from "./drawer";
import { List, ListRow } from "./list";
import { AchievementsView } from "./achievements-view";
import { FocusContext, useBackHandler, useFocusable } from "@/lib/focus";
import { playSound } from "@/lib/sound";
import { cn } from "@/lib/cn";
import {
  useApplyMatch,
  useSearchAlternateMatches,
  type AlternateMatch,
} from "@/hooks/use-game-actions";
import type { LibraryGame } from "@/hooks/use-library";

type GameCardDrawerProps = {
  game: LibraryGame;
  open: boolean;
  onClose: () => void;
};

type View = "menu" | "picker" | "achievements";

// Context menu for a single game card (see carousel-game-tile.tsx / game-tile.tsx), opened by
// the "menu" action (Y/Triangle/X, or M on keyboard -- see lib/input/keyboard.ts). Three views
// inside one drawer: the action list, a grid of alternate SteamGridDB matches ("Change Box Art"),
// and this game's RetroAchievements list. Back from either sub-view returns to the menu instead
// of closing the drawer -- each registers its own back-handler, layered on top of Drawer's own
// (LIFO, see lib/focus/backStack.ts), only while that sub-view is actually showing.
export function GameCardDrawer({ game, open, onClose }: GameCardDrawerProps) {
  const [view, setView] = useState<View>("menu");

  // Explicit here for the same reason drawer.tsx/modal.tsx play their own "back" on a real
  // open->close transition: Base UI's Dialog swallows the physical Escape keydown outright while
  // it's open (see drawer.tsx's own comment), so the generic input-stream sound never fires for a
  // keyboard back-press in that state. Drawer's own sound only covers *its* open/close transition,
  // not this sub-view swap, which doesn't touch `open` at all -- without this, keyboard Escape
  // from Achievements/Change Box Art back to the menu was silent (gamepad B still worked, since it
  // never goes through Base UI's Dialog at all). Already debounced against itself in
  // soundManager.ts, so this collapses cleanly with whatever the generic stream already played for
  // a gamepad-driven press instead of doubling up.
  const backToMenu = () => {
    playSound("back");
    setView("menu");
  };

  const closeAndReset = () => {
    onClose();
    // Deliberately not reset immediately -- reset after the close transition, so the drawer
    // doesn't visibly flash back to the menu view while it's still animating shut.
    setTimeout(() => setView("menu"), 200);
  };

  return (
    // Both sub-views grow to 2/3 width -- box art tiles and achievement descriptions are too
    // cramped at the menu's narrow max-w-sm. Menu view stays narrow since it's just a few rows.
    <Drawer
      open={open}
      onClose={closeAndReset}
      focusKey={`GAME_DRAWER_${game.id}`}
      className={view !== "menu" ? "max-w-[66vw]" : undefined}
    >
      {view === "menu" && (
        <GameCardMenu
          game={game}
          onChangeBoxArt={() => setView("picker")}
          onAchievements={() => setView("achievements")}
        />
      )}
      {view === "picker" && (
        <BoxArtPicker game={game} onBack={backToMenu} onApplied={closeAndReset} />
      )}
      {view === "achievements" && <AchievementsView game={game} onBack={backToMenu} />}
    </Drawer>
  );
}

// game.beaten comes from RetroAchievements' own award ladder, not a self-reported flag -- there's
// nothing left for this menu to let the player set directly; opening Achievements is what
// actually refreshes it (see get_achievements).
function GameCardMenu({
  game,
  onChangeBoxArt,
  onAchievements,
}: {
  game: LibraryGame;
  onChangeBoxArt: () => void;
  onAchievements: () => void;
}) {
  return (
    <>
      <h2 className="truncate px-4 pb-2 text-base font-semibold">{game.title}</h2>
      <List>
        <ListRow label="Change Box Art" onSelect={onChangeBoxArt} />
        <ListRow label="Achievements" onSelect={onAchievements} />
      </List>
    </>
  );
}

function BoxArtPicker({
  game,
  onBack,
  onApplied,
}: {
  game: LibraryGame;
  onBack: () => void;
  onApplied: () => void;
}) {
  const queryClient = useQueryClient();

  // Back from the picker returns to the menu, not close-the-drawer -- registered while this view
  // is mounted, unregistered when it isn't, taking priority over Drawer's own close-on-back
  // underneath it (LIFO, see lib/focus/backStack.ts).
  useBackHandler(onBack);

  const alternates = useSearchAlternateMatches();
  useEffect(() => {
    alternates.mutate(game.id);
    // Fires once per mount (view switch to "picker") -- game.id is stable for the lifetime of
    // this component, and re-running on every render would refire the search on any unrelated
    // state change.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [game.id]);

  // Drawer's own focusSelf() only runs when the drawer opens, not on this view switch -- the row
  // that was focused in the menu view unmounts when it swaps in, so this view needs its own
  // explicit landing spot. Keyed on the data arriving (not just mount) because the tiles it's
  // trying to focus don't exist yet on first render -- calling focusSelf() into an empty
  // container before the search resolves would have nothing to land on. saveLastFocusedChild is
  // required for focusSelf() to actually descend into the first tile rather than just focusing
  // this container itself (see drawer.tsx for the same gap and why it matters).
  const { ref, focusKey, focusSelf } = useFocusable({
    trackChildren: true,
    saveLastFocusedChild: true,
  });
  useEffect(() => {
    if (alternates.data) focusSelf();
  }, [alternates.data, focusSelf]);

  const applyMatch = useApplyMatch();

  return (
    <FocusContext.Provider value={focusKey}>
      <div ref={ref}>
        <h2 className="truncate px-4 pb-2 text-base font-semibold">Change Box Art</h2>

        {alternates.isPending && (
          <p className="flex items-center gap-2 px-4 py-6 text-sm text-muted-foreground">
            <RiLoader4Line className="h-4 w-4 animate-spin" aria-hidden="true" />
            Searching SteamGridDB…
          </p>
        )}

        {alternates.isError && (
          <p className="px-4 py-6 text-sm text-destructive">Couldn't search SteamGridDB.</p>
        )}

        {alternates.data && alternates.data.length === 0 && (
          <p className="px-4 py-6 text-sm text-muted-foreground">
            No other matches found for "{game.title}".
          </p>
        )}

        {alternates.data && alternates.data.length > 0 && (
          <div className="grid grid-cols-4 gap-3 px-4 py-2 sm:grid-cols-6">
            {alternates.data.map((candidate) => (
              <AlternateTile
                key={candidate.steamgriddb_id}
                candidate={candidate}
                applying={
                  applyMatch.isPending &&
                  applyMatch.variables?.steamgriddbId === candidate.steamgriddb_id
                }
                disabled={applyMatch.isPending}
                onSelect={() =>
                  applyMatch.mutate(
                    {
                      gameId: game.id,
                      steamgriddbId: candidate.steamgriddb_id,
                      title: candidate.title,
                    },
                    {
                      onSuccess: () => {
                        queryClient.invalidateQueries({ queryKey: ["library"] });
                        onApplied();
                      },
                      onError: () => toast("Couldn't apply that match"),
                    },
                  )
                }
              />
            ))}
          </div>
        )}
      </div>
    </FocusContext.Provider>
  );
}

function AlternateTile({
  candidate,
  applying,
  disabled,
  onSelect,
}: {
  candidate: AlternateMatch;
  applying: boolean;
  disabled: boolean;
  onSelect: () => void;
}) {
  const { ref, focused } = useFocusable({ focusable: !disabled, onEnterPress: onSelect });

  return (
    <div
      ref={ref}
      className={cn(
        "relative flex aspect-2/3 flex-col justify-end overflow-hidden rounded-lg bg-gray-800 p-2 transition-bounce",
        focused && "scale-105 ring-4 ring-inset ring-amber-500",
        disabled && !applying && "opacity-40",
      )}
    >
      {candidate.boxart_url && (
        <img
          src={candidate.boxart_url}
          alt=""
          className="absolute inset-0 h-full w-full object-cover"
        />
      )}
      {applying && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/50">
          <RiLoader4Line className="h-6 w-6 animate-spin text-white" aria-hidden="true" />
        </div>
      )}
      {!candidate.boxart_url && !applying && (
        <RiImageEditLine
          className="absolute inset-0 m-auto h-6 w-6 text-muted-foreground"
          aria-hidden="true"
        />
      )}
      <span className="relative line-clamp-2 text-xs font-medium text-white drop-shadow">
        {candidate.title}
      </span>
    </div>
  );
}
