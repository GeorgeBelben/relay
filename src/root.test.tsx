import { describe, expect, it, vi } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { App } from "./root";
import type { Game } from "./hooks/use-games";
import type { Rom } from "./hooks/use-roms";
import type { System } from "./hooks/use-systems";

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
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "list_games") return Promise.resolve(games);
      if (cmd === "list_systems") return Promise.resolve([]);
      if (cmd === "list_roms") return Promise.resolve([]);
      throw new Error(`unexpected invoke: ${cmd}`);
    });

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

  it("supports the full create system -> rom -> game, then edit and delete flow", async () => {
    let systems: System[] = [];
    let roms: Rom[] = [];
    let games: Game[] = [];

    invokeMock.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      switch (cmd) {
        case "list_systems":
          return Promise.resolve(systems);
        case "list_roms":
          return Promise.resolve(roms);
        case "list_games":
          return Promise.resolve(games);
        case "create_system": {
          const system: System = {
            id: args!.id as string,
            name: args!.name as string,
            extensions: args!.extensions as string,
            retroarch_core: (args!.retroarchCore as string) ?? null,
            standalone_binary: (args!.standaloneBinary as string) ?? null,
          };
          systems = [...systems, system];
          return Promise.resolve(system);
        }
        case "create_rom": {
          const rom: Rom = {
            id: "r1",
            system_id: args!.systemId as string,
            path: args!.path as string,
            crc32: null,
            size_bytes: null,
            discs: null,
            status: "ok",
            created_at: 0,
            updated_at: 0,
          };
          roms = [...roms, rom];
          return Promise.resolve(rom);
        }
        case "create_game": {
          const game: Game = {
            id: "g1",
            rom_id: args!.romId as string,
            title: args!.title as string,
            scanned_title: args!.title as string,
            steamgriddb_id: null,
            match_confidence: null,
            enriched_at: null,
            retroachievements_game_id: null,
            retroachievements_matched_at: null,
            ra_highest_award_kind: null,
            created_at: 0,
            updated_at: 0,
          };
          games = [...games, game];
          return Promise.resolve(game);
        }
        case "update_game": {
          games = games.map((g) => (g.id === args!.id ? { ...g, title: args!.title as string } : g));
          return Promise.resolve(games.find((g) => g.id === args!.id));
        }
        case "delete_game": {
          games = games.filter((g) => g.id !== args!.id);
          return Promise.resolve(undefined);
        }
        default:
          throw new Error(`unexpected invoke: ${cmd}`);
      }
    });

    renderApp();

    fireEvent.change(screen.getByPlaceholderText("id (e.g. nes)"), { target: { value: "nes" } });
    fireEvent.change(screen.getByPlaceholderText("name"), { target: { value: "NES" } });
    fireEvent.click(screen.getByText("Add system"));
    await waitFor(() => expect(screen.getByText("nes — NES")).toBeInTheDocument());

    fireEvent.change(screen.getByLabelText("System for rom"), { target: { value: "nes" } });
    fireEvent.change(screen.getByPlaceholderText("path"), { target: { value: "nes/Mario.nes" } });
    fireEvent.click(screen.getByText("Add rom"));
    await waitFor(() => expect(screen.getByText("nes/Mario.nes (nes)")).toBeInTheDocument());

    fireEvent.change(screen.getByLabelText("Rom for game"), { target: { value: "r1" } });
    fireEvent.change(screen.getByPlaceholderText("title"), { target: { value: "Super Mario Bros." } });
    fireEvent.click(screen.getByText("Add game"));
    await waitFor(() => expect(screen.getByText("Super Mario Bros.")).toBeInTheDocument());

    vi.spyOn(window, "prompt").mockReturnValue("Super Mario Bros. (Edited)");
    fireEvent.click(screen.getByText("Edit"));
    await waitFor(() => expect(screen.getByText("Super Mario Bros. (Edited)")).toBeInTheDocument());

    fireEvent.click(screen.getByText("Delete"));
    await waitFor(() => expect(screen.queryByText("Super Mario Bros. (Edited)")).not.toBeInTheDocument());
  });
});
