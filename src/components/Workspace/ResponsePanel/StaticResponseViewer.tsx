import { useState, useMemo } from "react";
import { AlertCircle, CheckCircle2, XCircle, Terminal, Sparkles } from "lucide-react";
import CodeEditor from "../../CodeEditor";
import ResponseMetricsBar from "./ResponseMetricsBar";
import { RequestTab } from "../../../types";
import { CookieRow } from "@veyak-internal/models";
import { useVartaStore } from "../../../store/vartaStore";

interface StaticResponseViewerProps {
  tab: RequestTab;
  isMobile?: boolean;
}

type SubTab = "body" | "headers" | "cookies" | "scripts";

export default function StaticResponseViewer({
  tab,
  isMobile = false,
}: StaticResponseViewerProps) {
  const [activeSubTab, setActiveSubTab] = useState<SubTab>("body");
  const [pretty, setPretty] = useState(true);
  const [copied, setCopied] = useState(false);

  const grpcResponseMetadata = useVartaStore((s) => s.grpcResponseMetadata);
  const grpcResponseStatus = useVartaStore((s) => s.grpcResponseStatus);
  const grpcLastLatency = useVartaStore((s) => s.grpcLastLatencyMs);
  const grpcMessages = useVartaStore((s) => s.grpcMessages);

  const isGrpc = tab.request.type === "grpc";
  const isGraphQl = tab.request.type === "graphql";

  // Data extraction per protocol
  const httpResponse = tab.response;
  const gqlResponse = tab.graphqlResponse;

  // Derive status, metrics, headers, cookies, body
  const {
    statusCode,
    statusText,
    statusClass,
    timeMs,
    sizeBytes,
    headers,
    cookies,
    rawBody,
    gqlErrors,
    gqlExtensions,
  } = useMemo(() => {
    if (isGraphQl && gqlResponse) {
      const isSuccess = gqlResponse.status >= 200 && gqlResponse.status < 300 && !gqlResponse.errors;
      return {
        statusCode: gqlResponse.status,
        statusText: gqlResponse.statusText || (isSuccess ? "OK" : "Error"),
        statusClass: isSuccess ? "text-success" : "text-error",
        timeMs: gqlResponse.timeMs,
        sizeBytes: gqlResponse.sizeBytes,
        headers: gqlResponse.headers ?? {},
        cookies: [] as CookieRow[],
        rawBody: gqlResponse.data ?? (gqlResponse.errors ? "" : "null"),
        gqlErrors: gqlResponse.errors,
        gqlExtensions: gqlResponse.extensions,
      };
    }

    if (isGrpc) {
      const lastReceivedMsg = [...grpcMessages].reverse().find((m) => m.direction === "received");
      const isSuccess = grpcResponseStatus ? grpcResponseStatus.code === 0 : true;
      return {
        statusCode: grpcResponseStatus?.code ?? (lastReceivedMsg?.statusCode ? undefined : undefined),
        statusText: grpcResponseStatus?.text || lastReceivedMsg?.statusCode || (isSuccess ? "OK" : "Error"),
        statusClass: isSuccess ? "text-success" : "text-error",
        timeMs: grpcLastLatency,
        sizeBytes: lastReceivedMsg ? BigInt(new Blob([lastReceivedMsg.data]).size) : null,
        headers: grpcResponseMetadata ?? {},
        cookies: [] as CookieRow[],
        rawBody: lastReceivedMsg?.data ?? "",
        gqlErrors: undefined,
        gqlExtensions: undefined,
      };
    }

    // Default: REST HTTP
    if (httpResponse) {
      const isSuccess = httpResponse.status >= 200 && httpResponse.status < 300;
      const isError = httpResponse.status >= 400;
      return {
        statusCode: httpResponse.status,
        statusText: httpResponse.statusText,
        statusClass: isSuccess ? "text-success" : isError ? "text-error" : "text-warning",
        timeMs: httpResponse.timeMs,
        sizeBytes: httpResponse.sizeBytes,
        headers: httpResponse.headers ?? {},
        cookies: httpResponse.cookies ?? [],
        rawBody: httpResponse.body ?? "",
        gqlErrors: undefined,
        gqlExtensions: undefined,
      };
    }

    return {
      statusCode: undefined,
      statusText: undefined,
      statusClass: "text-text-muted",
      timeMs: null,
      sizeBytes: null,
      headers: {},
      cookies: [] as CookieRow[],
      rawBody: "",
      gqlErrors: undefined,
      gqlExtensions: undefined,
    };
  }, [isGraphQl, gqlResponse, isGrpc, grpcResponseStatus, grpcLastLatency, grpcMessages, grpcResponseMetadata, httpResponse]);

  // Format body for presentation
  const formattedBody = useMemo(() => {
    if (!rawBody) return "";
    if (!pretty) return rawBody;
    try {
      const parsed = JSON.parse(rawBody);
      return JSON.stringify(parsed, null, 2);
    } catch {
      return rawBody;
    }
  }, [rawBody, pretty]);

  // Handle Copy
  const handleCopy = async () => {
    const textToCopy = formattedBody || gqlErrors || "";
    if (!textToCopy) return;
    await navigator.clipboard.writeText(textToCopy);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  // Handle Download / Export
  const handleDownload = () => {
    const textToDownload = formattedBody || gqlErrors || "";
    if (!textToDownload) return;
    const blob = new Blob([textToDownload], { type: "application/json" });
    const fileUrl = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = fileUrl;
    link.download = `response-${Date.now()}.json`;
    link.click();
    URL.revokeObjectURL(fileUrl);
  };

  const headerEntries = Object.entries(headers);

  const testResults = httpResponse?.testResults ?? [];
  const consoleOutput = httpResponse?.consoleOutput ?? [];
  const envUpdates = httpResponse?.envUpdates ?? {};
  const envUpdateEntries = Object.entries(envUpdates);
  const passedTestsCount = testResults.filter((t) => t.passed).length;

  // Subtabs configuration
  const subTabs: { id: SubTab; label: string; count?: number }[] = [
    { id: "body", label: "Body" },
    {
      id: "headers",
      label: isGrpc ? "Trailing Metadata" : "Headers",
      count: headerEntries.length,
    },
  ];

  if (!isGrpc && !isGraphQl) {
    subTabs.push({
      id: "cookies",
      label: "Cookies",
      count: cookies.length,
    });
    subTabs.push({
      id: "scripts",
      label: "Tests & Logs",
      count: testResults.length > 0 ? testResults.length : undefined,
    });
  }

  return (
    <div className="flex h-full flex-col min-w-0 bg-bg">
      {/* Universal Metrics Bar */}
      <ResponseMetricsBar
        statusCode={statusCode}
        statusText={statusText}
        statusClass={statusClass}
        timeMs={timeMs}
        sizeBytes={sizeBytes}
        showPretty={activeSubTab === "body" && !!formattedBody}
        pretty={pretty}
        onTogglePretty={() => setPretty((p) => !p)}
        onCopy={formattedBody ? handleCopy : undefined}
        isCopied={copied}
        onDownload={formattedBody ? handleDownload : undefined}
        isMobile={isMobile}
      />

      {/* Subtabs Bar */}
      <div
        className={`flex gap-1 border-b border-border bg-panel/30 shrink-0 ${
          isMobile ? "overflow-x-auto scrollbar-hide px-2" : "px-4"
        }`}
      >
        {subTabs.map((t) => (
          <button
            key={t.id}
            onClick={() => setActiveSubTab(t.id)}
            className={`tab-trigger shrink-0 flex items-center gap-1.5 ${
              activeSubTab === t.id ? "tab-trigger-active" : ""
            }`}
          >
            <span>{t.label}</span>
            {t.count !== undefined && t.count > 0 && (
              <span className="rounded-full bg-primary/20 px-1.5 py-0.2 text-[10px] font-mono text-primary">
                {t.count}
              </span>
            )}
            {t.id === "body" && gqlErrors && (
              <span className="rounded-full bg-error/20 px-1.5 py-0.2 text-[10px] font-mono text-error">
                !
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Content Area */}
      <div className="flex-1 overflow-hidden min-h-0 min-w-0">
        {/* Body Viewer */}
        {activeSubTab === "body" && (
          <div className="flex h-full flex-col min-h-0 min-w-0">
            {/* GraphQL Errors Banner */}
            {gqlErrors && (
              <div className="border-b border-error/20 bg-error/5 px-4 py-2.5 shrink-0">
                <div className="flex items-center gap-2 mb-1">
                  <AlertCircle size={13} className="text-error" />
                  <span className="text-xs font-semibold text-error">
                    GraphQL Errors
                  </span>
                </div>
                <pre className="whitespace-pre-wrap font-mono text-xs text-error/90 leading-relaxed max-h-32 overflow-auto">
                  {gqlErrors}
                </pre>
              </div>
            )}

            {/* Monaco / CodeEditor for response body */}
            <div className="flex-1 min-h-0 overflow-hidden">
              <CodeEditor
                language="json"
                value={formattedBody}
                readOnly
                lineNumbers
                fontSize={isMobile ? 12 : undefined}
                placeholder="No response payload"
              />
            </div>

            {/* GraphQL Extensions */}
            {gqlExtensions && (
              <div className="border-t border-border px-4 py-2 shrink-0 bg-panel/30">
                <details>
                  <summary className="cursor-pointer text-xs text-text-muted hover:text-text-secondary">
                    Extensions
                  </summary>
                  <pre className="mt-1 whitespace-pre-wrap font-mono text-xs text-text-secondary max-h-24 overflow-auto">
                    {gqlExtensions}
                  </pre>
                </details>
              </div>
            )}
          </div>
        )}

        {/* Headers Viewer */}
        {activeSubTab === "headers" && (
          <div className="h-full overflow-y-auto">
            {headerEntries.length === 0 ? (
              <div className="flex h-full items-center justify-center text-xs text-text-muted">
                No headers returned
              </div>
            ) : (
              <table className="w-full text-xs">
                <thead className="sticky top-0 bg-panel border-b border-border text-text-muted">
                  <tr>
                    <th className="px-4 py-2 text-left font-medium">Name</th>
                    <th className="px-4 py-2 text-left font-medium">Value</th>
                  </tr>
                </thead>
                <tbody>
                  {headerEntries.map(([k, v]) => (
                    <tr
                      key={k}
                      className="border-b border-border/40 hover:bg-panel-raised/30 transition-colors"
                    >
                      <td className="px-4 py-2 font-mono text-secondary">{k}</td>
                      <td className="px-4 py-2 font-mono text-text-primary break-all">
                        {v}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        )}

        {/* Cookies Viewer */}
        {activeSubTab === "cookies" && (
          <div className="h-full overflow-y-auto">
            {cookies.length === 0 ? (
              <div className="flex h-full items-center justify-center text-xs text-text-muted">
                No cookies set in this response
              </div>
            ) : (
              <table className="w-full text-xs">
                <thead className="sticky top-0 bg-panel border-b border-border text-text-muted">
                  <tr>
                    <th className="px-4 py-2 text-left font-medium">Name</th>
                    <th className="px-4 py-2 text-left font-medium">Value</th>
                    <th className="px-4 py-2 text-left font-medium">Domain</th>
                  </tr>
                </thead>
                <tbody>
                  {cookies.map((c, i) => (
                    <tr
                      key={c.id || i}
                      className="border-b border-border/40 hover:bg-panel-raised/30 transition-colors"
                    >
                      <td className="px-4 py-2 font-mono text-primary">{c.name}</td>
                      <td className="px-4 py-2 font-mono text-text-primary break-all">
                        {c.value}
                      </td>
                      <td className="px-4 py-2 font-mono text-text-muted">
                        {c.domain || "-"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        )}

        {/* Scripts, Tests & Logs Viewer */}
        {activeSubTab === "scripts" && (
          <div className="h-full overflow-y-auto p-4 space-y-4">
            {/* Saved Environment Variables */}
            {envUpdateEntries.length > 0 && (
              <div className="rounded-lg border border-emerald-500/30 bg-emerald-500/5 p-3">
                <div className="flex items-center gap-2 mb-2 text-xs font-semibold text-emerald-400">
                  <Sparkles size={14} />
                  <span>Saved to Environment ({envUpdateEntries.length})</span>
                </div>
                <div className="space-y-1.5">
                  {envUpdateEntries.map(([key, val]) => (
                    <div
                      key={key}
                      className="flex items-center justify-between rounded bg-panel/60 border border-border/60 px-3 py-1.5 text-xs font-mono"
                    >
                      <span className="text-primary font-semibold">{key}</span>
                      <span className="text-text-muted max-w-[65%] truncate" title={val}>
                        {val}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Test Assertions */}
            {testResults.length > 0 && (
              <div className="rounded-lg border border-border bg-panel/40 p-3">
                <div className="flex items-center justify-between mb-2">
                  <div className="text-xs font-semibold text-text-primary">
                    Test Results
                  </div>
                  <div className="flex items-center gap-2 text-xs">
                    <span className="text-emerald-400 font-medium">
                      ✓ {passedTestsCount} passed
                    </span>
                    {testResults.length - passedTestsCount > 0 && (
                      <span className="text-red-400 font-medium">
                        ✗ {testResults.length - passedTestsCount} failed
                      </span>
                    )}
                  </div>
                </div>
                <div className="space-y-1.5">
                  {testResults.map((tr, idx) => (
                    <div
                      key={idx}
                      className={`flex items-start gap-2 rounded border px-3 py-2 text-xs ${
                        tr.passed
                          ? "bg-emerald-500/5 border-emerald-500/20 text-text-primary"
                          : "bg-red-500/5 border-red-500/30 text-red-300"
                      }`}
                    >
                      {tr.passed ? (
                        <CheckCircle2 size={15} className="text-emerald-400 shrink-0 mt-0.5" />
                      ) : (
                        <XCircle size={15} className="text-red-400 shrink-0 mt-0.5" />
                      )}
                      <div className="flex-1 min-w-0">
                        <div className="font-medium">{tr.name}</div>
                        {tr.error && (
                          <div className="mt-0.5 text-[11px] font-mono text-red-400 opacity-90 break-words">
                            {tr.error}
                          </div>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Console Log Stream */}
            {consoleOutput.length > 0 && (
              <div className="rounded-lg border border-border bg-black/40 p-3">
                <div className="flex items-center gap-1.5 mb-2 text-xs font-semibold text-text-muted">
                  <Terminal size={14} className="text-primary" />
                  <span>Console Output</span>
                </div>
                <pre className="font-mono text-xs text-text-secondary whitespace-pre-wrap leading-relaxed space-y-0.5 max-h-60 overflow-y-auto">
                  {consoleOutput.map((line, idx) => (
                    <div key={idx} className="hover:bg-white/5 px-1 rounded">
                      {line}
                    </div>
                  ))}
                </pre>
              </div>
            )}

            {/* Empty State */}
            {testResults.length === 0 &&
              consoleOutput.length === 0 &&
              envUpdateEntries.length === 0 && (
                <div className="flex flex-col items-center justify-center py-12 text-center text-xs text-text-muted">
                  <Terminal size={24} className="mb-2 text-text-muted/40" />
                  <p>No tests or script logs for this request.</p>
                  <p className="mt-1 text-[11px] text-text-muted/70">
                    Add assertions or logs in the request <strong>Scripts</strong> tab.
                  </p>
                </div>
              )}
          </div>
        )}
      </div>
    </div>
  );
}
