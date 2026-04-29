"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import useSWR from "swr";
import { motion } from "framer-motion";
import { ChevronRight, Loader2 } from "lucide-react";
import { JobMetaPanel } from "@/components/jobs/JobMetaPanel";
import { JobTerminal } from "@/components/jobs/JobTerminal";
import { fetchJobById, fetchJobLogs, fetchMainSnapshot, type OrchestratorJob } from "@/lib/orchestrator-realtime";

type Props = {
  jobId: string;
};

const TERMINAL_STATES = new Set(["Done", "Failed", "Stopped", "Preempted"]);

type SseState = "pending" | "open" | "closed";

function splitLines(text: string) {
  return text
    .split("\n")
    .map((line) => line.trimEnd())
    .filter((line) => line.length > 0);
}

function parseSseChunk(raw: string) {
  if (!raw) {
    return [] as string[];
  }

  try {
    const payload = JSON.parse(raw) as { line?: string; message?: string; logs?: string[]; data?: string | string[] };

    if (Array.isArray(payload.logs)) {
      return payload.logs.flatMap((line) => splitLines(String(line)));
    }

    if (Array.isArray(payload.data)) {
      return payload.data.flatMap((line) => splitLines(String(line)));
    }

    const line = payload.line ?? payload.message ?? payload.data;
    if (typeof line === "string") {
      return splitLines(line);
    }
  } catch {
    return splitLines(raw);
  }

  return [] as string[];
}

export function JobDetailPageClient({ jobId }: Props) {
  const [sseState, setSseState] = useState<SseState>(() => {
    if (typeof window === "undefined") {
      return "pending";
    }

    return typeof EventSource === "undefined" ? "closed" : "pending";
  });
  const [streamLines, setStreamLines] = useState<string[]>([]);

  const { data: snapshot } = useSWR("/api/main/snapshot", () => fetchMainSnapshot(), {
    refreshInterval: 30000,
    revalidateOnFocus: true,
  });

  const {
    data: job,
    error: jobError,
    isLoading: isJobLoading,
  } = useSWR<OrchestratorJob>(`/api/orchestrator/jobs/${jobId}`, () => fetchJobById(jobId), {
    refreshInterval: (current) => {
      if (!current || !TERMINAL_STATES.has(current.status)) {
        return 5000;
      }
      return 0;
    },
    revalidateOnFocus: true,
  });

  const shouldPollLogs = useMemo(() => {
    if (!job) return true;
    if (sseState === "open") return false;
    return !TERMINAL_STATES.has(job.status);
  }, [job, sseState]);

  const {
    data: polledLines,
    error: logsError,
    isLoading: isLogsLoading,
    isValidating,
    mutate: retryLogs,
  } = useSWR<string[]>(
    sseState === "open" ? null : `/api/orchestrator/jobs/${jobId}/logs`,
    () => fetchJobLogs(jobId),
    {
      refreshInterval: shouldPollLogs ? 2000 : 0,
      revalidateOnFocus: true,
    },
  );

  useEffect(() => {
    if (typeof window === "undefined" || typeof EventSource === "undefined" || sseState === "closed") {
      return;
    }

    const source = new EventSource(`/api/orchestrator/jobs/${encodeURIComponent(jobId)}/logs`);
    let opened = false;

    source.onopen = () => {
      opened = true;
      setSseState("open");
    };

    source.onmessage = (event) => {
      const nextLines = parseSseChunk(event.data);
      if (nextLines.length === 0) {
        return;
      }

      setStreamLines((current) => [...current, ...nextLines]);
    };

    source.onerror = () => {
      source.close();
      setSseState(opened ? "closed" : "closed");
    };

    return () => {
      source.close();
    };
  }, [jobId, sseState]);

  const terminalLines = sseState === "open" ? streamLines : polledLines ?? [];
  const hasStarted = Boolean(job);
  const networkName = snapshot?.networks.find((network) => network.network_id === job?.network_id)?.name ?? job?.network_id ?? "unknown";

  if (isJobLoading) {
    return (
      <main className="mx-auto flex w-full max-w-7xl items-center justify-center px-4 py-20 sm:px-6 lg:px-8">
        <div className="glass-card flex items-center gap-3 rounded-2xl px-6 py-4 text-slate-300">
          <Loader2 size={18} className="animate-spin" /> Loading job details...
        </div>
      </main>
    );
  }

  if (!job || jobError) {
    return (
      <main className="mx-auto w-full max-w-4xl px-4 py-10 sm:px-6 lg:px-8">
        <div className="glass-card rounded-2xl border border-red-500/20 bg-red-500/10 p-6 text-red-200">
          Could not load job {jobId}. Please verify the ID and try again.
        </div>
      </main>
    );
  }

  return (
    <main className="mx-auto w-full max-w-7xl space-y-4 px-4 py-6 sm:px-6 lg:px-8 lg:py-10">
      <nav className="flex items-center gap-2 text-xs text-slate-400">
        <Link href="/provider" className="hover:text-slate-200">Provider</Link>
        <ChevronRight size={14} />
        <span>Jobs</span>
        <ChevronRight size={14} />
        <span className="font-mono text-slate-200">{job.job_id}</span>
      </nav>

      <motion.section
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.3, ease: "easeOut" }}
        className="grid gap-4 lg:grid-cols-[0.35fr_0.65fr]"
      >
        <JobMetaPanel job={job} networkName={networkName} />

        <JobTerminal
          jobId={job.job_id}
          status={job.status}
          lines={terminalLines}
          isLoading={isLogsLoading && terminalLines.length === 0}
          isRefreshing={isValidating}
          hasStarted={hasStarted}
          errorMessage={logsError ? "Failed to fetch logs." : undefined}
          onRetry={() => void retryLogs()}
        />
      </motion.section>
    </main>
  );
}
