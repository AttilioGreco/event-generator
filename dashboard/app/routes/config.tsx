import { useCallback, useEffect, useState } from "react";

import { Alert, AlertDescription } from "~/components/ui/alert";
import { Button } from "~/components/ui/button";
import { ConfigEditor } from "~/components/config-editor";

export default function ConfigPage() {
  const [config, setConfig] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  useEffect(() => {
    fetch("/api/config")
      .then((r) => r.text())
      .then((text) => {
        setConfig(text);
        setLoading(false);
      })
      .catch((e) => {
        setMessage({ type: "error", text: `Failed to load config: ${e}` });
        setLoading(false);
      });
  }, []);

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

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64 text-muted-foreground text-sm">
        Loading configuration…
      </div>
    );
  }

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
        <ConfigEditor value={config} onChange={setConfig} onSave={handleSave} />
      </div>
    </div>
  );
}
