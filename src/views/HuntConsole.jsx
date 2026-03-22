import { Crosshair, Play, Pause, Square, Terminal } from "lucide-react";
import { useHuntStore } from "../stores/huntStore";

export default function HuntConsole() {
  const { running, logs } = useHuntStore();

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3 mb-2">
        <Crosshair className="text-warning" size={28} />
        <div>
          <h2 className="text-xl font-bold text-text">Hunt Console</h2>
          <p className="text-text-muted text-xs">
            Control the discovery and claim pipeline
          </p>
        </div>
      </div>
      <div className="h-px bg-gradient-to-r from-warning to-transparent" />

      {/* Controls */}
      <div className="flex items-center gap-3">
        <button className="flex items-center gap-2 px-4 py-2 bg-primary/10 border border-primary/30 rounded text-primary text-sm hover:bg-primary/20 transition-colors">
          <Play size={14} /> Start Hunt
        </button>
        <button className="flex items-center gap-2 px-4 py-2 bg-warning/10 border border-warning/30 rounded text-warning text-sm hover:bg-warning/20 transition-colors">
          <Pause size={14} /> Pause
        </button>
        <button className="flex items-center gap-2 px-4 py-2 bg-danger/10 border border-danger/30 rounded text-danger text-sm hover:bg-danger/20 transition-colors">
          <Square size={14} /> Stop
        </button>
        <div className="ml-auto flex items-center gap-2 text-xs">
          <div
            className={`w-2 h-2 rounded-full ${running ? "bg-primary animate-pulse" : "bg-text-dim"}`}
          />
          <span className="text-text-muted">
            {running ? "Pipeline Running" : "Pipeline Idle"}
          </span>
        </div>
      </div>

      {/* Log Stream */}
      <div className="bg-surface rounded-lg border border-border">
        <div className="flex items-center gap-2 px-4 py-2 border-b border-border">
          <Terminal size={14} className="text-text-muted" />
          <span className="text-xs text-text-muted uppercase tracking-wider">
            Agent Log Stream
          </span>
        </div>
        <div className="h-96 overflow-y-auto p-4 font-mono text-xs space-y-1">
          {logs.length === 0 ? (
            <div className="text-text-dim italic">
              No activity yet. Start a hunt to begin.
            </div>
          ) : (
            logs.map((log, i) => (
              <div
                key={i}
                className={`${
                  log.level === "error"
                    ? "text-danger"
                    : log.level === "warn"
                      ? "text-warning"
                      : log.level === "success"
                        ? "text-primary"
                        : "text-text-muted"
                }`}
              >
                <span className="text-text-dim">[{log.timestamp}]</span>{" "}
                {log.message}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
