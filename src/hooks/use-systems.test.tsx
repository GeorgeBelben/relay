import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useSystem, useSystems, type System } from "./use-systems";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("useSystems", () => {
  it("fetches the fixed systems list from the backend", async () => {
    const systems: System[] = [
      { id: "nes", name: "NES", extensions: ["nes"], retroarch_core: "nestopia", standalone_binary: null },
      { id: "snes", name: "SNES", extensions: ["sfc", "smc"], retroarch_core: "snes9x", standalone_binary: null },
    ];
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "list_systems") return Promise.resolve(systems);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => useSystems(), { wrapper });

    await waitFor(() => expect(result.current.data).toEqual(systems));
    expect(invokeMock).toHaveBeenCalledWith("list_systems");
  });

  it("fetches a single system by id", async () => {
    const nes: System = { id: "nes", name: "NES", extensions: ["nes"], retroarch_core: "nestopia", standalone_binary: null };
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_system") return Promise.resolve(nes);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => useSystem("nes"), { wrapper });

    await waitFor(() => expect(result.current.data).toEqual(nes));
    expect(invokeMock).toHaveBeenCalledWith("get_system", { id: "nes" });
  });
});
