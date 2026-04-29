"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { Loader2 } from "lucide-react";
import { motion } from "framer-motion";
import { TerminalHeader } from "@/components/jobs/TerminalHeader";
import { LogLine } from "@/components/jobs/LogLine";

type Props = {
  jobId: string;
  status: string;
  lines: string[];
  isLoading: boolean;
  isRefreshing: boolean;
  hasStarted: boolean;
  errorMessage?: string;
  onRetry: () => void;
};

const FAILURE_STATES = new Set(["Failed", "Stopped", "Preempted"]);

export function JobTerminal({
  jobId,
  status,
  lines,
  isLoading,
  isRefreshing,
  hasStarted,
  errorMessage,
  onRetry,
}: Props) {
  const [scrollLocked, setScrollLocked] = useState(false);
  const [isAtBottom, setIsAtBottom] = useState(true);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const bottomRef = useRef<HTMLDivElement | null>(null);

  const terminalFailed = FAILURE_STATES.has(status);
  const logText = useMemo(() => lines.join("\n"), [lines]);

  useEffect(() => {
    if (scrollLocked || !bottomRef.current) {
      return;
    }

    bottomRef.current.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [lines, scrollLocked]);

  const onScroll = () => {
    const element = containerRef.current;
    if (!element) {
      return;
    }

    const nearBottom = element.scrollHeight - element.scrollTop - element.clientHeight < 24;
    setIsAtBottom(nearBottom);
    setScrollLocked(!nearBottom);
  };

  const onCopyAll = async () => {
    await navigator.clipboard.writeText(logText);
  };

  const onDownload = () => {
    const blob = new Blob([logText], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `${jobId}.log.txt`;
    document.body.appendChild(anchor);
    anchor.click();
    document.body.removeChild(anchor);
    URL.revokeObjectURL(url);
  };

  return (
    <section className="overflow-hidden rounded-xl border border-white/10 bg-[#0a0a0f]">
      <TerminalHeader label={`job ${jobId}`} />

      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-white/10 px-4 py-3 text-xs text-slate-400">
        <div className="flex items-center gap-2">
          <span className="rounded-full border border-white/10 bg-white/5 px-2 py-1 font-mono">{lines.length} lines</span>
          {isRefreshing ? <Loader2 size={13} className="animate-spin text-cyan-300" /> : null}
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setScrollLocked((current) => !current)}
            className="rounded-md border border-white/10 bg-white/5 px-2 py-1 hover:bg-white/10"
          >
            {scrollLocked ? "Resume auto-scroll" : "Scroll lock"}
          </button>
          <button
            type="button"
            onClick={onCopyAll}
            disabled={lines.length === 0}
            className="rounded-md border border-white/10 bg-white/5 px-2 py-1 hover:bg-white/10 disabled:opacity-50"
          >
            Copy All Logs
          </button>
          <button
            type="button"
            onClick={onDownload}
            disabled={lines.length === 0}
            className="rounded-md border border-white/10 bg-white/5 px-2 py-1 hover:bg-white/10 disabled:opacity-50"
          >
            Download Logs
          </button>
        </div>
      </div>

      {terminalFailed ? (
        <div className="border-b border-red-500/20 bg-red-500/10 px-4 py-2 text-sm text-red-300">Job exited with error</div>
      ) : null}

      <div ref={containerRef} onScroll={onScroll} className="h-[560px] overflow-auto px-3 py-4">
        {isLoading ? (
          <div className="flex h-full items-center justify-center gap-3 text-sm text-slate-400">
            <Loader2 size={16} className="animate-spin" /> Loading logs...
          </div>
        ) : errorMessage ? (
          <div className="mx-auto mt-8 max-w-md rounded-lg border border-red-500/20 bg-red-500/10 p-4 text-sm text-red-200">
            <p>{errorMessage}</p>
            <button type="button" onClick={onRetry} className="mt-3 rounded-md border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-slate-100">
              Retry
            </button>
          </div>
        ) : !hasStarted ? (
          <div className="flex h-full items-center justify-center gap-3 text-sm text-slate-400">
            <Loader2 size={16} className="animate-spin" /> Waiting for job to start...
          </div>
        ) : lines.length === 0 ? (
          <div className="flex h-full items-center justify-center text-sm text-slate-400">No logs available yet. Retrying...</div>
        ) : (
          <motion.div initial={{ opacity: 0.8 }} animate={{ opacity: 1 }} className="space-y-0.5">
            {lines.map((line, index) => (
              <LogLine key={`${index}-${line.slice(0, 24)}`} index={index} line={line} />
            ))}
            <div ref={bottomRef} />
          </motion.div>
        )}
      </div>

      {scrollLocked && !isAtBottom ? (
        <div className="border-t border-white/10 px-4 py-2 text-xs text-slate-500">Auto-scroll paused while you inspect earlier logs.</div>
      ) : null}
    </section>
  );
}
