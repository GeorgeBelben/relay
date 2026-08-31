import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

export function useSetting(key: string) {
  return useQuery({
    queryKey: ["settings", key],
    queryFn: () => invoke<string | null>("get_setting", { key }),
  });
}

export function useSetSetting(key: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (value: string) => invoke<void>("set_setting", { key, value }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["settings", key] });
    },
  });
}

export type ControllerType = "xbox" | "playstation" | "switch" | "generic";

// Mirrors src-tauri/src/db/settings.rs's GeneralSettings -- Tauri serializes Rust's own field
// names verbatim (no camelCase rename), so this stays snake_case rather than being normalized.
export type GeneralSettings = {
  onboarding_completed: boolean;
  controller_type: ControllerType;
  active_profile_id: string | null;
  retroarch_cores_path: string;
  wallpaper: string | null;
  sound_volume: number;
  rumble_enabled: boolean;
};

const GENERAL_SETTINGS_KEY = ["settings", "general"];

// One bundled read backing every typed settings hook below -- get_general_settings already
// applies defaults/parsing server-side (see settings.rs), so there's no per-key query/default
// duplicated here the way the Electron MVP had one hook file per setting.
export function useGeneralSettings() {
  return useQuery({
    queryKey: GENERAL_SETTINGS_KEY,
    queryFn: () => invoke<GeneralSettings>("get_general_settings"),
  });
}

function useInvalidateGeneralSettings() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: GENERAL_SETTINGS_KEY });
}

export function useSoundVolume(): number {
  const { data } = useGeneralSettings();
  return data?.sound_volume ?? 70;
}

export function useSetSoundVolume() {
  const invalidate = useInvalidateGeneralSettings();
  return useMutation({
    mutationFn: (volume: number) => invoke<void>("set_sound_volume", { volume }),
    onSuccess: invalidate,
  });
}

export function useRumbleEnabled(): boolean {
  const { data } = useGeneralSettings();
  return data?.rumble_enabled ?? true;
}

export function useSetRumbleEnabled() {
  const invalidate = useInvalidateGeneralSettings();
  return useMutation({
    mutationFn: (enabled: boolean) => invoke<void>("set_rumble_enabled", { enabled }),
    onSuccess: invalidate,
  });
}

export function useActiveProfileId(): string | null {
  const { data } = useGeneralSettings();
  return data?.active_profile_id ?? null;
}

export function useSetActiveProfileId() {
  const invalidate = useInvalidateGeneralSettings();
  return useMutation({
    mutationFn: (profileId: string | null) => invoke<void>("set_active_profile_id", { profileId }),
    onSuccess: invalidate,
  });
}

// "" means no wallpaper picked -- matches the backend's own default (settings.rs's wallpaper
// column), so there's one canonical "none" value instead of undefined/null/"" all meaning the
// same thing.
export function useWallpaper(): string {
  const { data } = useGeneralSettings();
  return data?.wallpaper ?? "";
}

export function useSetWallpaper() {
  const invalidate = useInvalidateGeneralSettings();
  return useMutation({
    mutationFn: (wallpaper: string) => invoke<void>("set_wallpaper", { wallpaper: wallpaper || null }),
    onSuccess: invalidate,
  });
}

// Filenames available in ~/Relay/wallpapers, for the picker in Settings. Resolving a filename
// into a loadable image URL is left to whatever component actually renders it (wallpaper-picker.tsx,
// __root.tsx's background) -- same "not this hook's job" reasoning as use-library.ts's boxart_path.
export function useWallpaperOptions(): string[] {
  const { data } = useQuery({
    queryKey: ["wallpaper", "list"],
    queryFn: () => invoke<string[]>("list_wallpapers"),
  });
  return data ?? EMPTY_WALLPAPER_OPTIONS;
}

const EMPTY_WALLPAPER_OPTIONS: string[] = [];
