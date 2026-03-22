import { create } from "zustand";

export const useAppStore = create((set) => ({
  activeView: "dashboard",
  commandPaletteOpen: false,

  setActiveView: (view) => set({ activeView: view }),
  setCommandPaletteOpen: (open) => set({ commandPaletteOpen: open }),
}));
