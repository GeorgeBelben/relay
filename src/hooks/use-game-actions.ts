import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQueryClient } from "@tanstack/react-query";

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
