import type { LibraryGame } from "@/hooks/use-library";

// Sits underneath the recents carousel on Home -- name and console for whichever tile is
// currently highlighted. Keyed by game.id at the call site so it remounts (and replays its
// entrance animation) every time the highlight moves, rather than just re-rendering in place.
//
// No year or short description -- SteamGridDB (the box art source) is art-only and doesn't have
// either. Add both here once/if a metadata source that has them exists.
export function HeroDetails({ game }: { game: LibraryGame | null }) {
  if (!game) return <div className="h-15"></div>;

  return (
    <div className="animate-hero-details-in flex flex-col items-start gap-1 px-16 text-center">
      <h1 className="max-w-2xl text-3xl font-bold text-white">{game.title}</h1>
      <span className="text-sm text-zinc-400 font-space-mono">{game.system_name}</span>
    </div>
  );
}
