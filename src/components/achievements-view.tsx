import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { RiLoader4Line } from "@remixicon/react";
import { FocusContext, scrollFocusedIntoView, useBackHandler, useFocusable } from "@/lib/focus";
import { cn } from "@/lib/cn";
import { useAchievements, type Achievement } from "@/hooks/use-game-actions";
import type { LibraryGame } from "@/hooks/use-library";

// Shared by GameCardDrawer's own Achievements tab and the in-game quick menu (REL-23) -- same
// read-only view of a game's RetroAchievements progress, just opened from two different places.

// Read-only -- no candidates to pick between, just RetroAchievements' own answer for "is this
// game matched, and what has the player earned." null (as opposed to an error) specifically means
// "not matched to a RetroAchievements game" -- unsupported system, no RA entry for this ROM, or no
// profile is RA-linked -- all of which read the same to a player: nothing to show here (yet).
const AWARD_LABELS: Record<string, string> = {
  "beaten-softcore": "Beaten",
  "beaten-hardcore": "Beaten (Hardcore)",
  completed: "Completed",
  mastered: "Mastered",
};

export function AchievementsView({ game, onBack }: { game: LibraryGame; onBack: () => void }) {
  useBackHandler(onBack);
  const queryClient = useQueryClient();

  const progress = useAchievements(game.id);

  // Whichever container this mounts into (Drawer, Modal) only calls its own focusSelf() on its
  // own open transition, not on a sub-view swap like this one -- the row that had focus in the
  // previous view unmounts when this view swaps in. Keyed on the data arriving since there's
  // nothing to focus into before then. Also invalidates the library queries here -- this fetch is
  // what persists game.beaten server-side (get_achievements), so whatever tile/UI this opened from
  // should reflect it without needing a manual rescan/renavigate.
  const { ref, focusKey, focusSelf } = useFocusable({
    trackChildren: true,
    saveLastFocusedChild: true,
  });
  useEffect(() => {
    if (progress.data) {
      focusSelf();
      queryClient.invalidateQueries({ queryKey: ["library"] });
    }
  }, [progress.data, focusSelf, queryClient]);

  return (
    <FocusContext.Provider value={focusKey}>
      <div ref={ref}>
        <h2 className="truncate px-4 pb-2 text-base font-semibold">{game.title} -- Achievements</h2>

        {progress.isPending && (
          <p className="flex items-center gap-2 px-4 py-6 text-sm text-muted-foreground">
            <RiLoader4Line className="h-4 w-4 animate-spin" aria-hidden="true" />
            Loading achievements…
          </p>
        )}

        {progress.isError && (
          <p className="px-4 py-6 text-sm text-destructive">{(progress.error as Error).message}</p>
        )}

        {progress.data === null && (
          <p className="px-4 py-6 text-sm text-muted-foreground">
            Not matched to a RetroAchievements game.
          </p>
        )}

        {progress.data && (
          <>
            <div className="flex items-center gap-2 px-4 pb-3">
              <p className="text-sm text-muted-foreground">
                {progress.data.num_awarded_to_user} / {progress.data.num_achievements} unlocked (
                {progress.data.user_completion})
              </p>
              {progress.data.highest_award_kind && (
                <span className="rounded-full bg-sky-500 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-white">
                  {AWARD_LABELS[progress.data.highest_award_kind] ??
                    progress.data.highest_award_kind}
                </span>
              )}
            </div>
            <div className="space-y-1 px-4 pb-4">
              {progress.data.achievements.map((achievement) => (
                <AchievementRow key={achievement.id} achievement={achievement} />
              ))}
            </div>
          </>
        )}
      </div>
    </FocusContext.Provider>
  );
}

function AchievementRow({ achievement }: { achievement: Achievement }) {
  // Focusable purely so d-pad down can reach (and scroll to) achievements past the fold -- with
  // no mouse in the picture, focus movement is the only thing that can drive scrolling here.
  // onEnterPress deliberately omitted: there's nothing to *do* to an achievement, just look at it.
  const { ref, focused } = useFocusable({
    onFocus: () => scrollFocusedIntoView(ref.current),
  });

  return (
    <div
      ref={ref}
      className={cn(
        "flex items-center gap-3 rounded px-3 py-2 transition-bounce",
        focused ? "bg-zinc-700" : "bg-zinc-800",
        !achievement.unlocked && "opacity-50",
      )}
    >
      <img src={achievement.badge_url} alt="" className="size-10 shrink-0 rounded" />
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium">{achievement.title}</p>
        <p className="truncate text-xs text-muted-foreground">{achievement.description}</p>
      </div>
      <span className="shrink-0 text-xs font-medium text-muted-foreground">
        {achievement.points}pt
      </span>
    </div>
  );
}
