import { cn } from "@/lib/utils";
import { Game } from "@/types/games";
import { FocusableComponentLayout } from "@noriginmedia/norigin-spatial-navigation-core";

type GameCardProps = {
    game: Game;
    onSelect?: (game: Game) => void;
    onFocus?: (layout: FocusableComponentLayout, game: Game) => void;
}

export function GameCard({ game }: GameCardProps) {
    return (
        <div className={cn("flex aspect-square w-full shrink-0 rounded-2xl bg-neutral-800 p-4 bg-center bg-cover")} data-id={game.id} style={{ backgroundImage: `url(${game.coverArt})` }} />
    );
}