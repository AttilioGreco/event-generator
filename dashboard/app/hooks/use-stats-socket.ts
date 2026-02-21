import { useCallback, useEffect, useRef, useState } from "react";

export interface StreamData {
  name: string;
  destination: string;
  eps: number;
  total: number;
  status: "running" | "stopped" | "error";
  error?: string | null;
}

interface StatsState {
  connected: boolean;
  uptimeSecs: number;
  totalEps: number;
  totalEvents: number;
  streams: StreamData[];
}

export function useStatsSocket(): StatsState {
  const [state, setState] = useState<StatsState>({
    connected: false,
    uptimeSecs: 0,
    totalEps: 0,
    totalEvents: 0,
    streams: [],
  });
  const wsRef = useRef<WebSocket | null>(null);
  const retryRef = useRef<ReturnType<typeof setTimeout>>();

  const connect = useCallback(() => {
    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    const ws = new WebSocket(`${proto}//${location.host}/ws`);

    ws.onopen = () => setState((s) => ({ ...s, connected: true }));
    ws.onerror = () => ws.close();
    ws.onclose = () => {
      setState((s) => ({ ...s, connected: false }));
      retryRef.current = setTimeout(connect, 3000);
    };
    ws.onmessage = (e) => {
      const d = JSON.parse(e.data);
      setState({
        connected: true,
        uptimeSecs: d.uptime_secs,
        totalEps: d.total_eps,
        totalEvents: d.total_events,
        streams: d.streams,
      });
    };

    wsRef.current = ws;
  }, []);

  useEffect(() => {
    connect();
    return () => {
      clearTimeout(retryRef.current);
      wsRef.current?.close();
    };
  }, [connect]);

  return state;
}
