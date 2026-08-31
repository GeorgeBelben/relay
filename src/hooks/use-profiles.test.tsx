import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import {
  useCreateProfile,
  useLinkRaWebApi,
  useProfiles,
  useRaStats,
  useRefreshRaStats,
  useUnlinkRa,
  type ProfileSummary,
  type RaStats,
} from "./use-profiles";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

function makeProfile(overrides: Partial<ProfileSummary>): ProfileSummary {
  return { id: "p1", name: "George", ra_username: null, has_web_api_link: false, has_connect_link: false, ...overrides };
}

describe("useProfiles / useCreateProfile", () => {
  it("refetches the profiles list after creating one", async () => {
    let profiles: ProfileSummary[] = [];
    invokeMock.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "list_profiles") return Promise.resolve(profiles);
      if (cmd === "create_profile") {
        const profile = makeProfile({ name: args!.name as string });
        profiles = [...profiles, profile];
        return Promise.resolve(profile);
      }
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const localWrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => ({ profiles: useProfiles(), create: useCreateProfile() }), { wrapper: localWrapper });

    await waitFor(() => expect(result.current.profiles.data).toEqual([]));

    await act(async () => {
      await result.current.create.mutateAsync("George");
    });

    await waitFor(() => expect(result.current.profiles.data).toEqual([makeProfile({ name: "George" })]));
  });
});

describe("useLinkRaWebApi", () => {
  it("invalidates both the profiles list and that profile's ra-stats query", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "link_ra_web_api") return Promise.resolve(undefined);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const localWrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => useLinkRaWebApi(), { wrapper: localWrapper });

    await act(async () => {
      await result.current.mutateAsync({ profileId: "p1", username: "retrouser", webApiKey: "key" });
    });

    expect(invokeMock).toHaveBeenCalledWith("link_ra_web_api", { profileId: "p1", username: "retrouser", webApiKey: "key" });
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["profiles"] });
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["profiles", "p1", "ra-stats"] });
    });
  });
});

describe("useUnlinkRa", () => {
  it("invalidates the profiles list and ra-stats for that profile", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "unlink_ra") return Promise.resolve(undefined);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const localWrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => useUnlinkRa(), { wrapper: localWrapper });

    await act(async () => {
      await result.current.mutateAsync("p1");
    });

    expect(invokeMock).toHaveBeenCalledWith("unlink_ra", { profileId: "p1" });
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["profiles"] });
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["profiles", "p1", "ra-stats"] });
    });
  });
});

describe("useRaStats / useRefreshRaStats", () => {
  it("returns stats from get_ra_stats and refetches after a refresh", async () => {
    let stats: RaStats | null = null;
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_ra_stats") return Promise.resolve(stats);
      if (cmd === "refresh_ra_stats") {
        stats = { points: 4200, rank: "1,234", recent_unlocks: [], refreshed_at: 0 };
        return Promise.resolve(undefined);
      }
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const localWrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => ({ stats: useRaStats("p1"), refresh: useRefreshRaStats() }), { wrapper: localWrapper });

    await waitFor(() => expect(result.current.stats.data).toBeNull());

    await act(async () => {
      await result.current.refresh.mutateAsync("p1");
    });

    await waitFor(() => expect(result.current.stats.data?.points).toBe(4200));
  });
});
