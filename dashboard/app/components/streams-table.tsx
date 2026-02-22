import { useState } from "react";
import { Play, Square } from "lucide-react";

import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import { ScrollArea } from "~/components/ui/scroll-area";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "~/components/ui/table";
import type { StreamData } from "~/hooks/use-stats-socket";
import { StreamPanel } from "./stream-panel";

interface StreamsTableProps {
  streams: StreamData[];
}

async function startStream(name: string) {
  await fetch(`/api/streams/${encodeURIComponent(name)}/start`, { method: "POST" });
}

async function stopStream(name: string) {
  await fetch(`/api/streams/${encodeURIComponent(name)}/stop`, { method: "POST" });
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
          <Button variant="outline" size="sm" onClick={startAll} className="gap-1.5">
            <Play size={12} />
            Start All
          </Button>
        )}
        {hasRunning && (
          <Button
            variant="outline"
            size="sm"
            onClick={stopAll}
            className="gap-1.5 hover:border-destructive hover:text-destructive hover:bg-destructive/10"
          >
            <Square size={12} />
            Stop All
          </Button>
        )}
      </div>

      <ScrollArea className="max-h-[50vh] rounded-xl border border-border">
        <Table>
          <TableHeader>
            <TableRow className="hover:bg-transparent">
              <TableHead>Name</TableHead>
              <TableHead>Destination</TableHead>
              <TableHead>EPS</TableHead>
              <TableHead>Activity</TableHead>
              <TableHead className="text-right">Total</TableHead>
              <TableHead className="text-center">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {streams.map((s) => (
              <StreamRow
                key={s.name}
                stream={s}
                maxEps={maxEps}
                isExpanded={expanded === s.name}
                onToggle={() => setExpanded((e) => (e === s.name ? null : s.name))}
              />
            ))}
          </TableBody>
        </Table>
      </ScrollArea>
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

  return (
    <>
      <TableRow
        className={`cursor-pointer ${isExpanded ? "bg-accent/5" : ""}`}
        onClick={onToggle}
      >
        <TableCell>
          <div className="flex items-center gap-2">
            <span
              className={`text-[0.65rem] text-muted-foreground transition-transform duration-150 ${isExpanded ? "rotate-90" : ""}`}
            >
              ▶
            </span>
            <StatusDot status={stream.status} />
            <span className="flex flex-col">
              <span>{stream.name}</span>
              {isError && stream.error && (
                <span className="text-[0.65rem] text-destructive truncate max-w-60">
                  {stream.error}
                </span>
              )}
            </span>
          </div>
        </TableCell>

        <TableCell className="text-xs text-muted-foreground max-w-60 truncate">
          {stream.destination}
        </TableCell>

        <TableCell className="font-semibold text-primary">
          {formatNumber(stream.eps)}
        </TableCell>

        <TableCell>
          <div className="w-[120px] h-1.5 bg-secondary rounded-full overflow-hidden">
            <div
              className="h-full bg-primary rounded-full transition-all duration-500"
              style={{
                width: `${Math.min(100, (stream.eps / maxEps) * 100)}%`,
                minWidth: "2px",
              }}
            />
          </div>
        </TableCell>

        <TableCell className="text-right text-muted-foreground">
          {formatNumber(stream.total)}
        </TableCell>

        <TableCell className="text-center">
          <Button
            variant="ghost"
            size="icon"
            className={`w-7 h-7 ${
              isRunning
                ? "hover:text-destructive hover:bg-destructive/10"
                : "hover:text-primary hover:bg-primary/10"
            }`}
            title={isRunning ? "Stop" : "Start"}
            onClick={(e) => {
              e.stopPropagation();
              isRunning ? stopStream(stream.name) : startStream(stream.name);
            }}
          >
            {isRunning ? <Square size={12} /> : <Play size={12} />}
          </Button>
        </TableCell>
      </TableRow>

      {isExpanded && (
        <TableRow className="hover:bg-transparent">
          <TableCell colSpan={6} className="p-0">
            <StreamPanel name={stream.name} />
          </TableCell>
        </TableRow>
      )}
    </>
  );
}

function StatusDot({ status }: { status: StreamData["status"] }) {
  const cls =
    status === "running"
      ? "bg-green-400 animate-pulse"
      : status === "error"
        ? "bg-destructive"
        : "bg-muted-foreground/30";
  return <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${cls}`} />;
}

// Unused export kept to avoid breaking any future Badge usage reference
export function StreamStatusBadge({ status }: { status: StreamData["status"] }) {
  if (status === "running")
    return <Badge className="bg-green-400/20 text-green-400 border-green-400/30">running</Badge>;
  if (status === "error")
    return <Badge variant="destructive">error</Badge>;
  return <Badge variant="secondary">stopped</Badge>;
}

function formatNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}
