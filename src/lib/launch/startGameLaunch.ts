import { invoke } from "@tauri-apps/api/core";
import { playSound } from "@/lib/sound";
import { useLaunchStore } from "./store";
import type { LibraryGame } from "@/hooks/use-library";

// Shared by every tile's onEnterPress (game-tile.tsx, carousel-game-tile.tsx) so the actual
// launch_game invoke lives in exactly one place. Kicks off the fade-to-logo beat immediately (via
// the store, synchronous) and fires the real launch in parallel -- nothing here waits on this
// promise, it reacts to the "launcher:status" push instead (see useLauncherListener.ts). Caught
// here only to avoid an unhandled-rejection warning: a launch failure already reaches the UI as a
// real "error"/"crashed" status push, so there's nothing left to do with the rejection itself.
export function startGameLaunch(game: LibraryGame) {
  useLaunchStore.getState().start(game);
  playSound("launch");
  invoke("launch_game", { gameId: game.id }).catch(() => {});
}
