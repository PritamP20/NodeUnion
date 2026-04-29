"use client";

import { useEffect, useMemo, useState } from "react";

type Props = {
  startedAtEpochSecs: number;
  isRunning: boolean;
};

function formatDuration(totalSeconds: number) {
  const safe = Math.max(0, totalSeconds);
  const hours = Math.floor(safe / 3600);
  const minutes = Math.floor((safe % 3600) / 60);
  const seconds = safe % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m ${seconds}s`;
  }

  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }

  return `${seconds}s`;
}

export function LiveDurationCounter({ startedAtEpochSecs, isRunning }: Props) {
  const [nowEpochSecs, setNowEpochSecs] = useState(() => Math.floor(Date.now() / 1000));

  useEffect(() => {
    if (!isRunning) {
      return;
    }

    const timer = window.setInterval(() => {
      setNowEpochSecs(Math.floor(Date.now() / 1000));
    }, 1000);

    return () => {
      window.clearInterval(timer);
    };
  }, [isRunning]);

  const value = useMemo(() => formatDuration(nowEpochSecs - startedAtEpochSecs), [nowEpochSecs, startedAtEpochSecs]);

  return <span className="font-mono text-slate-100">{value}</span>;
}
