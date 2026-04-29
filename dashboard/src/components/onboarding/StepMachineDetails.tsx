import { NodePreviewCard } from "@/components/onboarding/NodePreviewCard";
import type { MachineDetails } from "@/components/onboarding/useWizardState";

type Props = {
  machine: MachineDetails;
  regions: string[];
  onChange: (next: MachineDetails) => void;
};

export function StepMachineDetails({ machine, regions, onChange }: Props) {
  return (
    <div className="grid gap-4 lg:grid-cols-[1fr_0.95fr]">
      <div className="space-y-3">
        <label className="block text-sm text-slate-300">Node name *
          <input value={machine.nodeName} onChange={(event) => onChange({ ...machine, nodeName: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2" placeholder="provider-node-1" />
        </label>

        <label className="block text-sm text-slate-300">Region *
          <select value={machine.region} onChange={(event) => onChange({ ...machine, region: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2">
            <option value="">Select region</option>
            {regions.map((region) => (
              <option key={region} value={region}>{region}</option>
            ))}
          </select>
        </label>

        <div className="grid gap-3 sm:grid-cols-2">
          <label className="block text-sm text-slate-300">CPU capacity *
            <input type="number" min="1" step="0.5" value={machine.cpuCapacity} onChange={(event) => onChange({ ...machine, cpuCapacity: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2" />
          </label>

          <label className="block text-sm text-slate-300">RAM in GB *
            <input type="number" min="1" step="1" value={machine.ramGb} onChange={(event) => onChange({ ...machine, ramGb: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2" />
          </label>
        </div>

        <label className="block text-sm text-slate-300">Storage in GB (optional)
          <input type="number" min="1" step="1" value={machine.storageGb} onChange={(event) => onChange({ ...machine, storageGb: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2" />
        </label>
      </div>

      <NodePreviewCard machine={machine} />
    </div>
  );
}
