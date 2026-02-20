import { useState } from "react";

import type { StreamData } from "~/hooks/use-stats-socket";

import { StreamPanel } from "./stream-panel";

interface StreamsTableProps {
  streams: StreamData[];
}

export function StreamsTable({ streams }: StreamsTableProps) {
  const [expanded, setExpanded] = useState<string | null>(null);
  const maxEps = Math.max(1, ...streams.map((s) => s.eps));

  return (
    <div className="bg-surface border border-border rounded-xl overflow-y-auto max-h-[50vh]">
      <table className="w-full border-collapse">
        <thead>
          <tr>
            <th className="text-left px-5 py-3 text-[0.7rem] uppercase tracking-widest text-text-dim bg-surface2 border-b border-border sticky top-0 z-10">
              Name
            </th>
            <th className="text-left px-5 py-3 text-[0.7rem] uppercase tracking-widest text-text-dim bg-surface2 border-b border-border sticky top-0 z-10">
              Destination
            </th>
            <th className="text-left px-5 py-3 text-[0.7rem] uppercase tracking-widest text-text-dim bg-surface2 border-b border-border sticky top-0 z-10">
              EPS
            </th>
            <th className="text-left px-5 py-3 text-[0.7rem] uppercase tracking-widest text-text-dim bg-surface2 border-b border-border sticky top-0 z-10">
              Activity
            </th>
            <th className="text-right px-5 py-3 text-[0.7rem] uppercase tracking-widest text-text-dim bg-surface2 border-b border-border sticky top-0 z-10">
              Total
            </th>
          </tr>
        </thead>
        <tbody>
          {streams.map((s) => (
            <StreamRow
              key={s.name}
              stream={s}
              maxEps={maxEps}
              isExpanded={expanded === s.name}
              onToggle={() =>
                setExpanded((e) => (e === s.name ? null : s.name))
              }
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}

function StreamRow({
  stream,
  maxEps,
  isExpanded,
  onToggle,
}: {
  stream: StreamData;
  maxEps: number;
  isExpanded: boolean;
  onToggle: () => void;
}) {
  return (
    <>
      <tr
        className={`cursor-pointer hover:bg-accent/5 ${isExpanded ? "bg-accent/10" : ""}`}
        onClick={onToggle}
      >
        <td className="px-5 py-3.5 text-sm border-b border-border">
          <div className="flex items-center gap-2">
            <span
              className={`text-[0.65rem] text-text-dim transition-transform ${isExpanded ? "rotate-90" : ""}`}
            >
              &#9654;
            </span>
            <span className="w-1.5 h-1.5 rounded-full bg-green" />
            {stream.name}
          </div>
        </td>
        <td className="px-5 py-3.5 text-xs text-text-dim border-b border-border max-w-60 truncate">
          {stream.destination}
        </td>
        <td className="px-5 py-3.5 text-sm font-semibold text-accent border-b border-border">
          {formatNumber(stream.eps)}
        </td>
        <td className="px-5 py-3.5 border-b border-border">
          <div className="w-[120px] h-1.5 bg-surface2 rounded-full overflow-hidden">
            <div
              className="h-full bg-accent rounded-full transition-all duration-500"
              style={{
                width: `${Math.min(100, (stream.eps / maxEps) * 100)}%`,
                minWidth: "2px",
              }}
            />
          </div>
        </td>
        <td className="px-5 py-3.5 text-sm text-text-dim text-right border-b border-border">
          {formatNumber(stream.total)}
        </td>
      </tr>
      {isExpanded && (
        <tr>
          <td colSpan={5} className="p-0">
            <StreamPanel name={stream.name} />
          </td>
        </tr>
      )}
    </>
  );
}

function formatNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}
