import { lazy, Suspense, useCallback, useState } from "react";

import { Alert, AlertDescription } from "~/components/ui/alert";
import { Button } from "~/components/ui/button";
import type { Route } from "./+types/config";

// Lazy-load ConfigEditor so the CodeMirror bundle is a separate async chunk.
// The editor only loads when this route actually renders, not at app startup.
const ConfigEditor = lazy(() =>
  import("~/components/config-editor").then((m) => ({ default: m.ConfigEditor }))
);

// ── Loader ────────────────────────────────────────────────────────────────
// clientLoader runs in parallel with the route chunk download, eliminating
// the render-then-fetch waterfall of the old useEffect approach.

export async function clientLoader(): Promise<{ config: string; loadError: string | null }> {
  try {
    const res = await fetch("/api/config");
    if (!res.ok) return { config: "", loadError: `HTTP ${res.status}` };
    return { config: await res.text(), loadError: null };
  } catch (e) {
    return { config: "", loadError: `Failed to load config: ${e}` };
  }
}

// ── Editor skeleton ───────────────────────────────────────────────────────

function EditorSkeleton() {
  return (
    <div className="h-full w-full rounded-md border border-border bg-[#282c34] animate-pulse" />
  );
}

// ── Page ─────────────────────────────────────────────────────────────────

export default function ConfigPage({ loaderData }: Route.ComponentProps) {
  const [config, setConfig] = useState(loaderData.config);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(loaderData.loadError ? { type: "error", text: loaderData.loadError } : null);

  const handleSave = useCallback(async () => {
    setSaving(true);
    setMessage(null);
    try {
      const res = await fetch("/api/config", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ config }),
      });
      const data = await res.json();
      if (data.success) {
        setMessage({ type: "success", text: "Configuration applied. Streams restarted." });
      } else {
        setMessage({ type: "error", text: data.error || "Unknown error" });
      }
    } catch (e) {
      setMessage({ type: "error", text: `Request failed: ${e}` });
    } finally {
      setSaving(false);
    }
  }, [config]);

  return (
    <div className="flex flex-col gap-4 h-full min-h-0">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Configuration</h2>
        <Button onClick={handleSave} disabled={saving} size="sm">
          {saving ? "Applying…" : "Validate & Apply"}
        </Button>
      </div>

      {message && (
        <Alert variant={message.type === "error" ? "destructive" : "default"}>
          <AlertDescription>{message.text}</AlertDescription>
        </Alert>
      )}

      <div className="flex-1 min-h-0">
        <Suspense fallback={<EditorSkeleton />}>
          <ConfigEditor value={config} onChange={setConfig} onSave={handleSave} />
        </Suspense>
      </div>
    </div>
  );
}
