type StatusTone = "online" | "degraded" | "offline";

export function StatusDot({ tone = "online" }: { tone?: StatusTone }) {
  return <span aria-hidden="true" className="status-dot" data-tone={tone} />;
}