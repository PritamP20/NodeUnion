"use client";

import { Check, Copy } from "lucide-react";
import { useState } from "react";

export function CopyButton({ value }: { value: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(value);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1200);
  };

  return (
    <button type="button" onClick={handleCopy} className="copy-button inline-flex items-center gap-2 rounded-full px-3 py-1.5 text-[11px] font-semibold uppercase tracking-[0.2em] text-slate-200">
      {copied ? <Check size={14} className="text-emerald-300" /> : <Copy size={14} />}
      {copied ? "Copied" : "Copy"}
    </button>
  );
}