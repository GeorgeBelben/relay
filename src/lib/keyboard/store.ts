import { create } from "zustand";

export type OskTarget = HTMLInputElement | HTMLTextAreaElement;

type OskStore = {
  // The single real DOM element currently receiving on-screen-keyboard input, or null when the
  // keyboard should be hidden. Driven entirely by document-level focusin/focusout on
  // [data-osk] elements (see on-screen-keyboard.tsx) -- nothing calls setTarget directly from
  // application code, so showing/hiding the keyboard is just "does a tagged input have real DOM
  // focus right now", matching how a physical keyboard would behave.
  target: OskTarget | null;
  setTarget: (target: OskTarget | null) => void;
};

export const useOskStore = create<OskStore>((set) => ({
  target: null,
  setTarget: (target) => set({ target }),
}));
