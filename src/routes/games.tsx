import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { FocusContext, useBackHandler, usePageFocus } from "@/lib/focus";
import { useActionHints } from "@/lib/hints";
import { useAllGames } from "@/hooks/use-library";
import { Header } from "@/components/header";
import { GameTile } from "@/components/game-tile";
import { SearchField } from "@/components/search-field";

export const Route = createFileRoute("/games")({
  component: AllGames,
});

function AllGames() {
  const navigate = useNavigate();
  const allGames = useAllGames();

  const [query, setQuery] = useState("");
  const games = useMemo(() => {
    const q = query.trim().toLowerCase();
    return q ? allGames.filter((game) => game.title.toLowerCase().includes(q)) : allGames;
  }, [allGames, query]);

  const { ref, focusKey } = usePageFocus("ALL_GAMES");

  useBackHandler(() => navigate({ to: "/" }));

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
        <div className="flex items-center justify-between gap-4 px-16 pb-12">
          <h1 className="font-space-grotesk text-3xl font-bold">All Games</h1>
          <SearchField value={query} onChange={setQuery} placeholder="Search games" />
        </div>

        {allGames.length === 0 ? (
          <p className="px-8 text-sm text-muted-foreground">No games in your library yet.</p>
        ) : games.length === 0 ? (
          <p className="px-8 text-sm text-muted-foreground">No games match "{query}".</p>
        ) : (
          <div className="grid grid-cols-5 justify-center gap-8 px-16 pb-16">
            {games.map((game) => (
              <GameTile key={game.id} game={game} onFocusChange={setGameFocused} />
            ))}
          </div>
        )}
      </div>
    </FocusContext.Provider>
  );
}
