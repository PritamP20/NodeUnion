"use client";

import { useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";

const REFERENCE = {
  submit: [
    { flag: "--wallet", description: "User wallet address." },
    { flag: "--network", description: "Network ID to route the workload." },
    { flag: "--image", description: "Container image reference." },
    { flag: "--cmd", description: "Command executed inside the container." },
    { flag: "--cpu", description: "CPU cores requested." },
    { flag: "--ram", description: "Memory requested in GB." },
    { flag: "--port", description: "Exposed service port." },
    { flag: "--orchestrator", description: "Optional orchestrator base URL override." },
  ],
  register: [
    { flag: "--name", description: "Provider node name." },
    { flag: "--region", description: "Geographic region identifier." },
    { flag: "--cpu", description: "Node CPU capacity." },
    { flag: "--ram", description: "Node RAM capacity in GB." },
    { flag: "--network", description: "Network to join." },
    { flag: "--endpoint", description: "Agent endpoint URL." },
  ],
};

export function CommandReference() {
  const [open, setOpen] = useState(false);

  return (
    <section className="glass-card rounded-[1.75rem] p-5 sm:p-6">
      <button
        type="button"
        onClick={() => setOpen((value) => !value)}
        className="flex w-full items-center justify-between gap-3 text-left"
      >
        <div>
          <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-300">Command Reference</p>
          <h2 className="mt-2 text-xl font-semibold text-slate-100">CLI flags and meanings</h2>
        </div>
        {open ? <ChevronDown size={18} className="text-slate-300" /> : <ChevronRight size={18} className="text-slate-300" />}
      </button>

      {open ? (
        <div className="mt-5 grid gap-5 lg:grid-cols-2">
          <div className="rounded-xl border border-white/10 bg-[#0a0a0f] p-4">
            <p className="font-mono text-xs uppercase tracking-[0.24em] text-slate-400">submit job</p>
            <div className="mt-3 space-y-2 text-sm text-slate-300">
              {REFERENCE.submit.map((item) => (
                <p key={item.flag}>
                  <span className="font-mono text-cyan-400">{item.flag}</span> - {item.description}
                </p>
              ))}
            </div>
          </div>

          <div className="rounded-xl border border-white/10 bg-[#0a0a0f] p-4">
            <p className="font-mono text-xs uppercase tracking-[0.24em] text-slate-400">register node</p>
            <div className="mt-3 space-y-2 text-sm text-slate-300">
              {REFERENCE.register.map((item) => (
                <p key={item.flag}>
                  <span className="font-mono text-cyan-400">{item.flag}</span> - {item.description}
                </p>
              ))}
            </div>
          </div>
        </div>
      ) : null}
    </section>
  );
}
