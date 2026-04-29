type Props = {
  line: string;
  index: number;
};

function tone(line: string) {
  const normalized = line.toUpperCase();

  if (normalized.includes("ERROR") || normalized.includes("FATAL")) {
    return "text-red-400";
  }

  if (normalized.includes("WARN")) {
    return "text-amber-400";
  }

  if (normalized.includes("INFO") || normalized.includes("SUCCESS")) {
    return "text-green-400";
  }

  return "text-white/70";
}

export function LogLine({ line, index }: Props) {
  return (
    <p className={`font-mono text-sm leading-6 ${tone(line)}`}>
      <span className="mr-3 inline-block min-w-10 text-right text-xs text-slate-500">{index + 1}</span>
      {line}
    </p>
  );
}
