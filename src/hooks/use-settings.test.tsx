import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import {
  useActiveProfileId,
  useSetActiveProfileId,
  useSetSetting,
  useSetting,
  useSetWallpaper,
  useWallpaper,
  useWallpaperOptions,
  type GeneralSettings,
} from "./use-settings";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("useSetting cache invalidation", () => {
  it("refetches the setting after a mutation invalidates it", async () => {
    let value: string | null = null;

    invokeMock.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "get_setting") return Promise.resolve(value);
      if (cmd === "set_setting") {
        value = args!.value as string;
        return Promise.resolve(undefined);
      }
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(
      () => ({ setting: useSetting("steamgriddbApiKey"), setSetting: useSetSetting("steamgriddbApiKey") }),
      { wrapper },
    );

    await waitFor(() => expect(result.current.setting.data).toBeNull());

    await act(async () => {
      await result.current.setSetting.mutateAsync("abc123");
    });

    await waitFor(() => expect(result.current.setting.data).toBe("abc123"));
  });
});

function generalSettings(overrides: Partial<GeneralSettings>): GeneralSettings {
  return {
    onboarding_completed: false,
    controller_type: "xbox",
    active_profile_id: null,
    retroarch_cores_path: "/usr/lib/x86_64-linux-gnu/libretro",
    wallpaper: null,
    sound_volume: 70,
    rumble_enabled: true,
    ...overrides,
  };
}

describe("useWallpaper / useSetWallpaper / useWallpaperOptions", () => {
  it("defaults to the empty string ('no wallpaper') and updates via set_wallpaper", async () => {
    let settings = generalSettings({});

    invokeMock.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "get_general_settings") return Promise.resolve(settings);
      if (cmd === "set_wallpaper") {
        settings = { ...settings, wallpaper: args!.wallpaper as string | null };
        return Promise.resolve(undefined);
      }
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => ({ wallpaper: useWallpaper(), setWallpaper: useSetWallpaper() }), { wrapper });

    await waitFor(() => expect(result.current.wallpaper).toBe(""));

    await act(async () => {
      await result.current.setWallpaper.mutateAsync("space.jpg");
    });

    expect(invokeMock).toHaveBeenCalledWith("set_wallpaper", { wallpaper: "space.jpg" });
    await waitFor(() => expect(result.current.wallpaper).toBe("space.jpg"));
  });

  it("passes null (not an empty string) to set_wallpaper when clearing the wallpaper", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_general_settings") return Promise.resolve(generalSettings({}));
      if (cmd === "set_wallpaper") return Promise.resolve(undefined);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => useSetWallpaper(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync("");
    });

    expect(invokeMock).toHaveBeenCalledWith("set_wallpaper", { wallpaper: null });
  });

  it("useWallpaperOptions lists filenames from list_wallpapers", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "list_wallpapers") return Promise.resolve(["a.jpg", "b.png"]);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => useWallpaperOptions(), { wrapper });

    expect(result.current).toEqual([]);
    await waitFor(() => expect(result.current).toEqual(["a.jpg", "b.png"]));
  });
});

describe("useActiveProfileId / useSetActiveProfileId", () => {
  it("defaults to null and updates via set_active_profile_id", async () => {
    let settings = generalSettings({});

    invokeMock.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "get_general_settings") return Promise.resolve(settings);
      if (cmd === "set_active_profile_id") {
        settings = { ...settings, active_profile_id: args!.profileId as string | null };
        return Promise.resolve(undefined);
      }
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => ({ activeProfileId: useActiveProfileId(), setActiveProfileId: useSetActiveProfileId() }), { wrapper });

    await waitFor(() => expect(result.current.activeProfileId).toBeNull());

    await act(async () => {
      await result.current.setActiveProfileId.mutateAsync("profile-1");
    });

    expect(invokeMock).toHaveBeenCalledWith("set_active_profile_id", { profileId: "profile-1" });
    await waitFor(() => expect(result.current.activeProfileId).toBe("profile-1"));
  });
});
