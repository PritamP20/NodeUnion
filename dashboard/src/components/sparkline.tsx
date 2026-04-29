"use client";

import {
  Area,
  AreaChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

export function Sparkline({ values }: { values: number[] }) {
  const data = values.map((value, index) => ({
    label: String(index + 1),
    value,
  }));

  return (
    <div className="h-20 w-full">
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={data}>
          <defs>
            <linearGradient id="nodeunion-sparkline" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="#6366F1" stopOpacity={0.45} />
              <stop offset="100%" stopColor="#22D3EE" stopOpacity={0.04} />
            </linearGradient>
          </defs>
          <XAxis dataKey="label" hide />
          <YAxis hide domain={["dataMin - 1", "dataMax + 1"]} />
          <Tooltip
            contentStyle={{
              background: "rgba(9, 10, 15, 0.96)",
              border: "1px solid rgba(255, 255, 255, 0.12)",
              borderRadius: 12,
              color: "#f4f4f7",
              fontFamily: "var(--font-mono)",
              fontSize: 12,
            }}
            labelStyle={{ color: "#9aa3b2" }}
            cursor={{ stroke: "rgba(99, 102, 241, 0.3)" }}
          />
          <Area
            type="monotone"
            dataKey="value"
            stroke="#6366F1"
            strokeWidth={2}
            fill="url(#nodeunion-sparkline)"
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}