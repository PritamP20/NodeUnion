import { StatusDot } from "@/components/status-dot";
import type { MachineDetails } from "@/components/onboarding/useWizardState";

type Props = {
  machine: MachineDetails;
};

export function NodePreviewCard({ machine }: Props) {
  return (
    <div className="rounded-xl border border-white/10 bg-white/5 p-4">
      <p className="text-xs uppercase tracking-[0.22em] text-slate-500">Node Preview</p>
      <div className="mt-3 flex items-center justify-between gap-3">
        <div>
          <p className="text-sm font-semibold text-slate-100">{machine.nodeName || "your-node-name"}</p>
          <p className="mt-1 text-xs text-slate-400">{machine.region || "region"}</p>
        </div>
        <StatusDot tone="online" />
      </div>
      <div className="mt-3 grid grid-cols-3 gap-2 text-xs text-slate-300">
        <div className="rounded-lg border border-white/10 bg-[#0a0a0f] p-2">
          <p className="text-slate-500">CPU</p>
          <p className="font-mono text-slate-100">{machine.cpuCapacity || "0"}</p>
        </div>
        <div className="rounded-lg border border-white/10 bg-[#0a0a0f] p-2">
          <p className="text-slate-500">RAM</p>
          <p className="font-mono text-slate-100">{machine.ramGb || "0"}GB</p>
        </div>
        <div className="rounded-lg border border-white/10 bg-[#0a0a0f] p-2">
          <p className="text-slate-500">Storage</p>
          <p className="font-mono text-slate-100">{machine.storageGb || "-"}GB</p>
        </div>
      </div>
    </div>
  );
}
