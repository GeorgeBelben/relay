import { createFileRoute, useRouter } from "@tanstack/react-router";
import { useState } from "react";
import { FocusContext, useBackHandler, usePageFocus } from "@/lib/focus";
import { useActionHints } from "@/lib/hints";
import { useLibrary } from "@/hooks/use-library";
import { Header } from "@/components/header";
import { GameTile } from "@/components/game-tile";

export const Route = createFileRoute("/systems/$systemId")({
  component: SystemLibrary,
});

// Just a big grid -- no shelves, no hero, nothing else to navigate here besides the games
// themselves. Norigin's spatial nav handles 2D up/down/left/right movement natively as long as
// the tiles are laid out in a real CSS grid, so this needs no special focus-tree wiring beyond
// registering all of them under one container.
function SystemLibrary() {
  const { systemId } = Route.useParams();
  const router = useRouter();
  const shelf = useLibrary().find((s) => s.system_id === systemId);

  const { ref, focusKey } = usePageFocus(`SYSTEM_${systemId}`);

  useBackHandler(() => router.history.back());

  // "Options" only makes sense while a game card specifically has focus -- GameTile reports that
  // via onFocusChange (fires true/false on focus/blur) rather than this being shown always.
  const [gameFocused, setGameFocused] = useState(false);
  useActionHints([
    { action: "confirm", label: "Play" },
    ...(gameFocused ? [{ action: "menu" as const, label: "Options" }] : []),
    { action: "back", label: "Back" },
  ]);

  return (
    <FocusContext.Provider value={focusKey}>
      <div ref={ref} className="flex flex-1 flex-col overflow-y-auto">
        <Header />
        <div className="space-y-1 pt-6 py-12 px-16">
          <h1 className="font-space-grotesk text-3xl font-bold">{shelf?.system_name ?? systemId}</h1>
          <p className="text-zinc-400 font-space-mono">
            {shelf?.games.length} {shelf?.games.length === 1 ? "game" : "games"}
          </p>
        </div>
        {!shelf || shelf.games.length === 0 ? (
          <p className="px-8 text-sm text-muted-foreground">No games found for this system.</p>
        ) : (
          <div className="grid grid-cols-5 justify-center gap-8 px-16 pb-16">
            {shelf.games.map((game) => (
              <GameTile key={game.id} game={game} onFocusChange={setGameFocused} />
            ))}
          </div>
        )}
      </div>
    </FocusContext.Provider>
  );
}
