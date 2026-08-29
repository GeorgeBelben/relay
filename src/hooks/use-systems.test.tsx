import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useCreateSystem, useSystems, type System } from "./use-systems";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("useSystems cache invalidation", () => {
  it("refetches the systems list after a mutation invalidates it", async () => {
    const before: System[] = [];
    const after: System[] = [
      { id: "nes", name: "NES", extensions: '["nes"]', retroarch_core: "mesen", standalone_binary: null },
    ];

    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "list_systems") {
        return Promise.resolve(
          invokeMock.mock.calls.filter(([c]) => c === "list_systems").length <= 1 ? before : after,
        );
      }
      if (cmd === "create_system") {
        return Promise.resolve(after[0]);
      }
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => ({ systems: useSystems(), create: useCreateSystem() }), { wrapper });

    await waitFor(() => expect(result.current.systems.data).toEqual(before));

    await act(async () => {
      await result.current.create.mutateAsync({
        id: "nes",
        name: "NES",
        extensions: '["nes"]',
        retroarchCore: "mesen",
        standaloneBinary: null,
      });
    });

    await waitFor(() => expect(result.current.systems.data).toEqual(after));

    const listCalls = invokeMock.mock.calls.filter(([cmd]) => cmd === "list_systems");
    expect(listCalls.length).toBeGreaterThanOrEqual(2);
  });
});
