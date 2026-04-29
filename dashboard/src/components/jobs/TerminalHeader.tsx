type Props = {
  label: string;
};

export function TerminalHeader({ label }: Props) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-t-xl border-b border-white/10 bg-white/5 px-4 py-3">
      <div className="flex items-center gap-2">
        <span className="h-2.5 w-2.5 rounded-full bg-red-400" />
        <span className="h-2.5 w-2.5 rounded-full bg-amber-400" />
        <span className="h-2.5 w-2.5 rounded-full bg-emerald-400" />
      </div>
      <p className="font-mono text-xs uppercase tracking-[0.2em] text-slate-300">{label}</p>
      <span className="w-14" aria-hidden="true" />
    </div>
  );
}
