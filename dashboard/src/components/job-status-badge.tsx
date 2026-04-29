type JobState = "Pending" | "Queued" | "Scheduled" | "Running" | "Done" | "Completed" | "Failed" | "Preempted" | "Stopped";

const jobToneClasses: Record<JobState, string> = {
  Pending: "border-slate-500/20 bg-slate-500/10 text-slate-200",
  Queued: "border-slate-500/20 bg-slate-500/10 text-slate-200",
  Scheduled: "border-cyan-500/20 bg-cyan-500/10 text-cyan-200",
  Running: "border-emerald-500/20 bg-emerald-500/10 text-emerald-200",
  Done: "border-indigo-500/20 bg-indigo-500/10 text-indigo-200",
  Completed: "border-indigo-500/20 bg-indigo-500/10 text-indigo-200",
  Failed: "border-rose-500/20 bg-rose-500/10 text-rose-200",
  Preempted: "border-amber-500/20 bg-amber-500/10 text-amber-200",
  Stopped: "border-rose-500/20 bg-rose-500/10 text-rose-200",
};

export function JobStatusBadge({ state }: { state: JobState }) {
  return (
    <span className={`inline-flex rounded-full border px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.22em] ${jobToneClasses[state]}`}>
      {state}
    </span>
  );
}