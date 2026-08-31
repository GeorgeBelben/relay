import { create } from "zustand";
import type { Hint } from "./types";

type HintEntry = { id: string; hints: Hint[] };

type HintsStore = {
  // Ordered like a stack: the most recently registered entry (e.g. an open modal) is what's
  // shown, so a modal's hints naturally override its parent screen's while it's open, and the
  // parent's reappear automatically once the modal unmounts.
  entries: HintEntry[];
  push: (entry: HintEntry) => void;
  remove: (id: string) => void;
};

export const useHintsStore = create<HintsStore>((set) => ({
  entries: [],
  push: (entry) => set((state) => ({ entries: [...state.entries, entry] })),
  remove: (id) => set((state) => ({ entries: state.entries.filter((entry) => entry.id !== id) })),
}));

// A fresh `[]` literal in the selector below would be a new reference on every call once
// entries is empty, which useSyncExternalStore (what zustand's hook is built on) treats as
// "the snapshot changed" forever -- an infinite render loop, not just a wasted render.
const EMPTY_HINTS: Hint[] = [];

export function useCurrentHints(): Hint[] {
  return useHintsStore((state) => state.entries.at(-1)?.hints ?? EMPTY_HINTS);
}
