import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useAchievements, useApplyMatch, useSearchAlternateMatches, type AlternateMatch, type GameAchievementsProgress } from "./use-game-actions";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

function wrapper({ children }: { children: ReactNode }) {
  const queryClient = new QueryClient();
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

describe("useSearchAlternateMatches", () => {
  it("invokes search_alternate_matches with the game id and returns the candidates", async () => {
    const candidates: AlternateMatch[] = [{ steamgriddb_id: 99, title: "Chrono Trigger", boxart_url: "https://example.com/a.png" }];
    invokeMock.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "search_alternate_matches" && args?.gameId === "g1") return Promise.resolve(candidates);
      throw new Error(`unexpected invoke: ${cmd} ${JSON.stringify(args)}`);
    });

    const { result } = renderHook(() => useSearchAlternateMatches(), { wrapper });

    await act(async () => {
      const matches = await result.current.mutateAsync("g1");
      expect(matches).toEqual(candidates);
    });
  });
});

describe("useApplyMatch", () => {
  it("invokes apply_match with the expected args and invalidates games/library on success", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "apply_match") return Promise.resolve(undefined);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const localWrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => useApplyMatch(), { wrapper: localWrapper });

    await act(async () => {
      await result.current.mutateAsync({ gameId: "g1", steamgriddbId: 99, title: "Chrono Trigger" });
    });

    expect(invokeMock).toHaveBeenCalledWith("apply_match", { gameId: "g1", steamgriddbId: 99, title: "Chrono Trigger" });
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["games"] });
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["library"] });
    });
  });
});

describe("useAchievements", () => {
  it("returns null when the game isn't RA-matched (or no profile is linked)", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_achievements") return Promise.resolve(null);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const { result } = renderHook(() => useAchievements("g1"), { wrapper });

    await waitFor(() => expect(result.current.data).toBeNull());
  });

  it("returns the achievement progress from get_achievements", async () => {
    const progress: GameAchievementsProgress = {
      game_id: 99,
      title: "Chrono Trigger",
      console_name: "SNES",
      num_achievements: 1,
      num_awarded_to_user: 1,
      user_completion: "100.00%",
      highest_award_kind: "mastered",
      achievements: [{ id: 1, title: "Time's Up", description: "Beat the game", points: 10, badge_url: "https://i.retroachievements.org/Badge/12345.png", unlocked: true }],
    };
    invokeMock.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "get_achievements" && args?.gameId === "g1") return Promise.resolve(progress);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

    const { result } = renderHook(() => useAchievements("g1"), { wrapper });

    await waitFor(() => expect(result.current.data).toEqual(progress));
  });
});
