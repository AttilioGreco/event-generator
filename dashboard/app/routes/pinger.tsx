import { useCallback, useEffect, useState } from "react";
import { Loader2, Send } from "lucide-react";

import { cn } from "~/lib/utils";

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "~/components/ui/accordion";
import { Alert, AlertDescription } from "~/components/ui/alert";
import { Button } from "~/components/ui/button";
import { Card, CardContent } from "~/components/ui/card";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "~/components/ui/select";
import { Textarea } from "~/components/ui/textarea";
import { KeyValueList, type HeaderPair } from "~/components/key-value-list";
import { LuaScriptField, type PingResult } from "~/components/lua-script-field";

// ── Types ──────────────────────────────────────────────────────────────────

type FormatType =
  | "syslog_rfc5424"
  | "syslog_rfc3164"
  | "cef"
  | "leef"
  | "clf"
  | "json"
  | "java_log4j"
  | "java_logback"
  | "template"
  | "script";

type OutputType = "tcp" | "udp" | "http";

// ── Constants ──────────────────────────────────────────────────────────────

const STORAGE_KEY = "pinger_config_v2";

const FORMAT_LABELS: Record<FormatType, string> = {
  syslog_rfc5424: "Syslog RFC 5424",
  syslog_rfc3164: "Syslog RFC 3164",
  cef: "CEF",
  leef: "LEEF",
  clf: "CLF (Common Log)",
  json: "JSON",
  java_log4j: "Java Log4j",
  java_logback: "Java Logback",
  template: "Template (Tera)",
  script: "Script (Lua)",
};

const SYSLOG_FACILITIES = [
  "kern", "user", "mail", "daemon", "auth", "syslog",
  "lpr", "news", "uucp", "cron",
  "local0", "local1", "local2", "local3", "local4", "local5", "local6", "local7",
];

const SYSLOG_SEVERITIES = [
  "emerg", "alert", "crit", "err", "warning", "notice", "info", "debug",
];

function defaultMessage(): string {
  return new Date().toTimeString().slice(0, 8) + " test event";
}

// ── Persistence ────────────────────────────────────────────────────────────

interface StoredConfig {
  formatType: FormatType;
  facility: string;
  severity: string;
  appName: string;
  vendor: string;
  product: string;
  version: string;
  deviceEventClassId: string;
  extraFields: HeaderPair[];
  templateInline: string;
  scriptInline: string;
  outputType: OutputType;
  host: string;
  port: string;
  httpUrl: string;
  httpMethod: string;
  httpHeaders: HeaderPair[];
  message: string;
  accordionOpen: string[];
}

const DEFAULT_CONFIG: StoredConfig = {
  formatType: "syslog_rfc5424",
  facility: "local0",
  severity: "info",
  appName: "event-generator",
  vendor: "Acme",
  product: "Firewall",
  version: "1.0",
  deviceEventClassId: "100",
  extraFields: [],
  templateInline: "{{ timestamp_iso() }} {{ fake_hostname() }} {{ fields.message }}\n",
  scriptInline: 'emit(now_iso() .. " INFO " .. fake_hostname() .. " " .. fake_message())',
  outputType: "tcp",
  host: "",
  port: "514",
  httpUrl: "",
  httpMethod: "POST",
  httpHeaders: [{ key: "Content-Type", value: "application/json" }],
  message: defaultMessage(),
  accordionOpen: ["format", "output"],
};

function loadConfig(): StoredConfig {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return { ...DEFAULT_CONFIG, ...JSON.parse(raw) };
  } catch {
    // ignore
  }
  return DEFAULT_CONFIG;
}

function saveConfig(cfg: Partial<StoredConfig>) {
  try {
    const existing = localStorage.getItem(STORAGE_KEY);
    const current = existing ? JSON.parse(existing) : {};
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ ...current, ...cfg }));
  } catch {
    // ignore
  }
}

// ── Component ──────────────────────────────────────────────────────────────

export default function PingerPage() {
  const [initialized, setInitialized] = useState(false);

  // Format state
  const [formatType, setFormatType] = useState<FormatType>("syslog_rfc5424");
  const [facility, setFacility] = useState("local0");
  const [severity, setSeverity] = useState("info");
  const [appName, setAppName] = useState("event-generator");
  const [vendor, setVendor] = useState("Acme");
  const [product, setProduct] = useState("Firewall");
  const [version, setVersion] = useState("1.0");
  const [deviceEventClassId, setDeviceEventClassId] = useState("100");
  const [extraFields, setExtraFields] = useState<HeaderPair[]>([]);
  const [templateInline, setTemplateInline] = useState(
    "{{ timestamp_iso() }} {{ fake_hostname() }} {{ fields.message }}\n"
  );
  const [scriptInline, setScriptInline] = useState(
    'emit(now_iso() .. " INFO " .. fake_hostname() .. " " .. fake_message())'
  );

  // Output state
  const [outputType, setOutputType] = useState<OutputType>("tcp");
  const [host, setHost] = useState("");
  const [port, setPort] = useState("514");
  const [httpUrl, setHttpUrl] = useState("");
  const [httpMethod, setHttpMethod] = useState("POST");
  const [httpHeaders, setHttpHeaders] = useState<HeaderPair[]>([
    { key: "Content-Type", value: "application/json" },
  ]);

  // Message + result
  const [message, setMessage] = useState(defaultMessage);
  const [result, setResult] = useState<PingResult | null>(null);
  const [resultKey, setResultKey] = useState(0);
  const [loading, setLoading] = useState(false);

  // Accordion open state
  const [accordionOpen, setAccordionOpen] = useState<string[]>(["format", "output"]);

  // ── Persistence ───────────────────────────────────────────────────────

  useEffect(() => {
    const cfg = loadConfig();
    setFormatType(cfg.formatType);
    setFacility(cfg.facility);
    setSeverity(cfg.severity);
    setAppName(cfg.appName);
    setVendor(cfg.vendor);
    setProduct(cfg.product);
    setVersion(cfg.version);
    setDeviceEventClassId(cfg.deviceEventClassId);
    setExtraFields(cfg.extraFields);
    setTemplateInline(cfg.templateInline);
    setScriptInline(cfg.scriptInline);
    setOutputType(cfg.outputType);
    setHost(cfg.host);
    setPort(cfg.port);
    setHttpUrl(cfg.httpUrl);
    setHttpMethod(cfg.httpMethod);
    setHttpHeaders(cfg.httpHeaders);
    setMessage(cfg.message);
    setAccordionOpen(cfg.accordionOpen);
    setInitialized(true);
  }, []);

  useEffect(() => {
    if (!initialized) return;
    saveConfig({
      formatType, facility, severity, appName,
      vendor, product, version, deviceEventClassId,
      extraFields, templateInline, scriptInline,
      outputType, host, port, httpUrl, httpMethod, httpHeaders,
      message, accordionOpen,
    });
  }, [
    initialized,
    formatType, facility, severity, appName,
    vendor, product, version, deviceEventClassId,
    extraFields, templateInline, scriptInline,
    outputType, host, port, httpUrl, httpMethod, httpHeaders,
    message, accordionOpen,
  ]);

  // ── Build request body ────────────────────────────────────────────────

  const buildFormatConfig = useCallback(() => {
    const base = { type: formatType };
    switch (formatType) {
      case "syslog_rfc5424":
      case "syslog_rfc3164":
        return { ...base, facility, severity, app_name: appName };
      case "cef":
      case "leef":
        return { ...base, vendor, product, version, device_event_class_id: deviceEventClassId };
      case "json": {
        const extra = Object.fromEntries(
          extraFields.filter((h) => h.key.trim()).map((h) => [h.key, h.value])
        );
        return { ...base, extra_fields: Object.keys(extra).length ? extra : undefined };
      }
      case "template":
        return { ...base, template_inline: templateInline };
      case "script":
        return { ...base, script_inline: scriptInline };
      default:
        return base;
    }
  }, [
    formatType, facility, severity, appName,
    vendor, product, version, deviceEventClassId,
    extraFields, templateInline, scriptInline,
  ]);

  const buildOutputConfig = useCallback(() => {
    if (outputType === "tcp" || outputType === "udp") {
      return { type: outputType, host, port: Number(port) };
    }
    const headers = Object.fromEntries(
      httpHeaders.filter((h) => h.key.trim()).map((h) => [h.key, h.value])
    );
    return {
      type: "http",
      url: httpUrl,
      method: httpMethod,
      headers: Object.keys(headers).length ? headers : undefined,
    };
  }, [outputType, host, port, httpUrl, httpMethod, httpHeaders]);

  // ── Validation + run ──────────────────────────────────────────────────

  const validate = useCallback((): string | null => {
    if (outputType === "tcp" || outputType === "udp") {
      if (!host.trim()) return "Host is required";
      const p = Number(port);
      if (!port || isNaN(p) || p < 1 || p > 65535) return "Port must be between 1 and 65535";
    }
    if (outputType === "http") {
      if (!httpUrl.trim()) return "URL is required";
      if (!/^https?:\/\//i.test(httpUrl)) return "URL must start with http:// or https://";
    }
    if (formatType === "template" && !templateInline.trim()) return "Template content is required";
    if (formatType === "script" && !scriptInline.trim()) return "Script content is required";
    return null;
  }, [outputType, host, port, httpUrl, formatType, templateInline, scriptInline]);

  const runTest = useCallback(async () => {
    const validationError = validate();
    if (validationError) {
      setResult({ success: false, error: validationError, elapsed_ms: 0 });
      setResultKey((k) => k + 1);
      return;
    }
    const currentMessage = message.trim() || defaultMessage();
    setLoading(true);
    try {
      const res = await fetch("/api/ping", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          format: buildFormatConfig(),
          output: buildOutputConfig(),
          message: currentMessage,
        }),
      });
      const data: PingResult = await res.json();
      setResult(data);
      setResultKey((k) => k + 1);
    } catch (e) {
      setResult({ success: false, error: `Request failed: ${e}`, elapsed_ms: 0 });
      setResultKey((k) => k + 1);
    } finally {
      setLoading(false);
    }
  }, [validate, message, buildFormatConfig, buildOutputConfig]);

  // ── Render ────────────────────────────────────────────────────────────

  return (
    <div className="flex flex-col gap-4 max-w-3xl mx-auto w-full">
      <h2 className="text-lg font-semibold">Pinger</h2>

      <Accordion
        value={accordionOpen}
        onValueChange={(v) => setAccordionOpen(v as string[])}
        multiple
        className="border rounded-xl overflow-hidden"
      >
        {/* ── Format ── */}
        <AccordionItem value="format" className="px-4">
          <AccordionTrigger className="text-sm font-medium text-muted-foreground uppercase tracking-widest">
            Format
          </AccordionTrigger>
          <AccordionContent>
            <div className="flex flex-col gap-4 pb-2">
              <div className="flex flex-col gap-1.5">
                <Label>Format type</Label>
                <Select value={formatType} onValueChange={(v) => setFormatType(v as FormatType)}>
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {(Object.keys(FORMAT_LABELS) as FormatType[]).map((k) => (
                      <SelectItem key={k} value={k}>{FORMAT_LABELS[k]}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {(formatType === "syslog_rfc5424" || formatType === "syslog_rfc3164") && (
                <div className="grid grid-cols-2 gap-3">
                  <div className="flex flex-col gap-1.5">
                    <Label>Facility</Label>
                    <Select value={facility} onValueChange={(v) => v && setFacility(v)}>
                      <SelectTrigger className="w-full"><SelectValue /></SelectTrigger>
                      <SelectContent>
                        {SYSLOG_FACILITIES.map((f) => (
                          <SelectItem key={f} value={f}>{f}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="flex flex-col gap-1.5">
                    <Label>Severity</Label>
                    <Select value={severity} onValueChange={(v) => v && setSeverity(v)}>
                      <SelectTrigger className="w-full"><SelectValue /></SelectTrigger>
                      <SelectContent>
                        {SYSLOG_SEVERITIES.map((s) => (
                          <SelectItem key={s} value={s}>{s}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="col-span-2 flex flex-col gap-1.5">
                    <Label>App name</Label>
                    <Input value={appName} onChange={(e) => setAppName(e.target.value)} />
                  </div>
                </div>
              )}

              {(formatType === "cef" || formatType === "leef") && (
                <div className="grid grid-cols-2 gap-3">
                  <div className="flex flex-col gap-1.5">
                    <Label>Vendor</Label>
                    <Input value={vendor} onChange={(e) => setVendor(e.target.value)} />
                  </div>
                  <div className="flex flex-col gap-1.5">
                    <Label>Product</Label>
                    <Input value={product} onChange={(e) => setProduct(e.target.value)} />
                  </div>
                  <div className="flex flex-col gap-1.5">
                    <Label>Version</Label>
                    <Input value={version} onChange={(e) => setVersion(e.target.value)} />
                  </div>
                  <div className="flex flex-col gap-1.5">
                    <Label>Device event class ID</Label>
                    <Input value={deviceEventClassId} onChange={(e) => setDeviceEventClassId(e.target.value)} />
                  </div>
                </div>
              )}

              {formatType === "json" && (
                <div className="flex flex-col gap-2">
                  <Label>Extra fields <span className="text-muted-foreground font-normal">(optional)</span></Label>
                  <KeyValueList pairs={extraFields} onChange={setExtraFields} keyPlaceholder="field" valuePlaceholder="value" />
                </div>
              )}

              {formatType === "template" && (
                <div className="flex flex-col gap-1.5">
                  <Label>Template (Tera)</Label>
                  <Textarea
                    value={templateInline}
                    onChange={(e) => setTemplateInline(e.target.value)}
                    rows={4}
                    className="font-mono text-xs"
                    placeholder="{{ timestamp_iso() }} {{ fake_hostname() }} {{ fields.message }}"
                  />
                </div>
              )}

              {formatType === "script" && (
                <LuaScriptField
                  value={scriptInline}
                  onChange={setScriptInline}
                  onRun={runTest}
                  loading={loading}
                  result={result}
                />
              )}
            </div>
          </AccordionContent>
        </AccordionItem>

        {/* ── Output ── */}
        <AccordionItem value="output" className="px-4">
          <AccordionTrigger className="text-sm font-medium text-muted-foreground uppercase tracking-widest">
            Output
          </AccordionTrigger>
          <AccordionContent>
            <div className="flex flex-col gap-4 pb-2">
              <div className="flex flex-col gap-1.5">
                <Label>Transport</Label>
                <Select value={outputType} onValueChange={(v) => setOutputType(v as OutputType)}>
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="tcp">TCP</SelectItem>
                    <SelectItem value="udp">UDP</SelectItem>
                    <SelectItem value="http">HTTP</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {(outputType === "tcp" || outputType === "udp") && (
                <div className="grid grid-cols-[1fr_120px] gap-3">
                  <div className="flex flex-col gap-1.5">
                    <Label>Host</Label>
                    <Input value={host} onChange={(e) => setHost(e.target.value)} placeholder="192.168.1.10" />
                  </div>
                  <div className="flex flex-col gap-1.5">
                    <Label>Port</Label>
                    <Input type="number" min={1} max={65535} value={port} onChange={(e) => setPort(e.target.value)} placeholder="514" />
                  </div>
                </div>
              )}

              {outputType === "http" && (
                <>
                  <div className="grid grid-cols-[1fr_120px] gap-3">
                    <div className="flex flex-col gap-1.5">
                      <Label>URL</Label>
                      <Input value={httpUrl} onChange={(e) => setHttpUrl(e.target.value)} placeholder="https://collector.example.com/events" />
                    </div>
                    <div className="flex flex-col gap-1.5">
                      <Label>Method</Label>
                      <Select value={httpMethod} onValueChange={(v) => v && setHttpMethod(v)}>
                        <SelectTrigger className="w-full"><SelectValue /></SelectTrigger>
                        <SelectContent>
                          {["POST", "PUT", "PATCH", "GET"].map((m) => (
                            <SelectItem key={m} value={m}>{m}</SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                  <div className="flex flex-col gap-2">
                    <Label>Headers</Label>
                    <KeyValueList pairs={httpHeaders} onChange={setHttpHeaders} keyPlaceholder="Header-Name" valuePlaceholder="value" />
                  </div>
                </>
              )}
            </div>
          </AccordionContent>
        </AccordionItem>
      </Accordion>

      {/* ── Message + send ── */}
      <Card>
        <CardContent className="pt-4 flex flex-col gap-3">
          {formatType !== "script" && (
            <div className="flex flex-col gap-1.5">
              <Label>Message</Label>
              <Textarea
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                rows={2}
                className="font-mono text-xs"
                placeholder={defaultMessage()}
              />
              <p className="text-xs text-muted-foreground">
                Injected as the <code className="text-primary">message</code> field in the formatted event.
              </p>
            </div>
          )}
          <Button onClick={runTest} disabled={loading} className="w-fit gap-2">
            {loading ? <Loader2 size={14} className="animate-spin" /> : <Send size={14} />}
            Send test event
          </Button>
        </CardContent>
      </Card>

      {/* ── Result ── */}
      <div className="flex flex-col gap-2">
        {!result && (
          <div className={cn(
            "flex items-center justify-center border border-dashed border-border rounded-xl text-muted-foreground text-sm py-12 transition-opacity duration-300",
            loading && "opacity-40"
          )}>
            {loading ? "Sending…" : 'Hit "Send test event" to see the result'}
          </div>
        )}

        {result && (
          <div
            key={resultKey}
            className={cn(
              "flex flex-col gap-3 animate-in fade-in duration-300",
              loading && "opacity-50 pointer-events-none transition-opacity"
            )}
          >
            <Alert variant={result.success ? "default" : "destructive"}>
              <AlertDescription className="flex items-center justify-between">
                <span>
                  {result.success ? "✓ OK" : "✗ Failed"}
                  {result.error ? ` — ${result.error}` : ""}
                </span>
                <span className="text-xs text-muted-foreground ml-4 shrink-0">
                  {result.elapsed_ms} ms
                </span>
              </AlertDescription>
            </Alert>

            {result.event && (
              <div className="flex flex-col gap-1.5">
                <Label className="text-muted-foreground">Formatted event sent</Label>
                <pre className="border border-border rounded-xl bg-background text-green-400 p-4 text-[0.76rem] leading-relaxed whitespace-pre-wrap break-all overflow-auto">
                  {result.event}
                </pre>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
