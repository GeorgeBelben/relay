import { create } from "zustand";

type SystemMenuState = {
  open: boolean;
  openMenu: () => void;
  closeMenu: () => void;
  toggleMenu: () => void;
};

export const useSystemMenuStore = create<SystemMenuState>((set) => ({
  open: false,
  openMenu: () => set({ open: true }),
  closeMenu: () => set({ open: false }),
  toggleMenu: () => set((state) => ({ open: !state.open })),
}));
