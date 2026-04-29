import type { ReactNode } from "react";

export type TerminalLine = {
  kind: "input" | "output" | "system";
  text: string;
};

export function TerminalBlock({
  title,
  subtitle,
  lines,
  typingLine,
  prompt = ">",
  footer,
}: {
  title: string;
  subtitle?: string;
  lines: TerminalLine[];
  typingLine?: string;
  prompt?: string;
  footer?: ReactNode;
}) {
  return (
    <section className="terminal-shell rounded-[1.75rem] p-4 sm:p-5">
      <div className="flex items-center justify-between gap-4 border-b border-white/10 pb-4">
        <div>
          <p className="font-mono text-xs uppercase tracking-[0.28em] text-slate-300">{title}</p>
          {subtitle ? <p className="mt-1 text-sm text-slate-400">{subtitle}</p> : null}
        </div>
        {footer}
      </div>

      <div className="mt-4 rounded-[1.25rem] border border-white/10 bg-[#090b12] p-4 font-mono text-sm leading-6 text-slate-100">
        <div className="space-y-2">
          {lines.map((line, index) => (
            <p
              key={`${line.kind}-${index}-${line.text}`}
              className={
                line.kind === "input"
                  ? "text-cyan-300"
                  : line.kind === "system"
                    ? "text-slate-400"
                    : "text-emerald-300"
              }
            >
              <span className="mr-2 text-slate-500">{line.kind === "output" ? "✓" : ">"}</span>
              {line.text}
            </p>
          ))}
          {typingLine ? (
            <p className="text-cyan-300">
              <span className="mr-2 text-slate-500">{prompt}</span>
              {typingLine}
              <span className="ml-1 inline-block h-4 w-2 animate-pulse rounded-sm bg-cyan-300/90 align-middle" />
            </p>
          ) : null}
        </div>
      </div>
    </section>
  );
}