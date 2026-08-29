import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useSetSetting, useSetting } from "./use-settings";

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
