"use client";

import Link from "next/link";
import { TerminalBlock } from "@/components/terminal-block";
import type { OrchestratorNetwork } from "@/lib/orchestrator-realtime";
import type { WizardFormState } from "@/components/onboarding/useWizardState";

type Props = {
  form: WizardFormState;
  selectedNetwork?: OrchestratorNetwork;
  command: string;
};

export function StepReviewFinish({ form, selectedNetwork, command }: Props) {
  return (
    <div className="space-y-4">
      <div className="rounded-xl border border-white/10 bg-white/5 p-4 text-sm text-slate-300">
        <p className="text-xs uppercase tracking-[0.22em] text-cyan-300">Summary</p>
        <div className="mt-3 grid gap-2 sm:grid-cols-2">
          <p><span className="text-slate-500">Node:</span> <span className="text-slate-100">{form.machine.nodeName}</span></p>
          <p><span className="text-slate-500">Region:</span> <span className="text-slate-100">{form.machine.region}</span></p>
          <p><span className="text-slate-500">CPU:</span> <span className="text-slate-100">{form.machine.cpuCapacity}</span></p>
          <p><span className="text-slate-500">RAM:</span> <span className="text-slate-100">{form.machine.ramGb} GB</span></p>
          <p><span className="text-slate-500">Network:</span> <span className="text-slate-100">{selectedNetwork?.name || form.selectedNetworkId}</span></p>
          <p><span className="text-slate-500">Agent URL:</span> <span className="font-mono text-slate-100">{form.agent.endpointUrl || "not provided"}</span></p>
        </div>
      </div>

      <TerminalBlock title="Your Generated Command" lines={[{ kind: "input", text: command }]} />

      <div className="flex flex-wrap gap-3">
        <Link href="/networks?view=map" className="rounded-full bg-indigo-500 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-400">
          View My Node on Network Map
        </Link>
        <Link href="/provider" className="rounded-full border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-200 hover:bg-white/10">
          Submit a Job
        </Link>
      </div>
    </div>
  );
}
