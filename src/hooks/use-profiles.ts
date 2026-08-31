import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

// Mirrors src-tauri/src/db/profiles.rs's ProfileSummary -- the redacted, IPC-safe view. The
// encrypted RA credential columns never cross the IPC boundary at all; the frontend only ever
// learns *whether* a profile is linked, plus the username.
export type ProfileSummary = {
  id: string;
  name: string;
  ra_username: string | null;
  has_web_api_link: boolean;
  has_connect_link: boolean;
};

const PROFILES_KEY = ["profiles"] as const;

export function useProfiles() {
  return useQuery({ queryKey: PROFILES_KEY, queryFn: () => invoke<ProfileSummary[]>("list_profiles") });
}

export function useProfile(id: string) {
  return useQuery({ queryKey: [...PROFILES_KEY, id], queryFn: () => invoke<ProfileSummary | null>("get_profile", { id }) });
}

function useInvalidateProfiles() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: PROFILES_KEY });
}

export function useCreateProfile() {
  const invalidate = useInvalidateProfiles();
  return useMutation({
    mutationFn: (name: string) => invoke<ProfileSummary>("create_profile", { name }),
    onSuccess: invalidate,
  });
}

export function useRenameProfile() {
  const invalidate = useInvalidateProfiles();
  return useMutation({
    mutationFn: (args: { id: string; name: string }) => invoke<ProfileSummary>("rename_profile", { id: args.id, name: args.name }),
    onSuccess: invalidate,
  });
}

export function useDeleteProfile() {
  const invalidate = useInvalidateProfiles();
  return useMutation({
    mutationFn: (id: string) => invoke<void>("delete_profile", { id }),
    onSuccess: invalidate,
  });
}

const raStatsKey = (profileId: string) => ["profiles", profileId, "ra-stats"] as const;

// Validates against RA's own API before persisting -- a typo'd key fails immediately in the
// Settings UI instead of silently saving something that only surfaces as a failure later. Also
// caches stats server-side as part of linking, so the ra-stats query needs invalidating alongside
// the profiles list.
export function useLinkRaWebApi() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (args: { profileId: string; username: string; webApiKey: string }) =>
      invoke<void>("link_ra_web_api", { profileId: args.profileId, username: args.username, webApiKey: args.webApiKey }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: PROFILES_KEY });
      queryClient.invalidateQueries({ queryKey: raStatsKey(variables.profileId) });
    },
  });
}

export function useLinkRaConnectAccount() {
  const invalidate = useInvalidateProfiles();
  return useMutation({
    mutationFn: (args: { profileId: string; username: string; password: string }) =>
      invoke<void>("link_ra_connect_account", { profileId: args.profileId, username: args.username, password: args.password }),
    onSuccess: invalidate,
  });
}

// Clears every RA field at once -- the UI offers a single "Unlink RetroAchievements" action, not
// separate ones per auth flow, since both point at the same account. Also drops the cached stats
// (see commands::profiles::unlink_ra's own comment on why, unlike the Electron original).
export function useUnlinkRa() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (profileId: string) => invoke<void>("unlink_ra", { profileId }),
    onSuccess: (_data, profileId) => {
      queryClient.invalidateQueries({ queryKey: PROFILES_KEY });
      queryClient.invalidateQueries({ queryKey: raStatsKey(profileId) });
    },
  });
}

export type RaRecentUnlock = {
  game_title: string;
  title: string;
  points: number;
  badge_url: string;
  unlocked_at: string;
};

export type RaStats = {
  points: number;
  rank: string;
  recent_unlocks: RaRecentUnlock[];
  refreshed_at: number;
};

export function useRaStats(profileId: string) {
  return useQuery({
    queryKey: raStatsKey(profileId),
    queryFn: () => invoke<RaStats | null>("get_ra_stats", { profileId }),
  });
}

// Re-fetches and re-caches stats for a Web-API-linked profile -- run on app start / profile
// switch (see the Electron MVP's refreshActiveProfileStats). A no-op for a profile with no Web
// API link, not an error.
export function useRefreshRaStats() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (profileId: string) => invoke<void>("refresh_ra_stats", { profileId }),
    onSuccess: (_data, profileId) => {
      queryClient.invalidateQueries({ queryKey: raStatsKey(profileId) });
    },
  });
}
