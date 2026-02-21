import { useCallback, useEffect, useState } from "react";

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
        setMessage({
          type: "success",
          text: "Configuration applied successfully. Streams restarted.",
        });
      } else {
        setMessage({
          type: "error",
          text: data.error || "Unknown error",
        });
      }
    } catch (e) {
      setMessage({ type: "error", text: `Request failed: ${e}` });
    } finally {
      setSaving(false);
    }
  }, [config]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64 text-text-dim text-sm">
        Loading configuration...
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4 h-full min-h-0">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Configuration</h2>
        <button
          type="button"
          onClick={handleSave}
          disabled={saving}
          className="h-8 px-4 flex items-center gap-2 text-xs font-medium border border-accent rounded-lg bg-accent/10 text-accent hover:bg-accent/20 transition-colors disabled:opacity-50 cursor-pointer disabled:cursor-not-allowed"
        >
          {saving ? "Applying..." : "Validate & Apply"}
        </button>
      </div>

      {message && (
        <div
          className={`px-4 py-2.5 rounded-lg text-sm border ${
            message.type === "success"
              ? "bg-green/10 border-green/30 text-green"
              : "bg-red/10 border-red/30 text-red"
          }`}
        >
          {message.text}
        </div>
      )}

      <div className="flex-1 min-h-0">
        <ConfigEditor
          value={config}
          onChange={setConfig}
          onSave={handleSave}
        />
      </div>
    </div>
  );
}
