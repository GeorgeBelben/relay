import { create } from "zustand";
import type { InputMethod } from "./types";

type InputMethodStore = {
  lastInputMethod: InputMethod | null;
  setLastInputMethod: (method: InputMethod) => void;
};

export const useInputMethodStore = create<InputMethodStore>((set) => ({
  lastInputMethod: null,
  setLastInputMethod: (method) => set({ lastInputMethod: method }),
}));
