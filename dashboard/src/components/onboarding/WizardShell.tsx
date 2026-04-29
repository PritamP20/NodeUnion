"use client";

import { AnimatePresence, motion } from "framer-motion";
import { Check } from "lucide-react";
import type { ReactNode } from "react";

type Props = {
  step: number;
  direction: number;
  title: string;
  children: ReactNode;
  canGoBack: boolean;
  canGoNext: boolean;
  nextLabel: string;
  onBack: () => void;
  onNext: () => void;
  hideNext?: boolean;
};

const STEPS = ["Welcome", "Machine", "Network", "Agent", "Review"];

export function WizardShell({
  step,
  direction,
  title,
  children,
  canGoBack,
  canGoNext,
  nextLabel,
  onBack,
  onNext,
  hideNext = false,
}: Props) {
  return (
    <section className="mx-auto w-full max-w-4xl">
      <div className="glass-card rounded-[2rem] p-6 sm:p-8">
        <div className="mb-6">
          <div className="flex items-center justify-between gap-2 overflow-x-auto pb-2">
            {STEPS.map((label, index) => {
              const completed = index < step;
              const active = index === step;

              return (
                <div key={label} className="flex min-w-fit items-center gap-2">
                  <span
                    className={`inline-flex h-7 w-7 items-center justify-center rounded-full border text-xs ${
                      completed
                        ? "border-emerald-400/50 bg-emerald-500/20 text-emerald-200"
                        : active
                          ? "border-indigo-400/60 bg-indigo-500/20 text-indigo-200"
                          : "border-white/10 bg-white/5 text-slate-500"
                    }`}
                  >
                    {completed ? <Check size={13} /> : index + 1}
                  </span>
                  <span className={`text-xs uppercase tracking-[0.18em] ${active ? "text-slate-200" : "text-slate-500"}`}>{label}</span>
                  {index < STEPS.length - 1 ? <span className="mx-1 h-px w-8 bg-white/10" /> : null}
                </div>
              );
            })}
          </div>
        </div>

        <h1 className="text-3xl font-semibold tracking-tight text-slate-100 sm:text-4xl">{title}</h1>

        <div className="mt-6 min-h-[380px]">
          <AnimatePresence initial={false} mode="wait" custom={direction}>
            <motion.div
              key={step}
              custom={direction}
              initial={{ opacity: 0, x: direction > 0 ? 40 : -40 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: direction > 0 ? -40 : 40 }}
              transition={{ duration: 0.25, ease: "easeOut" }}
            >
              {children}
            </motion.div>
          </AnimatePresence>
        </div>

        <div className="mt-6 flex items-center justify-between gap-3 border-t border-white/10 pt-4">
          <button
            type="button"
            onClick={onBack}
            disabled={!canGoBack}
            className="rounded-full border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-200 hover:bg-white/10 disabled:opacity-40"
          >
            Back
          </button>

          {!hideNext ? (
            <button
              type="button"
              onClick={onNext}
              disabled={!canGoNext}
              className="rounded-full bg-indigo-500 px-5 py-2 text-sm font-semibold text-white hover:bg-indigo-400 disabled:opacity-50"
            >
              {nextLabel}
            </button>
          ) : <span />}
        </div>
      </div>
    </section>
  );
}
