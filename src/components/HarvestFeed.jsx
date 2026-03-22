import { Zap } from "lucide-react";
import ScoreGauge from "./ScoreGauge";

export default function HarvestFeed() {
  // Placeholder — will be populated by discovery engine
  const opportunities = [];

  if (opportunities.length === 0) {
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
      {opportunities.map((opp) => (
        <div
          key={opp.id}
          className="flex items-center gap-3 px-3 py-2 bg-surface-raised/50 rounded hover:bg-surface-raised transition-colors cursor-pointer"
        >
          <ScoreGauge score={opp.harvest_score} size="sm" />
          <div className="flex-1 min-w-0">
            <div className="text-xs text-text truncate">{opp.title}</div>
            <div className="text-[10px] text-text-muted">
              {opp.chain} • {opp.type}
            </div>
          </div>
          <div className="text-xs text-primary font-bold">
            ${opp.estimated_value_usd?.toFixed(2) || "?"}
          </div>
        </div>
      ))}
    </div>
  );
}
