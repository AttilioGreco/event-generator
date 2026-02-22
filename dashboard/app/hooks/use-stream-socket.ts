import { useCallback, useEffect, useRef, useState } from "react";

const MAX_EVENTS = 1000;

export function useStreamSocket(streamName: string | null) {
  const [logs, setLogs] = useState<string[]>([]);
  const wsRef = useRef<WebSocket | null>(null);
  const retryRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const connect = useCallback(() => {
    if (!streamName) return;

    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    const ws = new WebSocket(
      `${proto}//${location.host}/ws/stream/${encodeURIComponent(streamName)}`
    );

    ws.onmessage = (e) => {
      const d = JSON.parse(e.data);
      if (d.type === "snapshot" && Array.isArray(d.events)) {
        setLogs(d.events.slice(-MAX_EVENTS));
      } else if (d.type === "event" && typeof d.event === "string") {
        setLogs((prev) => {
          const next = [...prev, d.event];
          return next.length > MAX_EVENTS
            ? next.slice(next.length - MAX_EVENTS)
            : next;
        });
      }
    };

    ws.onerror = () => ws.close();
    ws.onclose = () => {
      retryRef.current = setTimeout(() => connect(), 3000);
    };

    wsRef.current = ws;
  }, [streamName]);

  useEffect(() => {
    setLogs([]);
    connect();
    return () => {
      clearTimeout(retryRef.current);
      wsRef.current?.close();
      wsRef.current = null;
    };
  }, [connect]);

  return logs;
}
