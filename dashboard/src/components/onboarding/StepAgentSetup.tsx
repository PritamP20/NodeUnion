"use client";

import { Loader2 } from "lucide-react";
import { CopyButton } from "@/components/copy-button";
import { TerminalBlock } from "@/components/terminal-block";

type VerifyStatus = "idle" | "checking" | "success" | "failed";

type Props = {
  installCommand: string;
  runCommand: string;
  endpointUrl: string;
  verifyStatus: VerifyStatus;
  onEndpointChange: (value: string) => void;
  onVerify: () => void;
};

export function StepAgentSetup({
  installCommand,
  runCommand,
  endpointUrl,
  verifyStatus,
  onEndpointChange,
  onVerify,
}: Props) {
  return (
    <div className="space-y-4">
      <div className="space-y-3">
        <TerminalBlock
          title="Install agent"
          lines={[{ kind: "input", text: installCommand }]}
          footer={<CopyButton value={installCommand} />}
        />
        <TerminalBlock
          title="Run agent"
          lines={[{ kind: "input", text: runCommand }]}
          footer={<CopyButton value={runCommand} />}
        />
      </div>

      <div className="rounded-xl border border-white/10 bg-white/5 p-4">
        <label className="block text-sm text-slate-300">Agent endpoint URL
          <input value={endpointUrl} onChange={(event) => onEndpointChange(event.target.value)} placeholder="http://127.0.0.1:8090" className="mt-2 w-full rounded-xl px-3 py-2" />
        </label>

        <div className="mt-3 flex items-center gap-3">
          <button type="button" onClick={onVerify} disabled={verifyStatus === "checking"} className="rounded-full bg-indigo-500 px-4 py-2 text-sm font-semibold text-white disabled:opacity-60">
            {verifyStatus === "checking" ? "Verifying..." : "Verify Connection"}
          </button>
          {verifyStatus === "checking" ? <Loader2 size={15} className="animate-spin text-slate-300" /> : null}
        </div>

        {verifyStatus === "success" ? (
          <p className="mt-3 rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-sm text-emerald-200">Node detected!</p>
        ) : null}

        {verifyStatus === "failed" ? (
          <p className="mt-3 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-sm text-red-200">Node not detected yet. Make sure the agent is running.</p>
        ) : null}
      </div>
    </div>
  );
}
