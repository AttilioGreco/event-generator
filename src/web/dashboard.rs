pub const HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Event Generator Dashboard</title>
<script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>
<style>
  :root {
    --bg: #0f172a;
    --surface: #1e293b;
    --surface2: #334155;
    --border: #475569;
    --text: #f1f5f9;
    --text-dim: #94a3b8;
    --accent: #38bdf8;
    --green: #4ade80;
    --orange: #fb923c;
    --red: #f87171;
    --purple: #c084fc;
  }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  html, body { height: 100%; }
  body {
    font-family: 'SF Mono', 'Cascadia Code', 'Fira Code', monospace;
    background: var(--bg);
    color: var(--text);
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }
  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 24px;
    flex: 1;
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  /* Header */
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 32px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  header h1 {
    font-size: 1.4rem;
    font-weight: 600;
    letter-spacing: -0.5px;
  }
  header h1 span { color: var(--accent); }
  .status-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.8rem;
    color: var(--text-dim);
  }
  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--green);
    animation: pulse 2s infinite;
  }
  .status-dot.disconnected { background: var(--red); animation: none; }
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  /* Summary Cards */
  .summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 16px;
    margin-bottom: 32px;
    flex-shrink: 0;
  }
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    transition: border-color 0.2s;
  }
  .card:hover { border-color: var(--accent); }
  .card-label {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-dim);
    margin-bottom: 8px;
  }
  .card-value {
    font-size: 2rem;
    font-weight: 700;
    line-height: 1;
  }
  .card-value.accent { color: var(--accent); }
  .card-value.green { color: var(--green); }
  .card-value.orange { color: var(--orange); }
  .card-value.purple { color: var(--purple); }
  .card-sub {
    font-size: 0.75rem;
    color: var(--text-dim);
    margin-top: 4px;
  }

  /* Streams Table */
  .streams-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .streams-section h2 {
    font-size: 0.75rem;
    font-weight: 600;
    margin-bottom: 16px;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 1px;
    flex-shrink: 0;
  }
  .streams-table {
    width: 100%;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    overflow-y: auto;
    max-height: 50vh;
  }
  .streams-table::-webkit-scrollbar {
    width: 6px;
  }
  .streams-table::-webkit-scrollbar-track {
    background: var(--surface);
  }
  .streams-table::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th {
    text-align: left;
    padding: 12px 20px;
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-dim);
    background: var(--surface2);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    z-index: 1;
  }
  td {
    padding: 14px 20px;
    font-size: 0.85rem;
    border-bottom: 1px solid var(--border);
  }
  tbody tr:last-child td { border-bottom: none; }
  tbody tr:hover td { background: rgba(56, 189, 248, 0.04); }
  .stream-name {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .stream-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--green);
    flex-shrink: 0;
  }
  .destination {
    color: var(--text-dim);
    font-size: 0.78rem;
    max-width: 240px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .eps-value { color: var(--accent); font-weight: 600; }
  .total-value { color: var(--text-dim); }

  /* EPS Bar */
  .eps-bar-container {
    width: 120px;
    height: 6px;
    background: var(--surface2);
    border-radius: 3px;
    overflow: hidden;
  }
  .eps-bar {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 0.5s ease;
    min-width: 2px;
  }

  /* Footer */
  footer {
    padding: 16px 0;
    border-top: 1px solid var(--border);
    font-size: 0.7rem;
    color: var(--text-dim);
    text-align: center;
    flex-shrink: 0;
    margin-top: 24px;
  }
</style>
</head>
<body>
<div class="container" x-data="dashboard()" x-init="connect()">

  <header>
    <h1><span>&#9654;</span> event-generator</h1>
    <div class="status-badge">
      <div class="status-dot" :class="{ 'disconnected': !connected }"></div>
      <span x-text="connected ? 'Live' : 'Disconnected'"></span>
    </div>
  </header>

  <div class="summary">
    <div class="card">
      <div class="card-label">Total EPS</div>
      <div class="card-value accent" x-text="formatNumber(totalEps)"></div>
      <div class="card-sub">events per second</div>
    </div>
    <div class="card">
      <div class="card-label">Total Events</div>
      <div class="card-value green" x-text="formatNumber(totalEvents)"></div>
      <div class="card-sub">since start</div>
    </div>
    <div class="card">
      <div class="card-label">Active Streams</div>
      <div class="card-value orange" x-text="streams.length"></div>
      <div class="card-sub">concurrent</div>
    </div>
    <div class="card">
      <div class="card-label">Uptime</div>
      <div class="card-value purple" x-text="formatUptime(uptimeSecs)"></div>
      <div class="card-sub" x-text="'since ' + startedAt"></div>
    </div>
  </div>

  <div class="streams-section">
    <h2>Streams</h2>
    <div class="streams-table">
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Destination</th>
            <th>EPS</th>
            <th>Activity</th>
            <th style="text-align:right">Total Events</th>
          </tr>
        </thead>
        <tbody>
          <template x-for="s in streams" :key="s.name">
            <tr>
              <td>
                <div class="stream-name">
                  <div class="stream-dot"></div>
                  <span x-text="s.name"></span>
                </div>
              </td>
              <td>
                <div class="destination" x-text="s.destination" :title="s.destination"></div>
              </td>
              <td class="eps-value" x-text="formatNumber(s.eps)"></td>
              <td>
                <div class="eps-bar-container">
                  <div class="eps-bar" :style="'width:' + Math.min(100, (s.eps / maxEps) * 100) + '%'"></div>
                </div>
              </td>
              <td class="total-value" style="text-align:right" x-text="formatNumber(s.total)"></td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>
  </div>

  <footer>event-generator &middot; Rust &middot; stats via WebSocket</footer>
</div>

<script>
function dashboard() {
  return {
    connected: false,
    totalEps: 0,
    totalEvents: 0,
    uptimeSecs: 0,
    streams: [],
    startedAt: new Date().toLocaleTimeString(),
    get maxEps() {
      if (this.streams.length === 0) return 1;
      return Math.max(1, ...this.streams.map(s => s.eps));
    },
    formatNumber(n) {
      if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
      if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
      return String(n);
    },
    formatUptime(secs) {
      const h = Math.floor(secs / 3600);
      const m = Math.floor((secs % 3600) / 60);
      const s = secs % 60;
      return String(h).padStart(2, '0') + ':' +
             String(m).padStart(2, '0') + ':' +
             String(s).padStart(2, '0');
    },
    connect() {
      const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
      const ws = new WebSocket(proto + '//' + location.host + '/ws');
      ws.onopen = () => { this.connected = true; };
      ws.onclose = () => {
        this.connected = false;
        setTimeout(() => this.connect(), 2000);
      };
      ws.onmessage = (e) => {
        const d = JSON.parse(e.data);
        this.uptimeSecs = d.uptime_secs;
        this.totalEps = d.total_eps;
        this.totalEvents = d.total_events;
        this.streams = d.streams;
      };
    }
  };
}
</script>
</body>
</html>
"##;
