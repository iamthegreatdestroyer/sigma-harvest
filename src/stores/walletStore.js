import { create } from "zustand";

export const useWalletStore = create((set) => ({
  vaultLocked: true,
  wallets: [],
  selectedWallet: null,

  setVaultLocked: (locked) => set({ vaultLocked: locked }),
  setWallets: (wallets) => set({ wallets }),
  setSelectedWallet: (wallet) => set({ selectedWallet: wallet }),
  addWallet: (wallet) => set((state) => ({ wallets: [...state.wallets, wallet] })),
}));
