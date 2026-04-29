import { Loader2 } from "lucide-react";
import type { OrchestratorNetwork } from "@/lib/orchestrator-realtime";

type Props = {
  networks: OrchestratorNetwork[];
  selectedNetworkId: string;
  isLoading: boolean;
  error?: string;
  onSelect: (networkId: string) => void;
};

export function StepNetworkSelect({ networks, selectedNetworkId, isLoading, error, onSelect }: Props) {
  if (isLoading) {
    return (
      <div className="flex min-h-56 items-center justify-center gap-3 text-sm text-slate-400">
        <Loader2 size={16} className="animate-spin" /> Loading available networks...
      </div>
    );
  }

  if (error) {
    return <div className="rounded-xl border border-red-500/20 bg-red-500/10 p-4 text-sm text-red-200">{error}</div>;
  }

  const selected = networks.find((network) => network.network_id === selectedNetworkId);

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-2">
        {networks.map((network) => {
          const active = selectedNetworkId === network.network_id;

          return (
            <button
              type="button"
              key={network.network_id}
              onClick={() => onSelect(network.network_id)}
              className={`rounded-xl border p-4 text-left transition ${
                active ? "border-indigo-500/50 bg-indigo-500/20" : "border-white/10 bg-white/5 hover:bg-white/10"
              }`}
            >
              <p className="text-sm font-semibold text-slate-100">{network.name}</p>
              <p className="mt-1 text-xs text-slate-400 font-mono">{network.network_id}</p>
              <p className="mt-3 text-sm text-slate-300">{network.description || "NodeUnion compute network"}</p>
            </button>
          );
        })}
      </div>

      {selected ? (
        <div className="rounded-xl border border-white/10 bg-[#0a0a0f] p-4 text-sm text-slate-300">
          <p className="text-xs uppercase tracking-[0.22em] text-cyan-300">Selected Network</p>
          <p className="mt-2 text-slate-100 font-semibold">{selected.name}</p>
          <p className="mt-1 text-slate-400">{selected.network_id}</p>
        </div>
      ) : null}
    </div>
  );
}
