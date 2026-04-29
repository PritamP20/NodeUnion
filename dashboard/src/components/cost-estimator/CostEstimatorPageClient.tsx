"use client";

import { useMemo, useState } from "react";
import { motion } from "framer-motion";
import { EstimatorInputs } from "@/components/cost-estimator/EstimatorInputs";
import { EstimatorResults } from "@/components/cost-estimator/EstimatorResults";
import { PresetCards } from "@/components/cost-estimator/PresetCards";
import { estimateCost, type EstimatorInput } from "@/lib/pricing";

export function CostEstimatorPageClient() {
  const [state, setState] = useState<EstimatorInput>({
    cpuCores: 2,
    ramGb: 8,
    hours: 2,
    minutes: 0,
    jobCount: 1,
    includeBaseFee: true,
  });

  const estimate = useMemo(() => estimateCost(state), [state]);

  return (
    <main className="mx-auto w-full max-w-7xl space-y-6 px-4 py-6 sm:px-6 lg:px-8 lg:py-10">
      <motion.section
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.35, ease: "easeOut" }}
        className="glass-card rounded-[2rem] p-6 sm:p-8"
      >
        <p className="text-[11px] uppercase tracking-[0.32em] text-cyan-300">Tools</p>
        <h1 className="mt-3 text-4xl font-semibold tracking-tight sm:text-5xl">Cost Estimator</h1>
        <p className="mt-3 max-w-3xl text-sm leading-7 text-slate-300 sm:text-base">Estimate job cost in SOL and USD before submitting workloads.</p>
      </motion.section>

      <section className="grid gap-4 lg:grid-cols-2">
        <EstimatorInputs value={state} onChange={setState} />
        <EstimatorResults
          cpuCost={estimate.cpuCost}
          ramCost={estimate.ramCost}
          baseFee={estimate.baseFee}
          totalSol={estimate.totalSol}
          totalUsd={estimate.totalUsd}
          hoursForOneSol={estimate.hoursForOneSol}
        />
      </section>

      <PresetCards onSelect={setState} />
    </main>
  );
}
