import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

// Mirrors src-tauri/src/game_actions.rs's AlternateMatch -- boxart_url here is the *remote*
// SteamGridDB image, straight from their CDN, for preview only. Nothing's downloaded or
// persisted until the user actually picks one (see useApplyMatch).
export type AlternateMatch = {
  steamgriddb_id: number;
  title: string;
  boxart_url: string | null;
};

// Both actions here are on-demand and one-shot (opening the "change box art" drawer, then picking
// a result) rather than passive derived state, so mutations fit better than a query with a stable
// cache key -- same reasoning as useCreateGame/useRescan elsewhere in this file set.
export function useSearchAlternateMatches() {
  return useMutation({
    mutationFn: (gameId: string) => invoke<AlternateMatch[]>("search_alternate_matches", { gameId }),
  });
}

export function useApplyMatch() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (args: { gameId: string; steamgriddbId: number; title: string }) =>
      invoke<void>("apply_match", { gameId: args.gameId, steamgriddbId: args.steamgriddbId, title: args.title }),
    onSuccess: () => {
      // Applying a match changes the game's title/boxart, which every library view surfaces.
      queryClient.invalidateQueries({ queryKey: ["games"] });
      queryClient.invalidateQueries({ queryKey: ["library"] });
    },
  });
}

// Mirrors src-tauri/src/game_actions.rs's AchievementView/GameAchievementsProgress. badge_url is
// already resolved to the locked/unlocked variant server-side.
export type Achievement = {
  id: number;
  title: string;
  description: string;
  points: number;
  badge_url: string;
  unlocked: boolean;
};

export type GameAchievementsProgress = {
  game_id: number;
  title: string;
  console_name: string;
  num_achievements: number;
  num_awarded_to_user: number;
  user_completion: string;
  highest_award_kind: string | null;
  achievements: Achievement[];
};

// null means "this game isn't matched to a RetroAchievements entry" (or no profile is
// RA-linked) -- a normal, expected outcome, not an error. A query rather than a mutation, unlike
// the two above: this backs a passive "show current progress" view (the drawer's Achievements
// tab, tile focus), not a one-shot user action, even though the backend also opportunistically
// persists the fetched highest_award_kind as a side effect.
// `enabled` defaults to true (the drawer's own Achievements view always wants this fetched), but
// a grid that renders many tiles at once (see game-tile.tsx's All Games usage) needs to gate this
// on focus -- without it, mounting hundreds of tiles fires hundreds of simultaneous achievement
// lookups instead of the one for whichever tile is actually highlighted.
export function useAchievements(gameId: string, options?: { enabled?: boolean }) {
  return useQuery({
    queryKey: ["games", gameId, "achievements"],
    queryFn: () => invoke<GameAchievementsProgress | null>("get_achievements", { gameId }),
    enabled: options?.enabled ?? true,
  });
}
