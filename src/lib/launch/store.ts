import { create } from "zustand";
import type { LibraryGame } from "@/hooks/use-library";

// "logo": the fade-to-black-and-logo beat every real console does before a game appears -- shown
// while the launcher is generating config/spawning the emulator, and for at least
// MIN_LOGO_DURATION_MS regardless of how fast that actually resolves (see useLauncherListener.ts)
// so a near-instant launch doesn't read as a jarring flash-cut.
// "playing": the emulator is actually running and in front -- entered once the logo beat's
// minimum duration has elapsed (see useLauncherListener.ts), left only when the real process
// exits. `game` (and the quick-menu's own state, see quickMenuOpen below) stays populated the
// whole time a game is playing, unlike the old bare dismiss()-to-idle this replaced, so the
// in-game quick menu (REL-23) still knows which game is running after the boot-style overlay
// itself has faded out.
// "error": a real launch failure (missing core/binary, spawn error, or a crash) -- surfaces the
// launcher's actual error message.
export type LaunchPhase = "idle" | "logo" | "playing" | "error";

type LaunchState = {
  phase: LaunchPhase;
  game: LibraryGame | null;
  message: string | null;
  // The Home-button quick menu (REL-23) -- only ever meaningful while phase is "playing", but
  // kept as its own flag rather than folded into `phase` so a future pause-while-open concern
  // doesn't need a third dimension crossed with every other phase.
  quickMenuOpen: boolean;
  start: (game: LibraryGame) => void;
  setPhase: (phase: LaunchPhase, message?: string | null) => void;
  enterPlaying: () => void;
  dismiss: () => void;
  openQuickMenu: () => void;
  closeQuickMenu: () => void;
  toggleQuickMenu: () => void;
};

export const useLaunchStore = create<LaunchState>((set) => ({
  phase: "idle",
  game: null,
  message: null,
  quickMenuOpen: false,
  start: (game) => set({ phase: "logo", game, message: null, quickMenuOpen: false }),
  setPhase: (phase, message = null) => set({ phase, message }),
  enterPlaying: () => set({ phase: "playing", message: null }),
  dismiss: () => set({ phase: "idle", game: null, message: null, quickMenuOpen: false }),
  openQuickMenu: () => set({ quickMenuOpen: true }),
  closeQuickMenu: () => set({ quickMenuOpen: false }),
  toggleQuickMenu: () => set((state) => ({ quickMenuOpen: !state.quickMenuOpen })),
}));
