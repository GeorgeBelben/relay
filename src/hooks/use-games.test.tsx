import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useCreateGame, useGames, type Game } from "./use-games";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

function makeGame(overrides: Partial<Game>): Game {
  return {
    id: "g1",
    rom_id: "r1",
    title: "New Game",
    scanned_title: "New Game",
    steamgriddb_id: null,
    match_confidence: null,
    enriched_at: null,
    retroachievements_game_id: null,
    retroachievements_matched_at: null,
    ra_highest_award_kind: null,
    created_at: 0,
    updated_at: 0,
    ...overrides,
  };
}

describe("useGames cache invalidation", () => {
  it("refetches the games list after a mutation invalidates it", async () => {
    const before: Game[] = [];
    const after: Game[] = [makeGame({})];

    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "list_games") {
        return Promise.resolve(
          invokeMock.mock.calls.filter(([c]) => c === "list_games").length <= 1 ? before : after,
        );
      }
      if (cmd === "create_game") {
        return Promise.resolve(after[0]);
      }
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => ({ games: useGames(), create: useCreateGame() }), { wrapper });

    await waitFor(() => expect(result.current.games.data).toEqual(before));

    await act(async () => {
      await result.current.create.mutateAsync({ romId: "r1", title: "New Game" });
    });

    // The mutation's onSuccess invalidates the ["games"] query key, so the list
    // hook should refetch and pick up the new row -- not just cache the mutation's
    // own response.
    await waitFor(() => expect(result.current.games.data).toEqual(after));

    const listCalls = invokeMock.mock.calls.filter(([cmd]) => cmd === "list_games");
    expect(listCalls.length).toBeGreaterThanOrEqual(2);
  });
});
