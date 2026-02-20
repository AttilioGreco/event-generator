document.addEventListener('alpine:init', () => {
  const MAX_UI_EVENTS = 1000;

  Alpine.data('dashboard', () => ({
    connected: false,
    totalEps: 0,
    totalEvents: 0,
    uptimeSecs: 0,
    streams: [],
    expandedStream: null,
    streamLogs: {},
    streamSearch: {},
    streamSockets: {},
    streamFollowTail: {},
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
    },
    toggleStream(name) {
      if (this.expandedStream === name) {
        this.expandedStream = null;
        this.disconnectStreamSocket(name);
        return;
      }

      if (this.expandedStream) {
        this.disconnectStreamSocket(this.expandedStream);
      }

      this.expandedStream = name;
      if (this.streamFollowTail[name] === undefined) {
        this.streamFollowTail[name] = true;
      }
      this.ensureStreamSocket(name);
      this.scrollStreamToBottom(name);
    },
    ensureStreamSocket(name) {
      if (this.streamSockets[name]) {
        return;
      }

      if (!this.streamLogs[name]) {
        this.streamLogs[name] = [];
      }

      const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
      const ws = new WebSocket(
        proto + '//' + location.host + '/ws/stream/' + encodeURIComponent(name)
      );

      ws.onmessage = (e) => {
        const d = JSON.parse(e.data);

        if (d.type === 'snapshot' && Array.isArray(d.events)) {
          this.streamLogs[name] = d.events.slice(-MAX_UI_EVENTS);
          this.scrollStreamToBottom(name);
          return;
        }

        if (d.type === 'event' && typeof d.event === 'string') {
          const next = this.streamLogs[name] ? [...this.streamLogs[name], d.event] : [d.event];
          if (next.length > MAX_UI_EVENTS) {
            next.splice(0, next.length - MAX_UI_EVENTS);
          }
          this.streamLogs[name] = next;
          this.scrollStreamToBottom(name);
        }
      };

      ws.onclose = () => {
        delete this.streamSockets[name];

        if (this.expandedStream === name) {
          setTimeout(() => {
            if (this.expandedStream === name) {
              this.ensureStreamSocket(name);
            }
          }, 2000);
        }
      };

      this.streamSockets[name] = ws;
    },
    disconnectStreamSocket(name) {
      const ws = this.streamSockets[name];
      if (!ws) {
        return;
      }

      ws.onclose = null;
      ws.close();
      delete this.streamSockets[name];
    },
    filteredStreamLogs(name) {
      const logs = this.streamLogs[name] || [];
      const query = (this.streamSearch[name] || '').trim().toLowerCase();
      if (!query) {
        return logs;
      }
      return logs.filter((line) => line.toLowerCase().includes(query));
    },
    isFollowTail(name) {
      return this.streamFollowTail[name] !== false;
    },
    toggleFollowTail(name) {
      this.streamFollowTail[name] = !this.isFollowTail(name);
      if (this.streamFollowTail[name]) {
        this.scrollStreamToBottom(name);
      }
    },
    scrollStreamToBottom(name) {
      if (this.expandedStream !== name) {
        return;
      }

      if (!this.isFollowTail(name)) {
        return;
      }

      this.$nextTick(() => {
        const lists = document.querySelectorAll('.stream-live-list');
        for (const list of lists) {
          if (list.getAttribute('data-stream') === name) {
            list.scrollTop = list.scrollHeight;
            return;
          }
        }
      });
    },
    destroy() {
      for (const name of Object.keys(this.streamSockets)) {
        this.disconnectStreamSocket(name);
      }
    }
  }));
});
