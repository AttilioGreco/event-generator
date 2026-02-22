import { useCallback, useEffect, useRef, useState } from "react";
import { ChevronRight, Download, FileText, Play, Plus, Trash2 } from "lucide-react";

import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";
import { ScrollArea } from "~/components/ui/scroll-area";
import { Separator } from "~/components/ui/separator";
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
  { name: "Java Request Trace", code: DEFAULT_SCRIPT },
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

function downloadFile(name: string, content: string) {
  const blob = new Blob([content], { type: "text/plain" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = name.endsWith(".rhai") ? name : `${name}.rhai`;
  a.click();
  URL.revokeObjectURL(url);
}

const TreeChevron = ({ open }: { open: boolean }) => (
  <ChevronRight
    size={12}
    className={`shrink-0 transition-transform duration-150 ${open ? "rotate-90" : ""}`}
  />
);

export default function StudioPage() {
  const [code, setCode] = useState(
    () => localStorage.getItem(LS_ACTIVE) || DEFAULT_SCRIPT
  );
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

  useEffect(() => {
    localStorage.setItem(LS_ACTIVE, code);
  }, [code]);

  const runScript = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const res = await fetch("/api/script/run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ code: codeRef.current, samples: Math.max(1, Math.min(20, samples)) }),
      });
      if (!res.ok) { setError(`Request failed: ${res.status}`); setOutput(""); return; }
      const data = await res.json();
      setError(data.error || "");
      setOutput(Array.isArray(data.output) ? data.output.join("\n---\n") : "");
    } catch {
      setError("Request failed");
      setOutput("");
    } finally {
      setLoading(false);
    }
  }, [samples]);

  const commitSave = useCallback((name: string) => {
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
  }, [code, saved]);

  const handleSave = useCallback(() => {
    if (activeName) commitSave(activeName);
    else { setSaveName(""); setShowSaveDialog(true); }
  }, [activeName, commitSave]);

  const handleNew = useCallback(() => {
    setCode(""); setActiveName(null);
    localStorage.removeItem(LS_ACTIVE_NAME);
    setOutput(""); setError("");
  }, []);

  const loadScript = useCallback((s: SavedScript) => {
    setCode(s.code); setActiveName(s.name);
    localStorage.setItem(LS_ACTIVE_NAME, s.name);
    setOutput(""); setError("");
  }, []);

  const deleteScript = useCallback((name: string) => {
    const updated = saved.filter((s) => s.name !== name);
    setSaved(updated);
    persistScripts(updated);
    if (activeName === name) {
      setActiveName(null);
      localStorage.removeItem(LS_ACTIVE_NAME);
    }
  }, [saved, activeName]);

  const tabTitle = activeName ? `${activeName}.rhai` : "untitled.rhai";

  return (
    <div className="flex flex-col flex-1 min-h-0 gap-0">
      {/* Toolbar */}
      <div className="flex items-center justify-between flex-wrap gap-2 mb-3">
        <div className="flex items-center gap-2">
          <h2 className="text-[0.75rem] font-semibold uppercase tracking-widest text-muted-foreground">
            Rhai Studio
          </h2>
          <span className="text-[0.72rem] text-muted-foreground/50 font-mono">{tabTitle}</span>
        </div>
        <div className="flex items-center gap-2 flex-wrap">
          <Button variant="secondary" size="sm" onClick={() => downloadFile(activeName || "script", codeRef.current)} className="gap-1.5">
            <Download size={13} />
            Download
          </Button>
          <Button variant="secondary" size="sm" onClick={handleSave} title={activeName ? `Save to "${activeName}" (Ctrl+S)` : "Save script (Ctrl+S)"}>
            {activeName ? "Save" : "Save As…"}
          </Button>
          <Separator orientation="vertical" className="h-5" />
          <span className="text-[0.72rem] text-muted-foreground uppercase tracking-widest">
            Samples
          </span>
          <Input
            type="number"
            min={1}
            max={20}
            value={samples}
            onChange={(e) => setSamples(Number(e.target.value))}
            className="w-16 h-8 text-xs"
          />
          <Button onClick={runScript} disabled={loading} size="sm" className="gap-1.5">
            <Play size={13} />
            {loading ? "Running…" : "Run"}
          </Button>
        </div>
      </div>

      {/* Save-as inline dialog */}
      {showSaveDialog && (
        <div className="flex items-center gap-2 px-3 py-2 mb-3 bg-secondary border border-border rounded-lg">
          <span className="text-xs text-muted-foreground shrink-0">Save as:</span>
          <Input
            value={saveName}
            onChange={(e) => setSaveName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") commitSave(saveName);
              if (e.key === "Escape") setShowSaveDialog(false);
            }}
            placeholder="script name…"
            className="flex-1 h-7 text-xs font-mono"
            // biome-ignore lint/a11y/noAutofocus: intentional dialog autofocus
            autoFocus
          />
          <Button size="sm" className="h-7 px-3 text-xs" onClick={() => commitSave(saveName)} disabled={!saveName.trim()}>
            Save
          </Button>
          <Button variant="ghost" size="sm" className="h-7 px-2 text-xs" onClick={() => setShowSaveDialog(false)}>
            Cancel
          </Button>
        </div>
      )}

      {/* Main split: sidebar + editor */}
      <div className="flex flex-1 min-h-0 gap-3">
        {/* Sidebar */}
        <aside className="w-52 shrink-0 flex flex-col bg-card border border-border rounded-lg overflow-hidden text-[0.72rem]">
          <div className="flex items-center justify-between px-3 py-2 border-b border-border">
            <span className="uppercase tracking-widest text-muted-foreground font-semibold">Explorer</span>
            <Button variant="ghost" size="icon" className="w-6 h-6" onClick={handleNew} title="New file (Ctrl+N)">
              <Plus size={13} />
            </Button>
          </div>

          <ScrollArea className="flex-1">
            <div className="py-1">
              {/* Presets */}
              <button
                type="button"
                onClick={() => setPresetsOpen((v) => !v)}
                className="w-full flex items-center gap-1 px-2 py-1 text-muted-foreground/70 hover:text-muted-foreground cursor-pointer uppercase tracking-wider font-semibold select-none"
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
                        onClick={() => { setCode(p.code); setActiveName(null); localStorage.removeItem(LS_ACTIVE_NAME); }}
                        className="w-full flex items-center gap-1.5 px-3 py-1 text-left text-muted-foreground hover:bg-accent hover:text-foreground cursor-pointer transition-colors truncate"
                      >
                        <FileText size={13} className="shrink-0 opacity-60" />
                        <span className="truncate">{p.name}</span>
                      </button>
                    </li>
                  ))}
                </ul>
              )}

              {/* Saved scripts */}
              <button
                type="button"
                onClick={() => setScriptsOpen((v) => !v)}
                className="w-full flex items-center gap-1 px-2 py-1 mt-1 text-muted-foreground/70 hover:text-muted-foreground cursor-pointer uppercase tracking-wider font-semibold select-none"
              >
                <TreeChevron open={scriptsOpen} />
                Saved ({saved.length})
              </button>
              {scriptsOpen && (
                <ul>
                  {saved.length === 0 && (
                    <li className="px-5 py-1 text-muted-foreground/50 italic">No scripts yet</li>
                  )}
                  {saved.map((s) => (
                    <li key={s.name}>
                      <div className={`group flex items-center gap-1 px-3 py-1 cursor-pointer transition-colors ${activeName === s.name ? "bg-primary/15 text-primary" : "text-muted-foreground hover:bg-accent hover:text-foreground"}`}>
                        <button
                          type="button"
                          onClick={() => loadScript(s)}
                          className="flex items-center gap-1.5 flex-1 min-w-0 text-left cursor-pointer"
                        >
                          <FileText size={13} className="shrink-0 opacity-60" />
                          <span className="truncate font-mono">{s.name}</span>
                        </button>
                        <div className={`flex items-center gap-0.5 shrink-0 ${activeName === s.name ? "opacity-100" : "opacity-0 group-hover:opacity-100"} transition-opacity`}>
                          <button type="button" onClick={() => downloadFile(s.name, s.code)} title={`Download ${s.name}.rhai`} className="p-0.5 hover:text-primary cursor-pointer">
                            <Download size={13} />
                          </button>
                          <button type="button" onClick={() => deleteScript(s.name)} title="Delete" className="p-0.5 hover:text-destructive cursor-pointer">
                            <Trash2 size={13} />
                          </button>
                        </div>
                      </div>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </ScrollArea>
        </aside>

        {/* Editor + Output */}
        <div className="flex flex-col flex-1 min-h-0 gap-3">
          <div className="flex-[2] min-h-[200px]">
            <RhaiEditor value={code} onChange={setCode} onSave={handleSave} onNew={handleNew} />
          </div>
          <div className="flex-1 flex flex-col min-h-[140px]">
            {error && <p className="text-xs text-destructive mb-2 px-1">{error}</p>}
            <pre className="flex-1 border border-border rounded-lg bg-background text-green-400 p-3 text-[0.76rem] leading-relaxed whitespace-pre-wrap break-all overflow-auto">
              {output || "Click ▶ Run to execute · Ctrl+S to save · Ctrl+N for new file"}
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
}
