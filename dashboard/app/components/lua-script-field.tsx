import { lazy, Suspense, useState } from "react";
import { Loader2, Maximize, Play, X } from "lucide-react";

import { cn } from "~/lib/utils";
import { Button } from "~/components/ui/button";
import { Label } from "~/components/ui/label";
import { EditorSkeleton } from "~/components/editor-skeleton";

// Lazy-load CodeMirror so it doesn't bloat the main bundle when script
// format is not selected.
const LuaEditor = lazy(() =>
  import("~/components/lua-editor").then((m) => ({ default: m.LuaEditor }))
);

export interface PingResult {
  success: boolean;
  event?: string;
  error?: string;
  elapsed_ms: number;
}

interface LuaScriptFieldProps {
  value: string;
  onChange: (value: string) => void;
  /** Called when the user clicks Run or presses Ctrl+Enter */
  onRun: () => void;
  loading: boolean;
  result: PingResult | null;
}

export function LuaScriptField({
  value,
  onChange,
  onRun,
  loading,
  result,
}: LuaScriptFieldProps) {
  const [fullscreen, setFullscreen] = useState(false);

  return (
    <>
      {/* Inline editor */}
      <div className="flex flex-col gap-1.5">
        <div className="flex items-center justify-between">
          <Label>Script (Lua)</Label>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 text-muted-foreground hover:text-foreground"
            onClick={() => setFullscreen(true)}
            title="Expand editor"
          >
            <Maximize size={13} />
          </Button>
        </div>
        <div className="h-48">
          <Suspense fallback={<EditorSkeleton />}>
            <LuaEditor value={value} onChange={onChange} onRun={onRun} />
          </Suspense>
        </div>
      </div>

      {/* Fullscreen overlay */}
      {fullscreen && (
        <div className="fixed inset-0 z-50 bg-background flex flex-col">
          {/* Header */}
          <div className="flex items-center justify-between px-4 py-2 border-b border-border shrink-0">
            <span className="text-sm font-medium text-muted-foreground">
              Script (Lua)
            </span>
            <div className="flex items-center gap-2">
              <Button
                size="sm"
                onClick={onRun}
                disabled={loading}
                className="gap-1.5 h-7 px-3 text-xs"
                title="Send test event (Ctrl+Enter)"
              >
                {loading
                  ? <Loader2 size={12} className="animate-spin" />
                  : <Play size={12} />}
                {loading ? "Sending…" : "Run"}
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setFullscreen(false)}
                title="Close fullscreen"
              >
                <X size={16} />
              </Button>
            </div>
          </div>

          {/* Editor — 3/4 */}
          <div className="flex-[3] min-h-0">
            <Suspense fallback={<EditorSkeleton />}>
              <LuaEditor value={value} onChange={onChange} onRun={onRun} />
            </Suspense>
          </div>

          {/* Output — 1/4 */}
          <div className="flex-[1] min-h-0 border-t border-border flex flex-col overflow-hidden">
            <div className="flex items-center justify-between px-4 py-1.5 shrink-0 border-b border-border/50">
              <span className="text-[0.7rem] uppercase tracking-widest text-muted-foreground font-semibold">
                Output
              </span>
              {result && (
                <span className={cn(
                  "text-[0.7rem] font-mono",
                  result.success ? "text-green-400" : "text-destructive"
                )}>
                  {result.success ? "✓ OK" : "✗ Failed"}
                  {result.error ? ` — ${result.error}` : ""}
                  <span className="text-muted-foreground ml-2">{result.elapsed_ms} ms</span>
                </span>
              )}
            </div>
            {result?.event ? (
              <pre className="flex-1 bg-background text-green-400 px-4 py-3 text-[0.76rem] leading-relaxed whitespace-pre-wrap break-all overflow-auto">
                {result.event}
              </pre>
            ) : (
              <div className="flex-1 flex items-center justify-center text-sm">
                {result?.error
                  ? <span className="text-destructive text-xs px-4">{result.error}</span>
                  : <span className="text-muted-foreground">Send a test event to see the output here</span>}
              </div>
            )}
          </div>
        </div>
      )}
    </>
  );
}
