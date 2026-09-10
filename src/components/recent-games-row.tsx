import { Game } from "@/types/games";
import { FocusContext, useFocusable } from "@noriginmedia/norigin-spatial-navigation-react";
import { useCallback, useRef } from "react";
import { GameCard } from "./game-card";
import { cn } from "@/lib/utils";

interface RecentGamesRowProps {
    games: Game[];
    onFocus: (game: Game) => void;
    onSelect: (game: Game) => void;
}

// Matches the card's `w-48` / row's `gap-4`. Kept as constants (not measured off the DOM)
// so the scroll target is exact and independent of the focused card's grow transition --
// measuring a card mid-transition is what caused the drift (see content-row scroll bug).
const CARD_WIDTH_PX = 192;
const CARD_GAP_PX = 16;
const CARD_STEP_PX = CARD_WIDTH_PX + CARD_GAP_PX;

// Scroll target for index 0 is 0, so the container's own `pl-32` (128px) below is what
// actually produces the constant left inset for every card, including the first.

export function RecentGamesRow({ games, onFocus, onSelect }: RecentGamesRowProps) {
    const scrollRef = useRef<HTMLDivElement>(null);
    const { ref, focusKey } = useFocusable({ focusKey: 'CONTENT_ROW' });

    const onCardFocus = useCallback((index: number, game: Game) => {
        scrollRef.current?.scrollTo({
            left: index * CARD_STEP_PX,
            behavior: 'smooth'
        });
        onFocus(game);
    }, [onFocus]);

    return (
        <FocusContext.Provider value={focusKey}>
            <div ref={scrollRef} className="overflow-x-auto overflow-y-visible pl-32 ">
                <div
                    ref={ref}
                    className="flex items-center gap-4 min-h-80 "
                >
                    {games.map((item, index) => (
                        <FocusableGameCard key={item.id} game={item} onFocus={(game) => onCardFocus(index, game)} onSelect={onSelect} />
                    ))}
                    {/* A real flex item, not `pr-*` on the scroll container -- Chromium's scrollable
                    overflow calc for a flex row doesn't reliably include trailing padding on the
                    scroll container itself, so it never showed up as extra scrollWidth. It also has
                    to have a non-zero height: a `height: 0` flex item (no content, no h-* class) gets
                    excluded from the scrollable-overflow calc entirely, so `w-32` alone (0 height)
                    silently contributed nothing -- confirmed by toggling it in the live DOM. Width is
                    128 minus one `gap-4` (16px), since the row's `gap-4` also applies between the
                    last card and this spacer, adding its own 16px on top of the spacer's own width. */}
                    <div className="w-28 h-px shrink-0" aria-hidden="true" />
                </div>
            </div>
        </FocusContext.Provider>
    );
}

type FocusableGameCardProps = {
    game: Game;
    onSelect: (game: Game) => void;
    onFocus: (game: Game) => void;
}

function FocusableGameCard({ game, onSelect, onFocus }: FocusableGameCardProps) {
    const { ref, focused } = useFocusable({
        focusKey: game.id,
        onEnterPress: () => onSelect(game),
        onFocus: () => onFocus(game),
    });

    return (
        <div ref={ref} className={cn("w-48 shrink-0 transition-all ease-snappy outline-4 outline-transparent outline-offset-6 rounded-2xl", { "w-72 mx-2 outline-white": focused })}>
            <GameCard game={game} />
        </div>
    )
}