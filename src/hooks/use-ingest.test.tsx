import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useScanStatus, type ScanStatus } from "./use-ingest";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

type Listener = (event: { payload: ScanStatus }) => void;
let capturedListener: Listener | undefined;
const unlistenMock = vi.fn();
const listenMock = vi.fn((_event: string, handler: Listener) => {
  capturedListener = handler;
  return Promise.resolve(unlistenMock);
});
vi.mock("@tauri-apps/api/event", () => ({
  listen: (event: string, handler: Listener) => listenMock(event, handler),
}));

function wrapper({ children }: { children: ReactNode }) {
  const queryClient = new QueryClient();
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

describe("useScanStatus", () => {
  it("pulls the initial status on mount, then stays live via the push event", async () => {
    invokeMock.mockResolvedValue({ state: "idle" });

    const { result } = renderHook(() => useScanStatus(), { wrapper });

    await waitFor(() => expect(result.current).toEqual({ state: "idle" }));
    expect(invokeMock).toHaveBeenCalledWith("get_scan_status");

    capturedListener!({ payload: { state: "scanning-files" } });
    await waitFor(() => expect(result.current).toEqual({ state: "scanning-files" }));

    capturedListener!({ payload: { state: "enriching-art", current: 3, total: 10 } });
    await waitFor(() => expect(result.current).toEqual({ state: "enriching-art", current: 3, total: 10 }));
  });

  it("invalidates games/roms queries when the push event reports done", async () => {
    invokeMock.mockResolvedValue({ state: "idle" });
    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const localWrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    renderHook(() => useScanStatus(), { wrapper: localWrapper });
    await waitFor(() => expect(listenMock).toHaveBeenCalled());

    capturedListener!({ payload: { state: "done" } });

    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["games"] });
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["roms"] });
    });
  });
});
