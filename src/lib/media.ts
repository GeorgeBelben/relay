import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";

// game_media.local_path (LibraryGame.boxart_path) and wallpaper filenames are both stored/served
// relative to a root directory on disk (see commands::game_media::get_media_root_path's and
// system::wallpaper::list_wallpapers' own doc comments) -- resolving one into something an <img>
// can load is deliberately left to the frontend. The root path itself barely ever changes, so it's
// cached indefinitely rather than refetched per tile.
export function useMediaRoot(): string | undefined {
  const { data } = useQuery({
    queryKey: ["media", "root"],
    queryFn: () => invoke<string>("get_media_root_path"),
    staleTime: Infinity,
  });
  return data;
}

export function resolveMediaUrl(root: string, relativePath: string): string {
  return convertFileSrc(`${root}/${relativePath}`);
}
