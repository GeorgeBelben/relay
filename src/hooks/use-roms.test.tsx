import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useCreateRom, useRoms, type Rom } from "./use-roms";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("useRoms cache invalidation", () => {
  it("refetches the roms list after a mutation invalidates it", async () => {
    const before: Rom[] = [];
    const after: Rom[] = [
      {
        id: "r1",
        system_id: "snes",
        path: "snes/game.sfc",
        crc32: null,
        size_bytes: null,
        discs: null,
        status: "ok",
        created_at: 0,
        updated_at: 0,
      },
    ];

    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "list_roms") {
        return Promise.resolve(
          invokeMock.mock.calls.filter(([c]) => c === "list_roms").length <= 1 ? before : after,
        );
      }
      if (cmd === "create_rom") {
        return Promise.resolve(after[0]);
      }
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => ({ roms: useRoms(), create: useCreateRom() }), { wrapper });

    await waitFor(() => expect(result.current.roms.data).toEqual(before));

    await act(async () => {
      await result.current.create.mutateAsync({
        systemId: "snes",
        path: "snes/game.sfc",
        crc32: null,
        sizeBytes: null,
        discs: null,
      });
    });

    await waitFor(() => expect(result.current.roms.data).toEqual(after));

    const listCalls = invokeMock.mock.calls.filter(([cmd]) => cmd === "list_roms");
    expect(listCalls.length).toBeGreaterThanOrEqual(2);
  });
});
