import { useCallback, useEffect, useRef, useState } from "react";
import { ChevronRight, Download, FileText, Plus, Trash2 } from "lucide-react";

import { RhaiEditor } from "~/components/rhai-editor";

const LS_KEY = "rhai-studio-scripts";
const LS_ACTIVE = "rhai-studio-active";
const LS_ACTIVE_NAME = "rhai-studio-active-name";

const DEFAULT_SCRIPT = `// Rhai Script - each execution = one trace
let req_id = uuid();
let user = pick(["alice", "bob", "carol"]);
let ip = fake_ipv4();

emit(now_iso() + " INFO  [req=" + req_id + "] start user=" + user + " ip=" + ip);

if weighted_bool(0.9) {
    let elapsed = int_range(5, 120);
    emit(now_iso() + " INFO  [req=" + req_id + "] 200 OK elapsed=" + elapsed + "ms");
} else {
    emit(now_iso() + " ERROR [req=" + req_id + "] 500 InternalServerError");
    emit("java.lang.RuntimeException: " + fake_message());
    emit("  at " + fake_java_class() + ".handle(Unknown Source)");
}

emit(now_iso() + " INFO  [req=" + req_id + "] end");
`;

const PRESETS: { name: string; code: string }[] = [
  {
    name: "Java Request Trace",
    code: DEFAULT_SCRIPT,
  },
  {
    name: "Simple Access Log",
    code: `let ip = fake_ipv4();
let method = fake_http_method();
let path = fake_http_path();
let status = fake_http_status();
let ua = fake_user_agent();
let bytes = int_range(200, 50000);

emit(ip + " - - [" + now_iso() + "] \\"" + method + " " + path + " HTTP/1.1\\" " + status + " " + bytes + " \\"-\\" \\"" + ua + "\\"");
`,
  },
  {
    name: "Firewall Events",
    code: `let src = fake_ipv4();
let dst = fake_ipv4();
let action = fake_action();
let proto = fake_protocol();
let src_port = fake_port();
let dst_port = pick(["80", "443", "22", "3389", "8080"]);
let sev = fake_severity();

emit(now_iso() + " FW action=" + action + " proto=" + proto + " src=" + src + ":" + src_port + " dst=" + dst + ":" + dst_port + " severity=" + sev);
`,
  },
];

interface SavedScript {
  name: string;
  code: string;
  updatedAt: number;
}

function loadSavedScripts(): SavedScript[] {
  try {
    return JSON.parse(localStorage.getItem(LS_KEY) || "[]");
  } catch {
    return [];
  }
}

function persistScripts(scripts: SavedScript[]) {
  localStorage.setItem(LS_KEY, JSON.stringify(scripts));
}

function loadActiveCode(): string | null {
  return localStorage.getItem(LS_ACTIVE);
}

function persistActiveCode(code: string) {
  localStorage.setItem(LS_ACTIVE, code);
}

// ---------------------------------------------------------------------------
// Download helper
// ---------------------------------------------------------------------------
function downloadFile(name: string, content: string) {
  const blob = new Blob([content], { type: "text/plain" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = name.endsWith(".rhai") ? name : `${name}.rhai`;
  a.click();
  URL.revokeObjectURL(url);
}

// ---------------------------------------------------------------------------
// File-tree sidebar icon helpers
// ---------------------------------------------------------------------------
const TreeChevron = ({ open }: { open: boolean }) => (
  <ChevronRight
    size={12}
    className={`shrink-0 transition-transform duration-150 ${open ? "rotate-90" : ""}`}
  />
);

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------
export default function StudioPage() {
  const [code, setCode] = useState(() => loadActiveCode() || DEFAULT_SCRIPT);
  const [activeName, setActiveName] = useState<string | null>(
    () => localStorage.getItem(LS_ACTIVE_NAME) ?? null
  );
  const [output, setOutput] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [samples, setSamples] = useState(3);
  const [saved, setSaved] = useState<SavedScript[]>(loadSavedScripts);
  const [showSaveDialog, setShowSaveDialog] = useState(false);
  const [saveName, setSaveName] = useState("");
  const [presetsOpen, setPresetsOpen] = useState(true);
  const [scriptsOpen, setScriptsOpen] = useState(true);
  const codeRef = useRef(code);
  codeRef.current = code;

  // Persist active code
  useEffect(() => {
    persistActiveCode(code);
  }, [code]);

  // ---------------------------------------------------------------------------
  // Run
  // ---------------------------------------------------------------------------
  const runScript = useCallback(async () => {
    const currentCode = codeRef.current;
    setLoading(true);
    setError("");
    try {
      const res = await fetch("/api/script/run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          code: currentCode,
          samples: Math.max(1, Math.min(20, samples)),
        }),
      });
      if (!res.ok) {
        setError(`Request failed: ${res.status}`);
        setOutput("");
        return;
      }
      const data = await res.json();
      setError(data.error || "");
      setOutput(
        Array.isArray(data.output) ? data.output.join("\n---\n") : ""
      );
    } catch {
      setError("Request failed");
      setOutput("");
    } finally {
      setLoading(false);
    }
  }, [samples]);

  // ---------------------------------------------------------------------------
  // Save
  // ---------------------------------------------------------------------------
  const commitSave = useCallback(
    (name: string) => {
      if (!name.trim()) return;
      const n = name.trim();
      const updated = saved.filter((s) => s.name !== n);
      updated.unshift({ name: n, code, updatedAt: Date.now() });
      setSaved(updated);
      persistScripts(updated);
      setActiveName(n);
      localStorage.setItem(LS_ACTIVE_NAME, n);
      setShowSaveDialog(false);
      setSaveName("");
    },
    [code, saved]
  );

  /** Ctrl+S: save to current name or open dialog */
  const handleSave = useCallback(() => {
    if (activeName) {
      commitSave(activeName);
    } else {
      setSaveName("");
      setShowSaveDialog(true);
    }
  }, [activeName, commitSave]);

  // ---------------------------------------------------------------------------
  // New / Load / Delete / Download
  // ---------------------------------------------------------------------------
  const handleNew = useCallback(() => {
    setCode("");
    setActiveName(null);
    localStorage.removeItem(LS_ACTIVE_NAME);
    setOutput("");
    setError("");
  }, []);

  const loadScript = useCallback((s: SavedScript) => {
    setCode(s.code);
    setActiveName(s.name);
    localStorage.setItem(LS_ACTIVE_NAME, s.name);
    setOutput("");
    setError("");
  }, []);

  const deleteScript = useCallback(
    (name: string) => {
      const updated = saved.filter((s) => s.name !== name);
      setSaved(updated);
      persistScripts(updated);
      if (activeName === name) {
        setActiveName(null);
        localStorage.removeItem(LS_ACTIVE_NAME);
      }
    },
    [saved, activeName]
  );

  const downloadScript = useCallback((s: SavedScript) => {
    downloadFile(s.name, s.code);
  }, []);

  const downloadCurrent = useCallback(() => {
    const name = activeName || "script";
    downloadFile(name, codeRef.current);
  }, [activeName]);

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------
  const tabTitle = activeName
    ? `${activeName}.rhai`
    : "untitled.rhai";

  return (
    <div className="flex flex-col flex-1 min-h-0 gap-0">
      {/* ── Top toolbar ─────────────────────────────────────────────── */}
      <div className="flex items-center justify-between flex-wrap gap-2 mb-3">
        <div className="flex items-center gap-2">
          <h2 className="text-[0.75rem] font-semibold uppercase tracking-widest text-text-dim">
            Rhai Studio
          </h2>
          <span className="text-[0.72rem] text-text-dim/50 font-mono">{tabTitle}</span>
        </div>
        <div className="flex items-center gap-2 flex-wrap">
          {/* Download current */}
          <button
            type="button"
            onClick={downloadCurrent}
            title="Download current script"
            className="h-8 px-3 text-xs font-medium bg-surface2 text-text-dim border border-border rounded-lg hover:text-text cursor-pointer transition-colors flex items-center gap-1"
          >
            <Download size={13} />
            Download
          </button>

          {/* Save */}
          <button
            type="button"
            onClick={handleSave}
            className="h-8 px-3 text-xs font-medium bg-surface2 text-text-dim border border-border rounded-lg hover:text-text cursor-pointer transition-colors"
            title={activeName ? `Save to "${activeName}" (Ctrl+S)` : "Save script (Ctrl+S)"}
          >
            {activeName ? "Save" : "Save As…"}
          </button>

          <div className="w-px h-5 bg-border" />

          <label className="text-[0.72rem] text-text-dim uppercase tracking-widest">
            Samples
          </label>
          <input
            type="number"
            min={1}
            max={20}
            value={samples}
            onChange={(e) => setSamples(Number(e.target.value))}
            className="w-16 h-8 px-2 text-xs bg-surface2 border border-border rounded-lg text-text focus:outline-none focus:border-accent"
          />

          <button
            type="button"
            onClick={runScript}
            disabled={loading}
            className="h-8 px-4 text-xs font-medium bg-accent/20 text-accent border border-accent/40 rounded-lg hover:bg-accent/30 disabled:opacity-50 cursor-pointer disabled:cursor-default transition-colors"
            title="Run"
          >
            {loading ? "Running…" : "▶ Run"}
          </button>
        </div>
      </div>

      {/* ── Save-as dialog ───────────────────────────────────────────── */}
      {showSaveDialog && (
        <div className="flex items-center gap-2 px-3 py-2 mb-3 bg-surface2 border border-border rounded-lg">
          <span className="text-xs text-text-dim shrink-0">Save as:</span>
          <input
            type="text"
            value={saveName}
            onChange={(e) => setSaveName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") commitSave(saveName);
              if (e.key === "Escape") setShowSaveDialog(false);
            }}
            placeholder="script name…"
            className="flex-1 h-7 px-2 text-xs bg-bg border border-border rounded text-text focus:outline-none focus:border-accent font-mono"
            // biome-ignore lint/a11y/noAutofocus: intentional dialog autofocus
            autoFocus
          />
          <button
            type="button"
            onClick={() => commitSave(saveName)}
            disabled={!saveName.trim()}
            className="h-7 px-3 text-xs font-medium bg-accent/20 text-accent border border-accent/40 rounded hover:bg-accent/30 disabled:opacity-50 cursor-pointer disabled:cursor-default transition-colors"
          >
            Save
          </button>
          <button
            type="button"
            onClick={() => setShowSaveDialog(false)}
            className="h-7 px-2 text-xs text-text-dim hover:text-text cursor-pointer"
          >
            Cancel
          </button>
        </div>
      )}

      {/* ── Main split: sidebar + editor ────────────────────────────── */}
      <div className="flex flex-1 min-h-0 gap-3">
        {/* ── File-tree sidebar ─────────────────────────────────── */}
        <aside className="w-52 shrink-0 flex flex-col bg-surface border border-border rounded-lg overflow-hidden text-[0.72rem]">
          {/* Sidebar header */}
          <div className="flex items-center justify-between px-3 py-2 border-b border-border">
            <span className="uppercase tracking-widest text-text-dim font-semibold">Explorer</span>
            <button
              type="button"
              onClick={handleNew}
              title="New file (Ctrl+N)"
              className="p-0.5 text-text-dim hover:text-accent cursor-pointer transition-colors"
            >
              <Plus size={13} />
            </button>
          </div>

          <div className="flex-1 overflow-y-auto py-1">
            {/* ── Presets section ─────────────────────────────── */}
            <button
              type="button"
              onClick={() => setPresetsOpen((v) => !v)}
              className="w-full flex items-center gap-1 px-2 py-1 text-text-dim/70 hover:text-text-dim cursor-pointer uppercase tracking-wider font-semibold select-none"
            >
              <TreeChevron open={presetsOpen} />
              Presets
            </button>
            {presetsOpen && (
              <ul>
                {PRESETS.map((p) => (
                  <li key={p.name}>
                    <button
                      type="button"
                      onClick={() => {
                        setCode(p.code);
                        setActiveName(null);
                        localStorage.removeItem(LS_ACTIVE_NAME);
                      }}
                      className="w-full flex items-center gap-1.5 px-3 py-1 text-left text-text-dim hover:bg-bg hover:text-text cursor-pointer transition-colors truncate"
                    >
                      <FileText size={13} className="shrink-0 opacity-60" />
                      <span className="truncate">{p.name}</span>
                    </button>
                  </li>
                ))}
              </ul>
            )}

            {/* ── Saved scripts section ───────────────────────── */}
            <button
              type="button"
              onClick={() => setScriptsOpen((v) => !v)}
              className="w-full flex items-center gap-1 px-2 py-1 mt-1 text-text-dim/70 hover:text-text-dim cursor-pointer uppercase tracking-wider font-semibold select-none"
            >
              <TreeChevron open={scriptsOpen} />
              Saved ({saved.length})
            </button>
            {scriptsOpen && (
              <ul>
                {saved.length === 0 && (
                  <li className="px-5 py-1 text-text-dim/50 italic">No scripts yet</li>
                )}
                {saved.map((s) => (
                  <li key={s.name}>
                    <div
                      className={`group flex items-center gap-1 px-3 py-1 cursor-pointer transition-colors ${
                        activeName === s.name
                          ? "bg-accent/15 text-accent"
                          : "text-text-dim hover:bg-bg hover:text-text"
                      }`}
                    >
                      {/* File name (clickable) */}
                      <button
                        type="button"
                        onClick={() => loadScript(s)}
                        className="flex items-center gap-1.5 flex-1 min-w-0 text-left cursor-pointer"
                      >
                        <FileText size={13} className="shrink-0 opacity-60" />
                        <span className="truncate font-mono">{s.name}</span>
                      </button>

                      {/* Action buttons – visible on hover or when active */}
                      <div className={`flex items-center gap-0.5 shrink-0 ${activeName === s.name ? "opacity-100" : "opacity-0 group-hover:opacity-100"} transition-opacity`}>
                        <button
                          type="button"
                          onClick={() => downloadScript(s)}
                          title={`Download ${s.name}.rhai`}
                          className="p-0.5 hover:text-accent cursor-pointer"
                        >
                          <Download size={13} />
                        </button>
                        <button
                          type="button"
                          onClick={() => deleteScript(s.name)}
                          title="Delete"
                          className="p-0.5 hover:text-red cursor-pointer"
                        >
                          <Trash2 size={13} />
                        </button>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </aside>

        {/* ── Editor + Output ──────────────────────────────────── */}
        <div className="flex flex-col flex-1 min-h-0 gap-3">
          <div className="flex-[2] min-h-[200px]">
            <RhaiEditor
              value={code}
              onChange={setCode}
              onSave={handleSave}
              onNew={handleNew}
            />
          </div>

          {/* Output panel */}
          <div className="flex-1 flex flex-col min-h-[140px]">
            {error && (
              <div className="text-xs text-red mb-2 px-1">{error}</div>
            )}
            <pre className="flex-1 border border-border rounded-lg bg-bg text-green p-3 text-[0.76rem] leading-relaxed whitespace-pre-wrap break-all overflow-auto">
              {output || "Click ▶ Run to execute · Ctrl+S to save · Ctrl+N for new file"}
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
}

