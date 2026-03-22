import {
  Search,
  ExternalLink,
  AlertTriangle,
  CheckCircle,
  Shield,
} from "lucide-react";
import ScoreGauge from "../components/ScoreGauge";

export default function OpportunityInspector() {
  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3 mb-2">
        <Search className="text-primary" size={28} />
        <div>
          <h2 className="text-xl font-bold text-text">Opportunity Inspector</h2>
          <p className="text-text-muted text-xs">
            Deep-dive analysis of discovered opportunities
          </p>
        </div>
      </div>
      <div className="h-px bg-gradient-to-r from-primary to-transparent" />

      {/* Search */}
      <div className="flex items-center gap-2">
        <div className="flex-1 relative">
          <Search
            size={14}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted"
          />
          <input
            type="text"
            placeholder="Search opportunities or paste contract address..."
            className="w-full pl-9 pr-4 py-2 bg-surface border border-border rounded text-sm text-text focus:outline-none focus:border-primary"
          />
        </div>
      </div>

      {/* Empty State */}
      <div className="bg-surface rounded-lg border border-border p-12 text-center">
        <Search size={40} className="text-text-dim mx-auto mb-4" />
        <p className="text-text-muted text-sm mb-2">
          Select an opportunity from the feed or search above
        </p>
        <p className="text-text-dim text-xs">
          The inspector shows contract verification, risk breakdown, and claim
          details
        </p>
      </div>
    </div>
  );
}
