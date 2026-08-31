import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { useCheckForUpdate, useDownloadAndInstallUpdate } from "./use-updates";
import type { Update } from "@tauri-apps/plugin-updater";

const checkMock = vi.fn();
vi.mock("@tauri-apps/plugin-updater", () => ({
  check: (...args: unknown[]) => checkMock(...args),
}));

const relaunchMock = vi.fn();
vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: (...args: unknown[]) => relaunchMock(...args),
}));

function wrapper({ children }: { children: ReactNode }) {
  const queryClient = new QueryClient();
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

describe("useCheckForUpdate", () => {
  it("resolves to null when there's no update", async () => {
    checkMock.mockResolvedValue(null);
    const { result } = renderHook(() => useCheckForUpdate(), { wrapper });

    await act(async () => {
      const update = await result.current.mutateAsync();
      expect(update).toBeNull();
    });
  });

  it("resolves to the Update object when one is available", async () => {
    const update = { version: "1.2.0", currentVersion: "1.1.0" };
    checkMock.mockResolvedValue(update);
    const { result } = renderHook(() => useCheckForUpdate(), { wrapper });

    await act(async () => {
      const resolved = await result.current.mutateAsync();
      expect(resolved).toEqual(update);
    });
  });
});

describe("useDownloadAndInstallUpdate", () => {
  it("downloads, installs, then relaunches", async () => {
    const downloadAndInstall = vi.fn().mockResolvedValue(undefined);
    const update = { downloadAndInstall } as unknown as Update;

    const { result } = renderHook(() => useDownloadAndInstallUpdate(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync(update);
    });

    expect(downloadAndInstall).toHaveBeenCalled();
    expect(relaunchMock).toHaveBeenCalled();
  });
});
