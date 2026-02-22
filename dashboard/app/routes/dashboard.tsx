import { StatCard } from "~/components/stat-card";
import { StreamsTable } from "~/components/streams-table";
import { Badge } from "~/components/ui/badge";
import { useStatsSocket } from "~/hooks/use-stats-socket";

export default function DashboardPage() {
  const { connected, uptimeSecs, totalEps, totalEvents, streams } = useStatsSocket();

  return (
    <>
      <div className="flex items-center gap-2 mb-4">
        <span
          className={`w-2 h-2 rounded-full ${connected ? "bg-green-400 animate-pulse" : "bg-destructive"}`}
        />
        <span className="text-xs text-muted-foreground">
          {connected ? "Live" : "Disconnected"}
        </span>
      </div>

      {!connected && (
        <div className="mb-4 px-4 py-3 rounded-lg border border-destructive/30 bg-destructive/10 text-destructive text-sm">
          Cannot connect to backend — retrying…
        </div>
      )}

      <div className="grid grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-4 mb-8">
        <StatCard
          label="Total EPS"
          value={formatNumber(totalEps)}
          sub="events per second"
          color="text-primary"
        />
        <StatCard
          label="Total Events"
          value={formatNumber(totalEvents)}
          sub="since start"
          color="text-green-400"
        />
        <StatCard
          label="Active Streams"
          value={String(streams.filter((s) => s.status === "running").length)}
          sub="concurrent"
          color="text-orange-400"
        />
        <StatCard
          label="Uptime"
          value={formatUptime(uptimeSecs)}
          sub="elapsed"
          color="text-purple-400"
        />
      </div>

      <div className="flex items-center gap-3 mb-4">
        <h2 className="text-[0.75rem] font-semibold uppercase tracking-widest text-muted-foreground">
          Streams
        </h2>
        <Badge variant="secondary">{streams.length}</Badge>
      </div>

      <StreamsTable streams={streams} />
    </>
  );
}

function formatNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

function formatUptime(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}
