import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";

// Mirrors src-tauri/src/db/library.rs's LibraryGame/LibraryShelf -- boxart_path is forward-slash-
// relative to the media root (get_media_root_path), not a loadable URL; resolving it into one is
// left to whatever component actually renders it (asset-protocol scope vs. a byte-serving
// command), not this hook's job.
export type LibraryGame = {
  id: string;
  title: string;
  system_id: string;
  system_name: string;
  boxart_path: string | null;
  beaten: boolean;
  added_at: number;
};

export type LibraryShelf = { system_id: string; system_name: string; games: LibraryGame[] };

const EMPTY_SHELVES: LibraryShelf[] = [];
const EMPTY_GAMES: LibraryGame[] = [];

// One shelf per system with at least one scanned game -- backs the Home screen.
export function useLibrary(): LibraryShelf[] {
  const { data } = useQuery({
    queryKey: ["library"],
    queryFn: () => invoke<LibraryShelf[]>("list_library_shelves"),
  });
  return data ?? EMPTY_SHELVES;
}

// Every playable game, alphabetical by title -- backs the "All Games" grid.
export function useAllGames(): LibraryGame[] {
  const { data } = useQuery({
    queryKey: ["library", "allGames"],
    queryFn: () => invoke<LibraryGame[]>("list_all_games_in_library"),
  });
  return data ?? EMPTY_GAMES;
}

// Fallback for the Home "recents" row when there's no real play history yet. Returns `undefined`
// while the query is still pending, rather than collapsing that into an empty array -- Home's
// carousel needs to tell "genuinely no recent games" apart from "hasn't loaded yet" so it doesn't
// autofocus into the trailing View All tile as if it were the only real slide.
export function useRecentlyAdded(): LibraryGame[] | undefined {
  const { data } = useQuery({
    queryKey: ["library", "recentlyAdded"],
    queryFn: () => invoke<LibraryGame[]>("list_recently_added_games"),
  });
  return data;
}

// No real play tracking yet -- that needs play-session data this codebase doesn't have (the
// launcher only reports process lifecycle, not a persisted play log). Always empty for now; Home
// shows placeholder boxes instead of a real row. Kept as its own hook so wiring in a real data
// source later doesn't touch call sites.
export function useRecentlyPlayed(): LibraryGame[] {
  return [];
}
