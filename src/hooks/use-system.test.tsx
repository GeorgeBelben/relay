import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useQuit, useUsername } from "./use-system";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

function wrapper({ children }: { children: ReactNode }) {
  const queryClient = new QueryClient();
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

describe("useUsername", () => {
  it("resolves to the value from get_username", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_username") return Promise.resolve("relay");
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const { result } = renderHook(() => useUsername(), { wrapper });

    expect(result.current).toBeUndefined();
    await waitFor(() => expect(result.current).toBe("relay"));
  });
});

describe("useQuit", () => {
  it("invokes the quit command", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "quit") return Promise.resolve(undefined);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const { result } = renderHook(() => useQuit(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync();
    });

    expect(invokeMock).toHaveBeenCalledWith("quit");
  });
});
