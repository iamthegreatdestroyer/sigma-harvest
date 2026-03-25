import { ResponsiveContainer, AreaChart, Area, Tooltip } from "recharts";

/**
 * Compact sparkline chart for dashboard stats.
 * Uses Recharts AreaChart in a small, minimal presentation.
 *
 * @param {Object} props
 * @param {Array<{date: string, value: number}>} props.data - Time-series data
 * @param {string} [props.color="#00e5ff"] - Line/fill color
 * @param {number} [props.height=48] - Chart height in pixels
 * @param {string} [props.dataKey="value"] - Key in data objects to plot
 */
export default function SparklineChart({
  data = [],
  color = "#00e5ff",
  height = 48,
  dataKey = "value",
}) {
  if (!data.length) {
    return (
      <div
        className="flex items-center justify-center text-text-muted text-[10px]"
        style={{ height }}
      >
        No data
      </div>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={height}>
      <AreaChart data={data} margin={{ top: 2, right: 0, bottom: 0, left: 0 }}>
        <defs>
          <linearGradient id={`grad-${dataKey}`} x1="0" y1="0" x2="0" y2="1">
            <stop offset="5%" stopColor={color} stopOpacity={0.3} />
            <stop offset="95%" stopColor={color} stopOpacity={0} />
          </linearGradient>
        </defs>
        <Tooltip
          contentStyle={{
            background: "#1a1a2e",
            border: "1px solid #2a2a3e",
            borderRadius: 6,
            fontSize: 11,
            padding: "4px 8px",
          }}
          labelStyle={{ color: "#888", fontSize: 10 }}
          itemStyle={{ color }}
          formatter={(val) => [`$${Number(val).toFixed(2)}`, null]}
          labelFormatter={(label) => label}
        />
        <Area
          type="monotone"
          dataKey={dataKey}
          stroke={color}
          strokeWidth={1.5}
          fill={`url(#grad-${dataKey})`}
          dot={false}
          activeDot={{ r: 2, fill: color }}
        />
      </AreaChart>
    </ResponsiveContainer>
  );
}
