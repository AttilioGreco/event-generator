import { useEffect, useRef, useState } from "react";

import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";
import { ScrollArea } from "~/components/ui/scroll-area";
import { useStreamSocket } from "~/hooks/use-stream-socket";

interface StreamPanelProps {
  name: string;
}

export function StreamPanel({ name }: StreamPanelProps) {
  const logs = useStreamSocket(name);
  const [search, setSearch] = useState("");
  const [followTail, setFollowTail] = useState(true);
  const bottomRef = useRef<HTMLDivElement>(null);

  const filtered = search
    ? logs.filter((l) => l.toLowerCase().includes(search.toLowerCase()))
    : logs;

  useEffect(() => {
    if (followTail && bottomRef.current) {
      bottomRef.current.scrollIntoView({ block: "end" });
    }
  }, [filtered.length, followTail]);

  return (
    <div className="border-t border-border p-3 bg-secondary/30">
      <div className="flex items-center justify-between gap-3 mb-2">
        <Input
          placeholder="Search events…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="max-w-[360px] h-8 text-xs"
        />
        <div className="flex items-center gap-3">
          <Button
            variant={followTail ? "default" : "outline"}
            size="sm"
            className="h-7 px-3 text-xs"
            onClick={() => setFollowTail((f) => !f)}
          >
            Tail {followTail ? "ON" : "OFF"}
          </Button>
          <span className="text-xs text-muted-foreground whitespace-nowrap">
            {logs.length} / 1000
          </span>
        </div>
      </div>
      <ScrollArea className="h-60 rounded-md border border-border bg-background">
        <div className="p-2">
          {filtered.map((line, i) => (
            <div
              key={i}
              className="text-[0.74rem] leading-relaxed text-foreground whitespace-pre-wrap break-all border-b border-white/5 py-1 last:border-b-0"
            >
              {line}
            </div>
          ))}
          <div ref={bottomRef} />
        </div>
      </ScrollArea>
    </div>
  );
}
