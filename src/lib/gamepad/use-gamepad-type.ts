import { create } from "zustand";
import { ControllerType } from "./types";

type GamepadTypeStore = {
  lastGamepadType: ControllerType | null;
  setLastGamepadType: (type: ControllerType) => void;
};

export const useGamepadTypeStore = create<GamepadTypeStore>((set) => ({
  lastGamepadType: null,
  setLastGamepadType: (type) => set(() => ({ lastGamepadType: type })),
}));

export function useGamepadType(): ControllerType | null {
  return useGamepadTypeStore((state) => state.lastGamepadType);
}
