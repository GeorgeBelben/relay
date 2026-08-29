import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import { App } from "./root";
import type { Game } from "./hooks/use-games";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

function renderApp() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>,
  );
}

describe("App", () => {
  it("shows a loading state, then renders the fetched games as a list", async () => {
    const games: Game[] = [
      {
        id: "g1",
        rom_id: "r1",
        title: "Chrono Trigger",
        scanned_title: "Chrono Trigger",
        steamgriddb_id: null,
        match_confidence: null,
        enriched_at: null,
        retroachievements_game_id: null,
        retroachievements_matched_at: null,
        ra_highest_award_kind: null,
        created_at: 0,
        updated_at: 0,
      },
    ];
    invokeMock.mockResolvedValue(games);

    renderApp();

    expect(screen.getByText("Loading games...")).toBeInTheDocument();

    await waitFor(() => expect(screen.getByText("Chrono Trigger")).toBeInTheDocument());
    expect(invokeMock).toHaveBeenCalledWith("list_games");
  });

  it("renders an error message when the command fails", async () => {
    invokeMock.mockRejectedValue(new Error("db unavailable"));

    renderApp();

    await waitFor(() => expect(screen.getByText(/Error:/)).toBeInTheDocument());
  });
});
