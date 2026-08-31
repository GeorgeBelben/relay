import { create } from "zustand";
import type { LibraryGame } from "@/hooks/use-library";

// "logo": the fade-to-black-and-logo beat every real console does before a game appears -- shown
// while the launcher is generating config/spawning the emulator, and for at least
// MIN_LOGO_DURATION_MS regardless of how fast that actually resolves (see useLauncherListener.ts)
// so a near-instant launch doesn't read as a jarring flash-cut.
// "error": a real launch failure (missing core/binary, spawn error, or a crash) -- surfaces the
// launcher's actual error message.
export type LaunchPhase = "idle" | "logo" | "error";

type LaunchState = {
  phase: LaunchPhase;
  game: LibraryGame | null;
  message: string | null;
  start: (game: LibraryGame) => void;
  setPhase: (phase: LaunchPhase, message?: string | null) => void;
  dismiss: () => void;
};

export const useLaunchStore = create<LaunchState>((set) => ({
  phase: "idle",
  game: null,
  message: null,
  start: (game) => set({ phase: "logo", game, message: null }),
  setPhase: (phase, message = null) => set({ phase, message }),
  dismiss: () => set({ phase: "idle", game: null, message: null }),
}));
