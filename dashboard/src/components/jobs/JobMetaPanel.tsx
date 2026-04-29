"use client";

import Link from "next/link";
import { format } from "date-fns";
import { ArrowLeft } from "lucide-react";
import { CopyButton } from "@/components/copy-button";
import { JobStatusBadge } from "@/components/job-status-badge";
import { LiveDurationCounter } from "@/components/jobs/LiveDurationCounter";
import type { OrchestratorJob } from "@/lib/orchestrator-realtime";

function mapStatus(status: OrchestratorJob["status"]) {
  if (status === "Done") return "Completed";
  if (status === "Pending") return "Queued";
  if (status === "Scheduled") return "Running";
  if (status === "Stopped") return "Failed";
  return status;
}

type Props = {
  job: OrchestratorJob;
  networkName: string;
};

export function JobMetaPanel({ job, networkName }: Props) {
  const mappedStatus = mapStatus(job.status) as Parameters<typeof JobStatusBadge>[0]["state"];
  const startedAt = format(new Date(job.created_at_epoch_secs * 1000), "PPpp");
  const running = mappedStatus === "Running" || mappedStatus === "Queued";

  return (
    <aside className="glass-card rounded-2xl p-5 sm:p-6">
      <div className="flex items-center justify-between gap-2">
        <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-300">Job Metadata</p>
        <Link
          href="/provider"
          className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-medium text-slate-200 hover:bg-white/10"
        >
          <ArrowLeft size={14} /> Back to Provider
        </Link>
      </div>

      <div className="mt-5 space-y-4 text-sm text-slate-300">
        <div className="rounded-xl border border-white/10 bg-white/5 p-3">
          <p className="text-xs uppercase tracking-[0.2em] text-slate-500">Job ID</p>
          <div className="mt-2 flex items-center justify-between gap-2">
            <p className="font-mono text-xs text-slate-100 break-all">{job.job_id}</p>
            <CopyButton value={job.job_id} />
          </div>
        </div>

        <div className="flex items-center gap-3">
          <JobStatusBadge state={mappedStatus} />
          {running ? <span className="h-2.5 w-2.5 animate-pulse rounded-full bg-emerald-400" /> : null}
        </div>

        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-1">
          <p><span className="text-slate-500">Image:</span> <span className="text-slate-100">{job.image}</span></p>
          <p><span className="text-slate-500">Assigned Node:</span> <span className="font-mono text-slate-100">{job.assigned_node_id || "unassigned"}</span></p>
          <p><span className="text-slate-500">Network:</span> <span className="text-slate-100">{networkName}</span></p>
          <p><span className="text-slate-500">Started:</span> <span className="text-slate-100">{startedAt}</span></p>
          <p>
            <span className="text-slate-500">Duration:</span>{" "}
            <LiveDurationCounter startedAtEpochSecs={job.created_at_epoch_secs} isRunning={running} />
          </p>
          <p><span className="text-slate-500">CPU Limit:</span> <span className="text-slate-100">{job.cpu_limit} cores</span></p>
          <p><span className="text-slate-500">RAM Limit:</span> <span className="text-slate-100">{Math.round(job.ram_limit_mb / 1024)} GB</span></p>
          <p><span className="text-slate-500">Exposed Port:</span> <span className="text-slate-100">{job.exposed_port ?? "not exposed"}</span></p>
        </div>
      </div>
    </aside>
  );
}
