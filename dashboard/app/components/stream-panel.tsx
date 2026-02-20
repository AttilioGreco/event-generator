import { useEffect, useRef, useState } from "react";

import { useStreamSocket } from "~/hooks/use-stream-socket";

interface StreamPanelProps {
  name: string;
}

export function StreamPanel({ name }: StreamPanelProps) {
  const logs = useStreamSocket(name);
  const [search, setSearch] = useState("");
  const [followTail, setFollowTail] = useState(true);
  const listRef = useRef<HTMLDivElement>(null);

  const filtered = search
    ? logs.filter((l) => l.toLowerCase().includes(search.toLowerCase()))
    : logs;

  useEffect(() => {
    if (followTail && listRef.current) {
      listRef.current.scrollTop = listRef.current.scrollHeight;
    }
  }, [filtered.length, followTail]);

  return (
    <div className="border-t border-border p-3 bg-surface2">
      <div className="flex items-center justify-between gap-3 mb-2">
        <input
          type="text"
          placeholder="Search events..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="flex-1 max-w-[360px] h-8 px-3 text-xs bg-surface border border-border rounded-lg text-text focus:outline-none focus:border-accent"
        />
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={() => setFollowTail((f) => !f)}
            className={`h-[30px] px-3 text-[0.72rem] border rounded-lg cursor-pointer ${
              followTail
                ? "border-accent text-text"
                : "border-border text-text-dim bg-surface"
            }`}
          >
            Tail {followTail ? "ON" : "OFF"}
          </button>
          <span className="text-[0.72rem] text-text-dim whitespace-nowrap">
            {logs.length} / 1000
          </span>
        </div>
      </div>
      <div
        ref={listRef}
        className="max-h-60 overflow-auto border border-border rounded-lg bg-surface p-2"
      >
        {filtered.map((line, i) => (
          <div
            key={i}
            className="text-[0.74rem] leading-relaxed text-text whitespace-pre-wrap break-all border-b border-white/5 py-1 last:border-b-0"
          >
            {line}
          </div>
        ))}
      </div>
    </div>
  );
}
