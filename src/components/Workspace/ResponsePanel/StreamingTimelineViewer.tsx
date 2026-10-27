import { useState, useMemo, useRef, useEffect } from "react";
import {
  ArrowUp,
  ArrowDown,
  Unplug,
  AlertCircle,
  Copy,
  Check,
  Send,
  X,
  Search,
  FileCode,
  Radio,
  Clock,
} from "lucide-react";
import { RequestTab } from "../../../types";
import { useVartaStore } from "../../../store/vartaStore";
import ResponseMetricsBar from "./ResponseMetricsBar";
import CodeEditor from "../../CodeEditor";
import { saveTextFile } from "../../../services/fileSaver";

interface StreamingTimelineViewerProps {
  tab: RequestTab;
  isMobile?: boolean;
}

export interface StreamTimelineItem {
  id: string;
  direction: "sent" | "received" | "closed" | "system";
  timestamp: string;
  eventType?: string;
  statusCode?: string;
  isError?: boolean;
  latencyMs?: number;
  payload: string;
  sizeBytes?: number;
}

/**
 * Displays and manages a streaming event timeline for WebSocket, gRPC, or GraphQL requests.
 *
 * @param tab - Request tab containing the stream configuration and event data
 * @param isMobile - Whether to use the mobile layout
 * @returns The streaming timeline viewer interface
 */
export default function StreamingTimelineViewer({
  tab,
  isMobile = false,
}: StreamingTimelineViewerProps) {
  const [selectedItem, setSelectedItem] = useState<StreamTimelineItem | null>(null);
  const [directionFilter, setDirectionFilter] = useState<"all" | "sent" | "received">("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [errorOnly, setErrorOnly] = useState(false);
  const [copied, setCopied] = useState(false);
  const [downloaded, setDownloaded] = useState(false);

  // Quick Composer State for WebSocket
  const [wsComposerText, setWsComposerText] = useState("");

  // Store actions & state
  const sendWsMessage = useVartaStore((s) => s.sendWsMessage);
  const disconnectWebSocket = useVartaStore((s) => s.disconnectWebSocket);
  const clearWsMessages = useVartaStore((s) => s.clearWsMessages);

  // gRPC store
  const grpcMessages = useVartaStore((s) => s.grpcMessages);
  const grpcCallStatus = useVartaStore((s) => s.grpcCallStatus);
  const cancelGrpcCall = useVartaStore((s) => s.cancelGrpcCall);
  const clearGrpcMessages = useVartaStore((s) => s.clearGrpcMessages);
  const sendGrpcMessage = useVartaStore((s) => s.sendGrpcMessage);
  const grpcRequestBody = useVartaStore((s) => s.grpcRequestBody);

  // GraphQL Subscriptions
  const gqlSubMessages = tab.graphqlSubscriptionMessages || [];
  const gqlCallStatus = tab.graphqlCallStatus || "idle";
  const cancelGqlSub = useVartaStore((s) => s.cancelGraphqlSubscription);
  const clearGqlMessages = useVartaStore((s) => s.clearGraphqlMessages);

  const isWs = tab.request.method === "WS";
  const isGrpc = tab.request.type === "grpc";
  const isGraphQl = tab.request.type === "graphql";

  const logEndRef = useRef<HTMLDivElement>(null);

  // ---------------------------------------------------------------------------
  // Normalize Events into Unified StreamTimelineItems
  // ---------------------------------------------------------------------------
  const timelineItems: StreamTimelineItem[] = useMemo(() => {
    if (isWs) {
      return (tab.wsMessages || []).map((msg, index) => {
        let eventType: string | undefined;
        let isError = false;

        // If graphql-ws protocol, parse opcodes
        if (tab.wsProtocol === "graphql-ws") {
          try {
            const parsed = JSON.parse(msg.data);
            if (parsed && typeof parsed.type === "string") {
              eventType = parsed.type.toUpperCase();
              if (parsed.type === "error") isError = true;
            }
          } catch {
            // plain text
          }
        }

        return {
          id: `ws-${index}-${msg.timestamp}`,
          direction: msg.direction,
          timestamp: msg.timestamp,
          eventType,
          isError,
          payload: msg.data,
          sizeBytes: new Blob([msg.data]).size,
        };
      });
    }

    if (isGrpc) {
      return grpcMessages.map((msg) => ({
        id: msg.id,
        direction: msg.direction,
        timestamp: msg.timestamp,
        eventType: msg.direction === "sent" ? "CLIENT_MSG" : "SERVER_MSG",
        statusCode: msg.statusCode,
        isError: msg.isError,
        latencyMs: msg.latencyMs,
        payload: msg.data,
        sizeBytes: new Blob([msg.data]).size,
      }));
    }

    if (isGraphQl) {
      return gqlSubMessages.map((msg) => ({
        id: msg.id,
        direction: msg.eventType === "connecting" ? "system" : "received",
        timestamp: msg.timestamp,
        eventType: msg.eventType.toUpperCase(),
        isError: msg.eventType === "error",
        payload: msg.payload,
        sizeBytes: new Blob([msg.payload]).size,
      }));
    }

    return [];
  }, [isWs, tab.wsMessages, tab.wsProtocol, isGrpc, grpcMessages, isGraphQl, gqlSubMessages]);

  // Filter items
  const filteredItems = useMemo(() => {
    return timelineItems.filter((item) => {
      if (directionFilter !== "all" && item.direction !== directionFilter) {
        return false;
      }
      if (errorOnly && !item.isError) {
        return false;
      }
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const matchesPayload = item.payload.toLowerCase().includes(q);
        const matchesType = item.eventType?.toLowerCase().includes(q);
        if (!matchesPayload && !matchesType) return false;
      }
      return true;
    });
  }, [timelineItems, directionFilter, errorOnly, searchQuery]);

  // Auto-scroll on new items if no item selected
  useEffect(() => {
    if (!selectedItem) {
      logEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [timelineItems.length, selectedItem]);

  // Handle Stop Stream
  const handleStopStream = () => {
    if (isWs) disconnectWebSocket();
    else if (isGrpc) cancelGrpcCall();
    else if (isGraphQl) cancelGqlSub(tab.id);
  };

  // Handle Clear
  const handleClear = () => {
    setSelectedItem(null);
    if (isWs) clearWsMessages(tab.id);
    else if (isGrpc) clearGrpcMessages();
    else if (isGraphQl) clearGqlMessages(tab.id);
  };

  // Connection status & label
  const { statusText, statusClass, isStreaming } = useMemo(() => {
    if (isWs) {
      const connected = tab.wsStatus === "connected";
      const connecting = tab.wsStatus === "connecting";
      return {
        statusText: connected ? "Connected" : connecting ? "Connecting…" : "Disconnected",
        statusClass: connected ? "text-success" : connecting ? "text-warning" : "text-text-muted",
        isStreaming: connected,
      };
    }

    if (isGrpc) {
      const streaming = grpcCallStatus === "streaming";
      const invoking = grpcCallStatus === "invoking";
      return {
        statusText: streaming ? "Streaming" : invoking ? "Invoking…" : grpcCallStatus.toUpperCase(),
        statusClass: streaming ? "text-secondary" : invoking ? "text-warning" : "text-text-muted",
        isStreaming: streaming,
      };
    }

    if (isGraphQl) {
      const streaming = gqlCallStatus === "streaming";
      return {
        statusText: streaming ? "Subscribed" : gqlCallStatus.toUpperCase(),
        statusClass: streaming ? "text-method-graphql" : "text-text-muted",
        isStreaming: streaming,
      };
    }

    return {
      statusText: "Idle",
      statusClass: "text-text-muted",
      isStreaming: false,
    };
  }, [isWs, tab.wsStatus, isGrpc, grpcCallStatus, isGraphQl, gqlCallStatus]);

  // Copy selected frame
  const handleCopySelected = async () => {
    if (!selectedItem) return;
    await navigator.clipboard.writeText(selectedItem.payload);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  // Send message from Quick Composer
  const handleSendWsComposer = () => {
    if (!wsComposerText.trim()) return;
    sendWsMessage(wsComposerText.trim());
    setWsComposerText("");
  };

  // Download all timeline frames
  const handleDownloadTimeline = async () => {
    if (timelineItems.length === 0) return;
    const content = JSON.stringify(timelineItems, null, 2);
    const baseName = tab.request.name
      ? tab.request.name.replace(/[^a-zA-Z0-9_-]/g, "_").toLowerCase()
      : "stream-timeline";
    const defaultFilename = `${baseName}-${Date.now()}.json`;

    const success = await saveTextFile({
      content,
      defaultPath: defaultFilename,
      filters: [
        { name: "JSON File", extensions: ["json"] },
        { name: "All Files", extensions: ["*"] },
      ],
      mimeType: "application/json",
    });

    if (success) {
      setDownloaded(true);
      setTimeout(() => setDownloaded(false), 1500);
    }
  };

  return (
    <div className="flex h-full flex-col min-w-0 bg-bg">
      {/* Universal Metrics Bar */}
      <ResponseMetricsBar
        statusText={statusText}
        statusClass={statusClass}
        itemCount={timelineItems.length}
        itemLabel="frames"
        isStreaming={isStreaming}
        onStopStream={handleStopStream}
        onClear={timelineItems.length > 0 ? handleClear : undefined}
        onDownload={timelineItems.length > 0 ? handleDownloadTimeline : undefined}
        isDownloaded={downloaded}
        isMobile={isMobile}
      />

      {/* Filter and Controls Toolbar */}
      <div className="flex items-center gap-2 border-b border-border bg-panel/40 px-4 py-2 shrink-0 flex-wrap">
        {/* Direction Filter Pills */}
        <div className="flex items-center rounded-md border border-border bg-panel p-0.5 text-xs">
          <button
            onClick={() => setDirectionFilter("all")}
            className={`px-2 py-0.5 rounded transition-colors ${directionFilter === "all"
                ? "bg-primary/20 text-primary font-medium"
                : "text-text-muted hover:text-text-secondary"
              }`}
          >
            All
          </button>
          <button
            onClick={() => setDirectionFilter("sent")}
            className={`flex items-center gap-1 px-2 py-0.5 rounded transition-colors ${directionFilter === "sent"
                ? "bg-primary/20 text-primary font-medium"
                : "text-text-muted hover:text-text-secondary"
              }`}
          >
            <ArrowUp size={11} />
            Sent
          </button>
          <button
            onClick={() => setDirectionFilter("received")}
            className={`flex items-center gap-1 px-2 py-0.5 rounded transition-colors ${directionFilter === "received"
                ? "bg-secondary/20 text-secondary font-medium"
                : "text-text-muted hover:text-text-secondary"
              }`}
          >
            <ArrowDown size={11} />
            Received
          </button>
        </div>

        {/* Search Input */}
        <div className="relative flex items-center min-w-[140px] flex-1 max-w-xs">
          <Search
            size={11}
            className="pointer-events-none absolute left-2 text-text-muted"
          />
          <input
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Filter frames…"
            className="h-6.5 w-full rounded-md border border-border bg-bg pl-6 pr-2 font-mono text-xs text-text-primary placeholder:text-text-muted outline-none focus:border-primary"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              className="absolute right-1.5 p-0.5 text-text-muted hover:text-text-primary"
            >
              <X size={11} />
            </button>
          )}
        </div>

        {/* Error filter toggle */}
        <button
          onClick={() => setErrorOnly((e) => !e)}
          className={`flex items-center gap-1 rounded px-2 py-1 text-xs transition-colors ${errorOnly
              ? "bg-error/20 text-error font-medium"
              : "text-text-muted hover:text-text-secondary"
            }`}
        >
          <AlertCircle size={12} />
          Errors Only
        </button>

        {selectedItem && (
          <button
            onClick={() => setSelectedItem(null)}
            className="ml-auto flex items-center gap-1 text-xs text-primary hover:underline"
          >
            Close Inspector
          </button>
        )}
      </div>

      {/* Main Timeline View + Inspector split */}
      <div className="flex-1 flex min-h-0 overflow-hidden flex-col md:flex-row">
        {/* Timeline Log Table */}
        <div
          className={`flex-1 overflow-y-auto ${selectedItem ? "md:border-r border-border md:w-1/2" : "w-full"
            }`}
        >
          {filteredItems.length === 0 ? (
            <div className="flex h-full flex-col items-center justify-center gap-2 p-6 text-center text-text-muted text-xs">
              <Radio size={24} className="text-text-muted/40 animate-pulse" />
              <span>
                {isStreaming
                  ? "Connected. Awaiting streaming events…"
                  : "Start a stream connection to see real-time events here."}
              </span>
            </div>
          ) : (
            <div className="divide-y divide-border/40 font-mono text-xs">
              {filteredItems.map((item) => {
                const isSelected = selectedItem?.id === item.id;
                const isSent = item.direction === "sent";
                const isClosed = item.direction === "closed";

                if (isClosed) {
                  return (
                    <div
                      key={item.id}
                      className="flex items-center gap-2 px-4 py-2 text-text-muted italic bg-panel/10"
                    >
                      <Unplug size={12} />
                      <span>Connection closed</span>
                      <span className="ml-auto text-[10px] opacity-60">
                        {new Date(item.timestamp).toLocaleTimeString()}
                      </span>
                    </div>
                  );
                }

                return (
                  <div
                    key={item.id}
                    onClick={() => setSelectedItem(item)}
                    className={`group flex items-center gap-2.5 px-4 py-2 cursor-pointer transition-colors ${isSelected
                        ? "bg-primary/15 border-l-2 border-primary"
                        : "hover:bg-panel-raised/40 border-l-2 border-transparent"
                      }`}
                  >
                    {/* Direction Icon */}
                    <div className="shrink-0">
                      {isSent ? (
                        <ArrowUp size={13} className="text-primary" />
                      ) : (
                        <ArrowDown size={13} className="text-secondary" />
                      )}
                    </div>

                    {/* Direction / Opcode badge */}
                    {item.eventType && (
                      <span
                        className={`rounded px-1.5 py-0.5 text-[9px] font-bold uppercase tracking-wider ${item.isError
                            ? "bg-error/20 text-error"
                            : isSent
                              ? "bg-primary/20 text-primary"
                              : "bg-secondary/20 text-secondary"
                          }`}
                      >
                        {item.eventType}
                      </span>
                    )}

                    {/* Latency if available */}
                    {item.latencyMs !== undefined && (
                      <span className="text-[10px] text-text-muted flex items-center gap-0.5">
                        <Clock size={10} />
                        {item.latencyMs}ms
                      </span>
                    )}

                    {/* Payload Preview */}
                    <span className="truncate flex-1 text-text-primary text-[11px]">
                      {item.payload.replace(/\s+/g, " ")}
                    </span>

                    {/* Timestamp */}
                    <span className="shrink-0 text-[10px] text-text-muted opacity-75">
                      {new Date(item.timestamp).toLocaleTimeString()}
                    </span>
                  </div>
                );
              })}
              <div ref={logEndRef} />
            </div>
          )}
        </div>

        {/* Interactive Frame Inspector */}
        {selectedItem && (
          <div className="flex flex-col md:w-1/2 min-h-0 bg-panel/20 border-t md:border-t-0 border-border">
            {/* Inspector Header */}
            <div className="flex items-center justify-between border-b border-border bg-panel px-3 py-2 shrink-0">
              <div className="flex items-center gap-2">
                <FileCode size={13} className="text-primary" />
                <span className="text-xs font-semibold text-text-primary">
                  Frame Inspector
                </span>
                <span
                  className={`rounded px-1.5 py-0.5 text-[9px] font-bold uppercase ${selectedItem.direction === "sent"
                      ? "bg-primary/20 text-primary"
                      : "bg-secondary/20 text-secondary"
                    }`}
                >
                  {selectedItem.direction}
                </span>
                {selectedItem.eventType && (
                  <span className="rounded bg-panel-raised px-1.5 py-0.5 text-[9px] font-mono text-text-muted">
                    {selectedItem.eventType}
                  </span>
                )}
              </div>

              <div className="flex items-center gap-1">
                <button
                  onClick={handleCopySelected}
                  className="flex items-center gap-1 rounded border border-border px-2 py-1 text-xs text-text-secondary hover:bg-panel-raised hover:text-text-primary transition-colors"
                  title="Copy payload"
                >
                  {copied ? <Check size={12} className="text-success" /> : <Copy size={12} />}
                  <span>{copied ? "Copied" : "Copy"}</span>
                </button>
                <button
                  onClick={() => setSelectedItem(null)}
                  className="p-1 text-text-muted hover:text-text-primary rounded"
                >
                  <X size={14} />
                </button>
              </div>
            </div>

            {/* Inspector Content */}
            <div className="flex-1 min-h-0 overflow-hidden">
              <CodeEditor
                language="json"
                value={(() => {
                  try {
                    return JSON.stringify(JSON.parse(selectedItem.payload), null, 2);
                  } catch {
                    return selectedItem.payload;
                  }
                })()}
                readOnly
                lineNumbers
                fontSize={12}
              />
            </div>
          </div>
        )}
      </div>

      {/* Quick Composer for Live Streams */}
      {isWs && tab.wsStatus === "connected" && (
        <div className="border-t border-border bg-panel p-2.5 flex items-center gap-2 shrink-0">
          <input
            value={wsComposerText}
            onChange={(e) => setWsComposerText(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleSendWsComposer();
            }}
            placeholder="Type a message to send to the WebSocket server…"
            className="flex-1 h-8 rounded-md border border-border bg-bg px-3 font-mono text-xs text-text-primary placeholder:text-text-muted outline-none focus:border-primary"
          />
          <button
            onClick={handleSendWsComposer}
            disabled={!wsComposerText.trim()}
            className="flex items-center gap-1.5 rounded-md bg-brand-gradient px-4 py-1.5 text-xs font-medium text-white shadow-panel hover:opacity-90 disabled:opacity-50 transition-opacity cursor-pointer"
          >
            <Send size={12} />
            <span>Send</span>
          </button>
        </div>
      )}

      {/* Quick Composer for gRPC Client/Bidi Streams */}
      {isGrpc && grpcCallStatus === "streaming" && (
        <div className="border-t border-border bg-panel p-2.5 flex items-center justify-between shrink-0">
          <span className="text-xs text-secondary flex items-center gap-1.5 font-medium">
            <Radio size={13} className="animate-pulse" />
            Stream Active: compose message in Request Panel or send payload
          </span>
          <button
            onClick={() => {
              if (grpcRequestBody.trim()) sendGrpcMessage(grpcRequestBody);
            }}
            disabled={!grpcRequestBody.trim()}
            className="flex items-center gap-1.5 rounded-md bg-brand-gradient px-4 py-1.5 text-xs font-medium text-white shadow-panel hover:opacity-90 disabled:opacity-50 transition-opacity cursor-pointer"
          >
            <Send size={12} />
            <span>Send Message</span>
          </button>
        </div>
      )}
    </div>
  );
}
