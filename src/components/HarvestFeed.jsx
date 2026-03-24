import { useHuntStore } from "../stores/huntStore";
import ScoreGauge from "./ScoreGauge";
import { Zap, CheckCircle, XCircle } from "lucide-react";

const CHAIN_COLORS = {
  ethereum: "#627EEA",
  arbitrum: "#28A0F0",
  optimism: "#FF0420",
  base: "#0052FF",
  polygon: "#8247E5",
  zksync: "#8C8DFC",
};

export default function HarvestFeed({ onSelect }) {
  const evaluations = useHuntStore((s) => s.evaluations);
  const top = [...evaluations]
    .sort((a, b) => (b.sigma_score ?? 0) - (a.sigma_score ?? 0))
    .slice(0, 10);

  if (top.length === 0) {
    return (
      <div className="py-8 text-center">
        <Zap size={24} className="text-text-dim mx-auto mb-2" />
        <p className="text-text-dim text-xs">No opportunities discovered yet</p>
        <p className="text-text-dim text-[10px] mt-1">
          Start the hunt to begin discovery
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {top.map((opp, i) => (
        <div
          key={opp.id || i}
          onClick={() => onSelect?.(opp)}
          className="flex items-center gap-3 px-3 py-2 bg-surface-raised/50 rounded
                     hover:bg-surface-raised transition-colors cursor-pointer border
                     border-transparent hover:border-border"
        >
          <ScoreGauge
            score={Math.round((opp.harvest_score ?? 0) * 100)}
            size="sm"
          />
          <div className="flex-1 min-w-0">
            <div className="text-xs text-text truncate">{opp.title}</div>
            <div className="text-[10px] text-text-muted flex items-center gap-1">
              <span
                className="w-2 h-2 rounded-full inline-block flex-shrink-0"
                style={{
                  backgroundColor: CHAIN_COLORS[opp.chain] ?? "#4a5568",
                }}
              />
              {opp.chain} · {opp.source}
            </div>
          </div>
          <div className="flex flex-col items-end gap-0.5">
            <span className="text-xs text-primary font-bold font-mono">
              Σ{(opp.sigma_score ?? 0).toFixed(2)}
            </span>
            {opp.proceed ? (
              <CheckCircle size={12} className="text-green-400" />
            ) : (
              <XCircle size={12} className="text-red-400" />
            )}
          </div>
        </div>
      ))}
    </div>
  );
}
