import { StatCard } from "~/components/stat-card";
import { StreamsTable } from "~/components/streams-table";
import { useStatsSocket } from "~/hooks/use-stats-socket";

export default function DashboardPage() {
  const { connected, uptimeSecs, totalEps, totalEvents, streams } =
    useStatsSocket();

  return (
    <>
      <div className="flex items-center gap-2 mb-4">
        <div
          className={`w-2 h-2 rounded-full ${connected ? "bg-green animate-pulse" : "bg-red"}`}
        />
        <span className="text-xs text-text-dim">
          {connected ? "Live" : "Disconnected"}
        </span>
      </div>

      {!connected && (
        <div className="mb-4 px-4 py-3 rounded-lg border border-red/30 bg-red/10 text-red text-sm">
          Can not connect to backend — retrying...
        </div>
      )}

      <div className="grid grid-cols-[repeat(auto-fit,minmax(220px,1fr))] gap-4 mb-8">
        <StatCard
          label="Total EPS"
          value={formatNumber(totalEps)}
          sub="events per second"
          color="text-accent"
        />
        <StatCard
          label="Total Events"
          value={formatNumber(totalEvents)}
          sub="since start"
          color="text-green"
        />
        <StatCard
          label="Active Streams"
          value={String(streams.length)}
          sub="concurrent"
          color="text-orange"
        />
        <StatCard
          label="Uptime"
          value={formatUptime(uptimeSecs)}
          sub="elapsed"
          color="text-purple"
        />
      </div>

      <h2 className="text-[0.75rem] font-semibold uppercase tracking-widest text-text-dim mb-4">
        Streams
      </h2>
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
