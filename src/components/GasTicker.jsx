import { useEffect } from "react";
import { Fuel, RefreshCw } from "lucide-react";
import { useChainStore } from "../stores/chainStore";

const CHAIN_COLORS = {
  ethereum: "#627EEA",
  arbitrum: "#28A0F0",
  optimism: "#FF0420",
  base: "#0052FF",
  polygon: "#8247E5",
  zksync: "#8C8DFC",
};

const CHAIN_LABELS = {
  ethereum: "Ethereum",
  arbitrum: "Arbitrum",
  optimism: "Optimism",
  base: "Base",
  polygon: "Polygon",
  zksync: "zkSync",
};

function gasColor(gwei, isL2) {
  if (isL2) {
    if (gwei < 0.1) return "text-primary";
    if (gwei < 1) return "text-warning";
    return "text-danger";
  }
  if (gwei < 20) return "text-primary";
  if (gwei < 50) return "text-warning";
  return "text-danger";
}

function formatGwei(gwei) {
  if (gwei === undefined || gwei === null) return "—";
  if (gwei < 0.01) return gwei.toFixed(4);
  if (gwei < 1) return gwei.toFixed(2);
  return gwei.toFixed(1);
}

export default function GasTicker({ autoRefresh = true, intervalMs = 15000 }) {
  const { gasPrices, gasLoading, fetchGasPrices } = useChainStore();

  useEffect(() => {
    fetchGasPrices();
    if (!autoRefresh) return;
    const timer = setInterval(fetchGasPrices, intervalMs);
    return () => clearInterval(timer);
  }, [fetchGasPrices, autoRefresh, intervalMs]);

  const chainOrder = [
    "ethereum",
    "arbitrum",
    "optimism",
    "base",
    "polygon",
    "zksync",
  ];

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between mb-1">
        <span className="text-[10px] text-text-dim uppercase tracking-wider">
          Gas Prices
        </span>
        <button
          onClick={fetchGasPrices}
          disabled={gasLoading}
          className="text-text-dim hover:text-primary transition-colors disabled:opacity-30"
        >
          <RefreshCw size={10} className={gasLoading ? "animate-spin" : ""} />
        </button>
      </div>
      {chainOrder.map((chainKey) => {
        const gas = gasPrices.find(
          (g) => g.chain.toLowerCase() === chainKey,
        );
        const isL2 = chainKey !== "ethereum";
        const gwei = gas?.total_gwei;

        return (
          <div
            key={chainKey}
            className="flex items-center justify-between px-3 py-2 bg-surface-raised/50 rounded"
          >
            <div className="flex items-center gap-2">
              <div
                className="w-2 h-2 rounded-full"
                style={{ backgroundColor: CHAIN_COLORS[chainKey] }}
              />
              <span className="text-xs text-text">
                {CHAIN_LABELS[chainKey]}
              </span>
            </div>
            <div
              className={`flex items-center gap-1 text-xs ${gwei != null ? gasColor(gwei, isL2) : "text-text-muted"}`}
            >
              <Fuel size={10} />
              <span>{formatGwei(gwei)} gwei</span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
