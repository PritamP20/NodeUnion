"use client";

import { Bar, BarChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

type Props = {
  cpuCost: number;
  ramCost: number;
  baseFee: number;
};

export function CostBreakdownChart({ cpuCost, ramCost, baseFee }: Props) {
  const data = [
    { name: "CPU", value: Number(cpuCost.toFixed(4)) },
    { name: "RAM", value: Number(ramCost.toFixed(4)) },
    { name: "Base", value: Number(baseFee.toFixed(4)) },
  ];

  return (
    <div className="h-56 w-full rounded-xl border border-white/10 bg-[#0a0a0f] p-3">
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={data}>
          <XAxis dataKey="name" stroke="#94a3b8" tickLine={false} axisLine={false} />
          <YAxis stroke="#94a3b8" tickLine={false} axisLine={false} />
          <Tooltip
            formatter={((value: number | undefined) => (value !== undefined ? `${value} SOL` : "")) as any}
            contentStyle={{
              background: "rgba(10,10,15,0.95)",
              border: "1px solid rgba(255,255,255,0.1)",
              borderRadius: 12,
              color: "#e5e7eb",
            }}
          />
          <Bar dataKey="value" fill="#6366f1" radius={[8, 8, 0, 0]} />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
