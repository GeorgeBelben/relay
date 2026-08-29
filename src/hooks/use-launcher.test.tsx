import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useLauncherLog, useLauncherStatus, type LauncherStatus, type LogLine } from "./use-launcher";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

type Listener<T> = (event: { payload: T }) => void;
const listeners: Record<string, Listener<unknown>> = {};
const unlistenMock = vi.fn();
const listenMock = vi.fn((event: string, handler: Listener<unknown>) => {
  listeners[event] = handler;
  return Promise.resolve(unlistenMock);
});
vi.mock("@tauri-apps/api/event", () => ({
  listen: (event: string, handler: Listener<unknown>) => listenMock(event, handler),
}));

function emit<T>(event: string, payload: T) {
  listeners[event]?.({ payload });
}

function wrapper({ children }: { children: ReactNode }) {
  const queryClient = new QueryClient();
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

describe("useLauncherStatus", () => {
  it("pulls the initial status on mount, then stays live via the push event", async () => {
    invokeMock.mockResolvedValue({ state: "idle" });

    const { result } = renderHook(() => useLauncherStatus(), { wrapper });

    await waitFor(() => expect(result.current).toEqual({ state: "idle" }));
    expect(invokeMock).toHaveBeenCalledWith("get_launcher_status");

    emit<LauncherStatus>("launcher:status", { state: "running" });
    await waitFor(() => expect(result.current).toEqual({ state: "running" }));

    emit<LauncherStatus>("launcher:status", { state: "crashed", exit_code: 1, signal: null });
    await waitFor(() =>
      expect(result.current).toEqual({ state: "crashed", exit_code: 1, signal: null }),
    );
  });
});

describe("useLauncherLog", () => {
  it("accumulates log lines and clears them on the next launch", async () => {
    invokeMock.mockResolvedValue({ state: "idle" });

    const { result } = renderHook(() => useLauncherLog(), { wrapper });

    const line: LogLine = { stream: "stdout", line: "hello", timestamp: 0 };
    emit<LogLine>("launcher:log", line);
    await waitFor(() => expect(result.current).toEqual([line]));

    emit<LauncherStatus>("launcher:status", { state: "launching" });
    await waitFor(() => expect(result.current).toEqual([]));
  });
});
