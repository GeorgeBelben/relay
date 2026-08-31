import { FocusableLink } from "./focusable-link";
import { FOCUS_RING } from "@/lib/focus";
import { cn } from "@/lib/cn";
import type { LibraryShelf } from "@/hooks/use-library";
import { getConsoleIconUrl } from "@/lib/library/consoleIcons";

export function SystemTile({ shelf }: { shelf: LibraryShelf }) {
  const iconUrl = getConsoleIconUrl(shelf.system_id);

  return (
    <FocusableLink
      to={`/systems/${shelf.system_id}`}
      className={(focused) =>
        cn(
          "flex aspect-square flex-col justify-center rounded-lg p-16 bg-zinc-900 transition-bounce",
          focused && cn("scale-103 bg-zinc-800", FOCUS_RING),
        )
      }
    >
      <img src={iconUrl} alt={shelf.system_name} className={cn("w-full h-auto opacity-80")} />
    </FocusableLink>
  );
}
