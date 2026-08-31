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
