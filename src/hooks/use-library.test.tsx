import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useAllGames, useLibrary, useRecentlyAdded, useRecentlyPlayed, type LibraryGame, type LibraryShelf } from "./use-library";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

function wrapper({ children }: { children: ReactNode }) {
  const queryClient = new QueryClient();
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

function makeGame(overrides: Partial<LibraryGame>): LibraryGame {
  return {
    id: "g1",
    title: "Chrono Trigger",
    system_id: "snes",
    system_name: "SNES",
    boxart_path: null,
    beaten: false,
    added_at: 0,
    ...overrides,
  };
}

describe("useLibrary", () => {
  it("returns the shelves from list_library_shelves, empty until loaded", async () => {
    const shelves: LibraryShelf[] = [{ system_id: "snes", system_name: "SNES", games: [makeGame({})] }];
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "list_library_shelves") return Promise.resolve(shelves);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const { result } = renderHook(() => useLibrary(), { wrapper });

    expect(result.current).toEqual([]);
    await waitFor(() => expect(result.current).toEqual(shelves));
  });
});

describe("useAllGames", () => {
  it("returns the games from list_all_games_in_library", async () => {
    const games = [makeGame({ title: "A" }), makeGame({ id: "g2", title: "B" })];
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "list_all_games_in_library") return Promise.resolve(games);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const { result } = renderHook(() => useAllGames(), { wrapper });

    await waitFor(() => expect(result.current).toEqual(games));
  });
});

describe("useRecentlyAdded", () => {
  it("stays undefined while pending, then resolves to the fetched games", async () => {
    const games = [makeGame({})];
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "list_recently_added_games") return Promise.resolve(games);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const { result } = renderHook(() => useRecentlyAdded(), { wrapper });

    expect(result.current).toBeUndefined();
    await waitFor(() => expect(result.current).toEqual(games));
  });
});

describe("useRecentlyPlayed", () => {
  it("is always an empty array (no play-session tracking yet)", () => {
    const { result } = renderHook(() => useRecentlyPlayed(), { wrapper });
    expect(result.current).toEqual([]);
  });
});
