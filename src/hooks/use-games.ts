import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

export type Game = {
  id: string;
  rom_id: string;
  title: string;
  scanned_title: string | null;
  steamgriddb_id: number | null;
  match_confidence: number | null;
  enriched_at: number | null;
  retroachievements_game_id: number | null;
  retroachievements_matched_at: number | null;
  ra_highest_award_kind: string | null;
  created_at: number;
  updated_at: number;
};

const gamesKey = ["games"] as const;

export function useGames() {
  return useQuery({
    queryKey: gamesKey,
    queryFn: () => invoke<Game[]>("list_games"),
  });
}

export function useGame(id: string) {
  return useQuery({
    queryKey: [...gamesKey, id],
    queryFn: () => invoke<Game | null>("get_game", { id }),
  });
}

export function useCreateGame() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (args: { romId: string; title: string }) =>
      invoke<Game>("create_game", { romId: args.romId, title: args.title }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: gamesKey });
    },
  });
}

export function useUpdateGame() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (args: { id: string; title: string }) =>
      invoke<Game>("update_game", { id: args.id, title: args.title }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: gamesKey });
    },
  });
}

export function useDeleteGame() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => invoke<void>("delete_game", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: gamesKey });
    },
  });
}
