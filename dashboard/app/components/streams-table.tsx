import { useState } from "react";
import { Play, Square } from "lucide-react";

import type { StreamData } from "~/hooks/use-stats-socket";

import { StreamPanel } from "./stream-panel";

interface StreamsTableProps {
  streams: StreamData[];
}

async function startStream(name: string) {
  await fetch(`/api/streams/${encodeURIComponent(name)}/start`, {
    method: "POST",
  });
}

async function stopStream(name: string) {
  await fetch(`/api/streams/${encodeURIComponent(name)}/stop`, {
    method: "POST",
  });
}

async function startAll() {
  await fetch("/api/streams/start-all", { method: "POST" });
}

async function stopAll() {
  await fetch("/api/streams/stop-all", { method: "POST" });
}

export function StreamsTable({ streams }: StreamsTableProps) {
  const [expanded, setExpanded] = useState<string | null>(null);
  const maxEps = Math.max(1, ...streams.map((s) => s.eps));
  const hasRunning = streams.some((s) => s.status === "running");
  const hasStopped = streams.some((s) => s.status === "stopped");

  return (
    <div className="flex flex-col gap-3">
      <div className="flex gap-2 justify-end">
        {hasStopped && (
          <button
            type="button"
            onClick={startAll}
            className="h-7.5 px-3 flex items-center gap-1.5 text-xs border border-border rounded-lg bg-surface hover:bg-accent/10 hover:border-accent text-text-dim hover:text-text transition-colors cursor-pointer"
          >
            <Play size={12} />
            Start All
          </button>
        )}
        {hasRunning && (
          <button
            type="button"
            onClick={stopAll}
            className="h-7.5 px-3 flex items-center gap-1.5 text-xs border border-border rounded-lg bg-surface hover:bg-red/10 hover:border-red text-text-dim hover:text-red transition-colors cursor-pointer"
          >
            <Square size={12} />
            Stop All
          </button>
        )}
      </div>
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
              <th className="text-center px-5 py-3 text-[0.7rem] uppercase tracking-widest text-text-dim bg-surface2 border-b border-border sticky top-0 z-10">
                Actions
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
  const isRunning = stream.status === "running";
  const isError = stream.status === "error";

  const dotColor = isRunning
    ? "bg-green"
    : isError
      ? "bg-red"
      : "bg-text-dim/30";

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
            <span className={`w-1.5 h-1.5 rounded-full ${dotColor}`} />
            <span className="flex flex-col">
              <span>{stream.name}</span>
              {isError && stream.error && (
                <span className="text-[0.65rem] text-red truncate max-w-60">
                  {stream.error}
                </span>
              )}
            </span>
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
        <td className="px-5 py-3.5 border-b border-border text-center">
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              if (isRunning) {
                stopStream(stream.name);
              } else {
                startStream(stream.name);
              }
            }}
            className={`inline-flex items-center justify-center w-7 h-7 rounded-md border transition-colors cursor-pointer ${
              isRunning
                ? "border-border hover:border-red hover:bg-red/10 text-text-dim hover:text-red"
                : "border-border hover:border-accent hover:bg-accent/10 text-text-dim hover:text-accent"
            }`}
            title={isRunning ? "Stop" : "Start"}
          >
            {isRunning ? <Square size={12} /> : <Play size={12} />}
          </button>
        </td>
      </tr>
      {isExpanded && (
        <tr>
          <td colSpan={6} className="p-0">
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
