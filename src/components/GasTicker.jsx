import { Fuel } from "lucide-react";

const CHAINS = [
  { name: "Ethereum", symbol: "ETH", gas: "—", color: "#627EEA" },
  { name: "Arbitrum", symbol: "ARB", gas: "—", color: "#28A0F0" },
  { name: "Optimism", symbol: "OP", gas: "—", color: "#FF0420" },
  { name: "Base", symbol: "BASE", gas: "—", color: "#0052FF" },
  { name: "Polygon", symbol: "MATIC", gas: "—", color: "#8247E5" },
  { name: "zkSync", symbol: "ZK", gas: "—", color: "#8C8DFC" },
];

export default function GasTicker() {
  return (
    <div className="space-y-2">
      {CHAINS.map((chain) => (
        <div
          key={chain.name}
          className="flex items-center justify-between px-3 py-2 bg-surface-raised/50 rounded"
        >
          <div className="flex items-center gap-2">
            <div
              className="w-2 h-2 rounded-full"
              style={{ backgroundColor: chain.color }}
            />
            <span className="text-xs text-text">{chain.name}</span>
          </div>
          <div className="flex items-center gap-1 text-xs text-text-muted">
            <Fuel size={10} />
            <span>{chain.gas} gwei</span>
          </div>
        </div>
      ))}
    </div>
  );
}
