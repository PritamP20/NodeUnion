import { formatDistanceToNow } from "date-fns";
import { getStatusColor, type NodeHealthMapNode } from "@/lib/node-utils";

function truncateNodeId(nodeId: string) {
  return nodeId.length > 12 ? `${nodeId.slice(0, 12)}…` : nodeId;
}

function statusLabel(status: NodeHealthMapNode["status"]) {
  return status[0].toUpperCase() + status.slice(1);
}

export function MapTooltip({
  node,
  x,
  y,
}: {
  node: NodeHealthMapNode;
  x: number;
  y: number;
}) {
  const cpuPct = typeof node.cpuAvailablePct === "number" ? Math.max(0, 100 - node.cpuAvailablePct) : undefined;
  const ramPct = typeof node.ramAvailableMb === "number" ? Math.max(0, 100 - Math.round((node.ramAvailableMb / 64_000) * 100)) : undefined;
  const statusColor = getStatusColor(node.status, node.knownRegion);

  return (
    <div
      className="pointer-events-none absolute z-30 min-w-56 rounded-lg border border-white/10 bg-[#0f0f1a] px-3 py-2 text-left text-sm shadow-xl"
      style={{ left: x, top: y, transform: "translate(-50%, calc(-100% - 12px))" }}
    >
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-xs text-white">{truncateNodeId(node.nodeId)}</p>
          <p className="mt-1 text-[11px] uppercase tracking-[0.22em] text-slate-400">{node.networkName}</p>
        </div>
        <div className="flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-2 py-1 text-[11px] text-slate-100">
          <span className="h-2 w-2 rounded-full" style={{ backgroundColor: statusColor }} />
          {statusLabel(node.status)}
        </div>
      </div>

      <div className="mt-3 grid gap-1 text-xs text-slate-300">
        <p>
          Last heartbeat: <span className="text-slate-100">{formatDistanceToNow(new Date(node.lastSeenEpochSecs * 1000), { addSuffix: true })}</span>
        </p>
        {typeof cpuPct === "number" ? <p>CPU: <span className="text-slate-100">{cpuPct}%</span></p> : null}
        {typeof ramPct === "number" ? <p>RAM: <span className="text-slate-100">{ramPct}%</span></p> : null}
      </div>
    </div>
  );
}