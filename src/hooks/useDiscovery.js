import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

export function useDiscovery() {
  return useQuery({
    queryKey: ["opportunities"],
    queryFn: () => invoke("get_opportunities"),
    enabled: false, // Manual trigger only for now
    staleTime: 30_000,
  });
}
