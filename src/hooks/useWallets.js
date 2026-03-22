import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

export function useWallets() {
  return useQuery({
    queryKey: ["wallets"],
    queryFn: () => invoke("list_wallets"),
    enabled: false, // Enable after vault unlock
    staleTime: 60_000,
  });
}

export function useVaultStatus() {
  return useQuery({
    queryKey: ["vault_status"],
    queryFn: () => invoke("get_vault_status"),
    staleTime: 5_000,
  });
}
