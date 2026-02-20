interface StatCardProps {
  label: string;
  value: string;
  sub: string;
  color: string;
}

export function StatCard({ label, value, sub, color }: StatCardProps) {
  return (
    <div className="bg-surface border border-border rounded-xl p-5 transition-colors hover:border-accent">
      <div className="text-[0.7rem] uppercase tracking-widest text-text-dim mb-2">
        {label}
      </div>
      <div className={`text-3xl font-bold leading-none ${color}`}>{value}</div>
      <div className="text-xs text-text-dim mt-1">{sub}</div>
    </div>
  );
}
