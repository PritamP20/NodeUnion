"use client";

import { useMemo, useState } from "react";
import useSWR from "swr";
import { motion } from "framer-motion";
import { CLIBuilderForm } from "@/components/cli-builder/CLIBuilderForm";
import { CLIPreviewTerminal } from "@/components/cli-builder/CLIPreviewTerminal";
import { CommandReference } from "@/components/cli-builder/CommandReference";
import {
  useCommandBuilder,
  type BuilderTab,
  type RegisterNodeForm,
  type SubmitJobForm,
} from "@/components/cli-builder/useCommandBuilder";
import { REGION_COORDINATES } from "@/lib/region-coordinates";
import { fetchMainSnapshot } from "@/lib/orchestrator-realtime";

type Snapshot = Awaited<ReturnType<typeof fetchMainSnapshot>>;

export function CLIBuilderPageClient() {
  const { data, error, isLoading } = useSWR<Snapshot>("/api/main/snapshot", () => fetchMainSnapshot(), {
    refreshInterval: 30000,
    revalidateOnFocus: true,
  });

  const [tab, setTab] = useState<BuilderTab>("submit");
  const [submitForm, setSubmitForm] = useState<SubmitJobForm>({
    wallet: "",
    network: "",
    image: "ghcr.io/nodeunion/demo:latest",
    command: "python app.py",
    cpu: "2",
    ramGb: "4",
    port: "3000",
    orchestratorUrl: "",
  });
  const [registerForm, setRegisterForm] = useState<RegisterNodeForm>({
    name: "",
    region: "",
    cpu: "8",
    ramGb: "16",
    network: "",
    endpoint: "",
  });

  const command = useCommandBuilder(tab, submitForm, registerForm);

  const networks = useMemo(
    () =>
      (data?.networks ?? []).map((network) => ({
        id: network.network_id,
        name: `${network.name} (${network.network_id})`,
      })),
    [data?.networks],
  );

  const regions = useMemo(() => Object.keys(REGION_COORDINATES).sort(), []);

  return (
    <main className="mx-auto w-full max-w-7xl space-y-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-10">
      <motion.section
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.35, ease: "easeOut" }}
        className="glass-card rounded-[2rem] p-6 sm:p-8"
      >
        <p className="text-[11px] uppercase tracking-[0.32em] text-cyan-300">Tools</p>
        <h1 className="mt-3 text-4xl font-semibold tracking-tight sm:text-5xl">CLI Command Builder</h1>
        <p className="mt-3 max-w-3xl text-sm leading-7 text-slate-300 sm:text-base">
          Build submit and node registration commands visually, then copy exact CLI output.
        </p>
        {error ? (
          <div className="mt-4 rounded-xl border border-red-500/20 bg-red-500/10 px-4 py-3 text-sm text-red-200">
            Failed to load networks. Command builder still works with manual input.
          </div>
        ) : null}
      </motion.section>

      <section className="grid gap-4 lg:grid-cols-2">
        <CLIBuilderForm
          tab={tab}
          onTabChange={setTab}
          submit={submitForm}
          register={registerForm}
          networks={networks}
          regions={regions}
          onSubmitChange={setSubmitForm}
          onRegisterChange={setRegisterForm}
        />
        <CLIPreviewTerminal tab={tab} commandText={command} submitForm={submitForm} />
      </section>

      {isLoading ? (
        <section className="glass-card rounded-[1.75rem] p-6">
          <div className="h-4 w-36 animate-pulse rounded bg-white/10" />
          <div className="mt-3 h-24 animate-pulse rounded bg-white/10" />
        </section>
      ) : (
        <CommandReference />
      )}
    </main>
  );
}
