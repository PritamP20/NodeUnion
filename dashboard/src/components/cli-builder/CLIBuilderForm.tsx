"use client";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { BuilderTab, RegisterNodeForm, SubmitJobForm } from "@/components/cli-builder/useCommandBuilder";

type Props = {
  tab: BuilderTab;
  onTabChange: (value: BuilderTab) => void;
  submit: SubmitJobForm;
  register: RegisterNodeForm;
  networks: Array<{ id: string; name: string }>;
  regions: string[];
  onSubmitChange: (next: SubmitJobForm) => void;
  onRegisterChange: (next: RegisterNodeForm) => void;
};

export function CLIBuilderForm({
  tab,
  onTabChange,
  submit,
  register,
  networks,
  regions,
  onSubmitChange,
  onRegisterChange,
}: Props) {
  return (
    <section className="glass-card rounded-[1.75rem] p-5 sm:p-6">
      <div className="flex items-center justify-between gap-3">
        <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-300">CLI Builder</p>
      </div>

      <Tabs value={tab} onValueChange={(value) => onTabChange(value as BuilderTab)} className="mt-4">
        <TabsList className="inline-flex rounded-full border border-white/10 bg-white/5 p-1">
          <TabsTrigger
            value="submit"
            className="rounded-full border px-3 py-1 text-xs font-medium transition data-[state=active]:border-indigo-500/50 data-[state=active]:bg-indigo-500/20 data-[state=active]:text-indigo-200 data-[state=inactive]:border-white/10 data-[state=inactive]:bg-white/5 data-[state=inactive]:text-slate-400"
          >
            Submit Job
          </TabsTrigger>
          <TabsTrigger
            value="register"
            className="rounded-full border px-3 py-1 text-xs font-medium transition data-[state=active]:border-indigo-500/50 data-[state=active]:bg-indigo-500/20 data-[state=active]:text-indigo-200 data-[state=inactive]:border-white/10 data-[state=inactive]:bg-white/5 data-[state=inactive]:text-slate-400"
          >
            Register Node
          </TabsTrigger>
        </TabsList>

        <TabsContent value="submit" className="mt-4 space-y-3">
          <label className="block text-sm text-slate-300">Wallet address
            <input value={submit.wallet} onChange={(event) => onSubmitChange({ ...submit, wallet: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" placeholder="wallet" />
          </label>

          <label className="block text-sm text-slate-300">Network selector
            <select value={submit.network} onChange={(event) => onSubmitChange({ ...submit, network: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 text-sm">
              <option value="">Select network</option>
              {networks.map((network) => (
                <option key={network.id} value={network.id}>{network.name}</option>
              ))}
            </select>
          </label>

          <label className="block text-sm text-slate-300">Container image
            <input value={submit.image} onChange={(event) => onSubmitChange({ ...submit, image: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" placeholder="ghcr.io/org/image:tag" />
          </label>

          <label className="block text-sm text-slate-300">Command
            <input value={submit.command} onChange={(event) => onSubmitChange({ ...submit, command: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" placeholder="python app.py" />
          </label>

          <div className="grid gap-3 sm:grid-cols-3">
            <label className="block text-sm text-slate-300">CPU cores
              <input type="number" min="0.5" step="0.5" value={submit.cpu} onChange={(event) => onSubmitChange({ ...submit, cpu: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" />
            </label>
            <label className="block text-sm text-slate-300">RAM in GB
              <input type="number" min="1" step="1" value={submit.ramGb} onChange={(event) => onSubmitChange({ ...submit, ramGb: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" />
            </label>
            <label className="block text-sm text-slate-300">Exposed port
              <input type="number" min="1" max="65535" value={submit.port} onChange={(event) => onSubmitChange({ ...submit, port: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" />
            </label>
          </div>

          <label className="block text-sm text-slate-300">Orchestrator URL (optional)
            <input value={submit.orchestratorUrl} onChange={(event) => onSubmitChange({ ...submit, orchestratorUrl: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" placeholder="https://orchestrator.nodeunion.dev" />
          </label>
        </TabsContent>

        <TabsContent value="register" className="mt-4 space-y-3">
          <label className="block text-sm text-slate-300">Node name
            <input value={register.name} onChange={(event) => onRegisterChange({ ...register, name: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" placeholder="provider-node-1" />
          </label>

          <label className="block text-sm text-slate-300">Region
            <select value={register.region} onChange={(event) => onRegisterChange({ ...register, region: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 text-sm">
              <option value="">Select region</option>
              {regions.map((region) => (
                <option key={region} value={region}>{region}</option>
              ))}
            </select>
          </label>

          <div className="grid gap-3 sm:grid-cols-2">
            <label className="block text-sm text-slate-300">CPU capacity
              <input type="number" min="1" step="0.5" value={register.cpu} onChange={(event) => onRegisterChange({ ...register, cpu: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" />
            </label>
            <label className="block text-sm text-slate-300">RAM capacity (GB)
              <input type="number" min="1" step="1" value={register.ramGb} onChange={(event) => onRegisterChange({ ...register, ramGb: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" />
            </label>
          </div>

          <label className="block text-sm text-slate-300">Network to join
            <select value={register.network} onChange={(event) => onRegisterChange({ ...register, network: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 text-sm">
              <option value="">Select network</option>
              {networks.map((network) => (
                <option key={network.id} value={network.id}>{network.name}</option>
              ))}
            </select>
          </label>

          <label className="block text-sm text-slate-300">Agent endpoint URL
            <input value={register.endpoint} onChange={(event) => onRegisterChange({ ...register, endpoint: event.target.value })} className="mt-2 w-full rounded-xl px-3 py-2 font-mono text-sm" placeholder="http://127.0.0.1:8090" />
          </label>
        </TabsContent>
      </Tabs>
    </section>
  );
}
