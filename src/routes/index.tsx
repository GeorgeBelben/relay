import { createFileRoute, useRouter } from "@tanstack/react-router";
import { cn } from "@/lib/utils";
import { games } from "@/mock-data/games";
// import { FocusableLink } from "@/components/focusable";
// import { GameCard } from "@/components/game-card";
// import { GameControllerIcon, GearSixIcon } from "@phosphor-icons/react";
import { FocusContext, useFocusable } from "@noriginmedia/norigin-spatial-navigation-react";
import { useEffect, useState } from "react";
import { Game } from "@/types/games";
import { rememberFocus, restoreFocus } from "@/lib/focus/restore";
import { RecentGamesRow } from "@/components/recent-games-row";
import { useActionHints } from "@/lib/hints";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {


  return (
    <main
      className={cn(
        "flex flex-col gap-16 min-h-screen items-center justify-center",
      )}
    >
      <RecentGames />
      {/* <HomeMenu /> */}
    </main>
  );
}

function RecentGames() {

  const { navigate } = useRouter();
  const { ref, focusKey, focusSelf } = useFocusable({ focusKey: 'RECENT_GAMES' });

  const [focusedGame, setFocusedGame] = useState<Game | null>(null);

  const recentGames = games.filter((game) => game.lastPlayed !== null).sort((a, b) => {
    if (a.lastPlayed && b.lastPlayed) {
      return new Date(b.lastPlayed).getTime() - new Date(a.lastPlayed).getTime();
    }
    return 0;
  }).slice(0, 20);

  useEffect(() => {
    restoreFocus('recent-games', focusSelf);
  }, [focusSelf]);

  const handleOnSelectGame = (game: Game) => {
    navigate({ to: "/games/$gameId", params: { gameId: game.id } });
  }

  const handleOnFocusGame = (game: Game) => {
    console.log(`Focused game: ${game.name}`);
    rememberFocus('recent-games', game.id);
    setFocusedGame(game);
  }

  useActionHints([
    { action: "confirm", label: "Select" },
    ...(focusedGame ? [{ action: "menu" as const, label: "Options" }] : []),
  ]);

  return (
    <FocusContext.Provider value={focusKey}>
      <div ref={ref} className="max-w-full">
        {/* <div
          key={`backdrop-${focusedGame?.id}`}
          className="opacity-20 absolute inset-0 -z-1 bg-cover bg-center animate-in fade-in duration-700 ease-out-expo"
          style={{ backgroundImage: `url(${focusedGame?.backdropArt})` }}
        /> */}
        <p key={`detail-${focusedGame?.id}`} className="mb-4 pl-32 animate-in fade-in slide-in-from-top-2 duration-300 ease-snappy">
          {focusedGame?.playtimeMinutes}
        </p>
        <RecentGamesRow games={recentGames} onFocus={handleOnFocusGame} onSelect={handleOnSelectGame} />
        <p key={`title-${focusedGame?.id}`} className="mt-4 pl-32 text-3xl font-bold animate-in fade-in slide-in-from-bottom-2 duration-300 ease-out-back">
          {focusedGame?.name}
        </p>
      </div>
    </FocusContext.Provider>

  )
}

// function HomeMenu() {
//   const { ref, focusKey } = useFocusable({ focusKey: 'HOME_MENU' });

//   return (
//     <FocusContext.Provider value={focusKey}>
//       <div ref={ref} className="rounded-full bg-neutral-700 px-6 py-3 flex items-center gap-4">
//         <FocusableLink to="/games">
//           <GameControllerIcon size={32} />
//         </FocusableLink>
//         <FocusableLink to="/settings">
//           <GearSixIcon size={32} />
//         </FocusableLink>
//       </div>
//     </FocusContext.Provider>
//   )
// }