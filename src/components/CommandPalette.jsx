import { Command } from "cmdk";
import { useAppStore } from "../stores/appStore";
import { useHuntStore } from "../stores/huntStore";
import { useWalletStore } from "../stores/walletStore";
import {
  LayoutDashboard,
  Crosshair,
  Wallet,
  Search,
  BarChart3,
  Play,
  Lock,
  ArrowRightLeft,
  Settings,
} from "lucide-react";

export default function CommandPalette() {
  const { commandPaletteOpen, setCommandPaletteOpen, setActiveView } =
    useAppStore();

  if (!commandPaletteOpen) return null;

  const runAction = (action) => {
    action();
    setCommandPaletteOpen(false);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]">
      <div
        className="fixed inset-0 bg-black/60"
        onClick={() => setCommandPaletteOpen(false)}
      />
      <Command
        className="relative w-[560px] bg-surface border border-border rounded-xl shadow-2xl overflow-hidden"
        label="Command palette"
      >
        <Command.Input
          placeholder="Type a command..."
          className="w-full px-4 py-3 bg-transparent text-text text-sm border-b border-border focus:outline-none placeholder:text-text-dim"
          autoFocus
        />
        <Command.List className="max-h-72 overflow-y-auto p-2">
          <Command.Empty className="p-4 text-text-muted text-sm text-center">
            No results found.
          </Command.Empty>

          <Command.Group
            heading="Navigate"
            className="text-[10px] text-text-dim uppercase tracking-wider px-2 py-1"
          >
            <Command.Item
              onSelect={() => runAction(() => setActiveView("dashboard"))}
              className="flex items-center gap-3 px-3 py-2 rounded text-sm text-text-muted hover:bg-surface-raised hover:text-text cursor-pointer data-[selected=true]:bg-surface-raised data-[selected=true]:text-text"
            >
              <LayoutDashboard size={14} /> Command Center
            </Command.Item>
            <Command.Item
              onSelect={() => runAction(() => setActiveView("hunt"))}
              className="flex items-center gap-3 px-3 py-2 rounded text-sm text-text-muted hover:bg-surface-raised hover:text-text cursor-pointer data-[selected=true]:bg-surface-raised data-[selected=true]:text-text"
            >
              <Crosshair size={14} /> Hunt Console
            </Command.Item>
            <Command.Item
              onSelect={() => runAction(() => setActiveView("wallets"))}
              className="flex items-center gap-3 px-3 py-2 rounded text-sm text-text-muted hover:bg-surface-raised hover:text-text cursor-pointer data-[selected=true]:bg-surface-raised data-[selected=true]:text-text"
            >
              <Wallet size={14} /> Wallet Manager
            </Command.Item>
            <Command.Item
              onSelect={() => runAction(() => setActiveView("inspect"))}
              className="flex items-center gap-3 px-3 py-2 rounded text-sm text-text-muted hover:bg-surface-raised hover:text-text cursor-pointer data-[selected=true]:bg-surface-raised data-[selected=true]:text-text"
            >
              <Search size={14} /> Opportunity Inspector
            </Command.Item>
            <Command.Item
              onSelect={() => runAction(() => setActiveView("analytics"))}
              className="flex items-center gap-3 px-3 py-2 rounded text-sm text-text-muted hover:bg-surface-raised hover:text-text cursor-pointer data-[selected=true]:bg-surface-raised data-[selected=true]:text-text"
            >
              <BarChart3 size={14} /> Analytics Bay
            </Command.Item>
          </Command.Group>

          <Command.Group
            heading="Actions"
            className="text-[10px] text-text-dim uppercase tracking-wider px-2 py-1"
          >
            <Command.Item
              onSelect={() => runAction(() => useHuntStore.getState().runHuntCycle())}
              className="flex items-center gap-3 px-3 py-2 rounded text-sm text-text-muted hover:bg-surface-raised hover:text-text cursor-pointer data-[selected=true]:bg-surface-raised data-[selected=true]:text-text"
            >
              <Play size={14} /> Start Hunt
            </Command.Item>
            <Command.Item
              onSelect={() => runAction(() => useWalletStore.getState().lockVault())}
              className="flex items-center gap-3 px-3 py-2 rounded text-sm text-text-muted hover:bg-surface-raised hover:text-text cursor-pointer data-[selected=true]:bg-surface-raised data-[selected=true]:text-text"
            >
              <Lock size={14} /> Lock Vault
            </Command.Item>
            <Command.Item className="flex items-center gap-3 px-3 py-2 rounded text-sm text-text-muted hover:bg-surface-raised hover:text-text cursor-pointer data-[selected=true]:bg-surface-raised data-[selected=true]:text-text">
              <ArrowRightLeft size={14} /> Consolidate Funds
            </Command.Item>
          </Command.Group>
        </Command.List>
      </Command>
    </div>
  );
}
