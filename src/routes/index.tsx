import { createFileRoute } from "@tanstack/react-router";
import { useState, type ReactNode } from "react";
import { FocusContext, useFocusable, useRegisterPageFocus } from "@/lib/focus";
import { useActionHints } from "@/lib/hints";
import { useLibrary, useRecentlyAdded, useRecentlyPlayed } from "@/hooks/use-library";
import { Header } from "@/components/header";
import { Carousel } from "@/components/carousel";
import { Grid } from "@/components/grid";
import { CarouselGameTile } from "@/components/carousel-game-tile";
import { SystemTile } from "@/components/system-tile";
import { ViewAllTile } from "@/components/view-all-tile";
import { HeroDetails } from "@/components/hero-details";
import { FocusableLink } from "@/components/focusable-link";
import { cn } from "@/lib/cn";
import type { LibraryGame } from "@/hooks/use-library";

export const Route = createFileRoute("/")({
  component: Home,
});

function Home() {
  const shelves = useLibrary();
  const recentlyPlayed = useRecentlyPlayed();
  const recentlyAdded = useRecentlyAdded();
  const hasSystems = shelves.length > 0;

  // Recently Played wins when there's real play history; until then, fall back to what was
  // scanned in most recently so the row isn't just empty. A trailing "View All" tile is always
  // there regardless, so the row is never truly empty and always autofocuses first.
  const recentGames = recentlyPlayed.length > 0 ? recentlyPlayed : (recentlyAdded ?? []);

  // recentlyAdded is undefined only until its query resolves -- autoFocus has to wait for that,
  // or it fires while View All is the only slide that exists yet (recentGames still []) and
  // locks focus there, never revisiting once the real games render in alongside it.
  const recentGamesReady = recentlyPlayed.length > 0 || recentlyAdded !== undefined;

  // The carousel is the top of the hero; this is the bottom -- name/console for whichever tile
  // is currently highlighted. undefined means "nothing's been focused yet" (falls back to the
  // first real game, matching Carousel's own autoFocus landing there first); null means View All
  // (not a game) is explicitly focused, and has to stay distinct from undefined or there's no way
  // to tell "show the default" apart from "show nothing" -- both would otherwise just be falsy.
  const [highlightedGame, setHighlightedGame] = useState<LibraryGame | null | undefined>(undefined);
  const heroGame = highlightedGame === undefined ? (recentGames[0] ?? null) : highlightedGame;

  const { ref, focusKey } = useFocusable({ focusKey: "HOME" });
  // Not usePageFocus -- Home drives its own initial focus via Carousel's autoFocus (data-dependent,
  // waits on recentGamesReady below), it just still needs to register as the fallback Modal
  // restores to on close (see lib/focus/pageFocus.ts).
  useRegisterPageFocus(focusKey);

  // "Options" only makes sense while a game card specifically has focus -- CarouselGameTile
  // reports that via onFocusChange (fires true/false on focus/blur); SystemTile/ViewAllTile
  // don't report it at all, so it correctly stays false while either of those is focused.
  const [gameFocused, setGameFocused] = useState(false);
  useActionHints([
    { action: "confirm", label: "Select" },
    ...(gameFocused ? [{ action: "menu" as const, label: "Options" }] : []),
  ]);

  return (
    <FocusContext.Provider value={focusKey}>
      <div ref={ref} className="flex flex-1 flex-col overflow-y-auto">
        {/* One full viewport tall, so the recents row sits dead-centre on first load and the
            consoles section below only comes into view after a full page's worth of scrolling. */}
        <Section focusKey="SECTION_RECENT" className="flex min-h-full shrink-0 flex-col pb-16">
          <Header />

          <div className="flex flex-1 flex-col justify-end gap-6">
            <Carousel focusKey="ROW_RECENT" autoFocus={recentGamesReady}>
              {recentGames.map((game) => (
                <CarouselGameTile
                  key={game.id}
                  game={game}
                  onFocusTile={setHighlightedGame}
                  onFocusChange={setGameFocused}
                  scrollContainerRef={ref}
                />
              ))}
              <ViewAllTile onFocusTile={() => setHighlightedGame(null)} scrollContainerRef={ref} />
            </Carousel>

            <HeroDetails key={heroGame?.id} game={heroGame} />
          </div>
        </Section>

        <Section focusKey="SECTION_CONSOLES" className="py-16 border-t-1 border-zinc-800">
          {hasSystems ? (
            <Grid focusKey="GRID_SYSTEMS">
              {shelves.map((shelf) => (
                <SystemTile key={shelf.system_id} shelf={shelf} />
              ))}
            </Grid>
          ) : (
            <EmptyLibrary />
          )}
        </Section>
      </div>
    </FocusContext.Provider>
  );
}

// Purely a focus-context boundary now -- no scroll side effect of its own. Every focusable tile
// centers itself vertically on focus, which handles cross-section transitions for free: focus a
// tile in Consoles and the page scrolls to put *it* mid-screen, no separate "jump to section"
// logic needed.
function Section({ focusKey, className, children }: { focusKey: string; className?: string; children: ReactNode }) {
  const { ref, focusKey: resolvedFocusKey } = useFocusable({ focusKey, trackChildren: true });

  return (
    <FocusContext.Provider value={resolvedFocusKey}>
      <div ref={ref} className={className}>
        {children}
      </div>
    </FocusContext.Provider>
  );
}

function EmptyLibrary() {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-3 px-8 text-center">
      <p className="text-lg font-medium">No games yet</p>
      <p className="max-w-sm text-sm text-muted-foreground">Copy ROMs into ~/Relay/roms, then rescan from Settings.</p>
      <FocusableLink
        to="/settings/library"
        className={(focused) =>
          cn(
            "mt-2 rounded-md px-4 py-2 text-sm transition-bounce",
            focused ? "scale-105 bg-primary text-primary-foreground" : "bg-muted text-muted-foreground",
          )
        }
      >
        Open Library Settings
      </FocusableLink>
    </div>
  );
}
