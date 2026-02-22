import { Card, CardContent } from "~/components/ui/card";

interface StatCardProps {
  label: string;
  value: string;
  sub: string;
  color: string;
}

export function StatCard({ label, value, sub, color }: StatCardProps) {
  return (
    <Card className="transition-colors hover:border-primary/50">
      <CardContent className="pt-5">
        <div className="text-[0.7rem] uppercase tracking-widest text-muted-foreground mb-2">
          {label}
        </div>
        <div className={`text-3xl font-bold leading-none ${color}`}>{value}</div>
        <div className="text-xs text-muted-foreground mt-1">{sub}</div>
      </CardContent>
    </Card>
  );
}
