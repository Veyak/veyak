import { useState, useRef, useEffect, type ReactNode } from "react";
import {
  Copy,
  Check,
  Download,
  Trash2,
  Code2,
  AlignLeft,
  Square,
  Clock,
  Search,
  ChevronDown,
  RotateCcw,
} from "lucide-react";
import { HistoryEntry } from "@veyak-internal/models";

export interface ResponseMetricsBarProps {
  statusText?: string;
  statusCode?: number | string;
  statusClass?: string;
  timeMs?: number | bigint | null;
  sizeBytes?: number | bigint | null;
  itemCount?: number;
  itemLabel?: string;
  isStreaming?: boolean;
  onStopStream?: () => void;
  onClear?: () => void;
  onCopy?: () => void;
  isCopied?: boolean;
  showPretty?: boolean;
  pretty?: boolean;
  onTogglePretty?: () => void;
  onDownload?: () => void;
  filterText?: string;
  onFilterChange?: (text: string) => void;
  showFilter?: boolean;
  isMobile?: boolean;
  extraActions?: ReactNode;
  historyEntries?: HistoryEntry[];
  activeHistoryId?: string | null;
  onSelectHistoryEntry?: (entry: HistoryEntry) => void;
  onRestoreLatestHistory?: () => void;
}

export function formatBytes(bytes?: number | bigint | null): string {
  if (bytes === undefined || bytes === null || Number(bytes) <= 0) return "";
  const b = Number(bytes);
  if (b < 1024) return `${b} B`;
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`;
  return `${(b / (1024 * 1024)).toFixed(1)} MB`;
}

export default function ResponseMetricsBar({
  statusText,
  statusCode,
  statusClass = "text-success",
  timeMs,
  sizeBytes,
  itemCount,
  itemLabel = "messages",
  isStreaming = false,
  onStopStream,
  onClear,
  onCopy,
  isCopied = false,
  showPretty = false,
  pretty = true,
  onTogglePretty,
  onDownload,
  filterText,
  onFilterChange,
  showFilter = false,
  isMobile = false,
  extraActions,
  historyEntries,
  activeHistoryId,
  onSelectHistoryEntry,
  onRestoreLatestHistory,
}: ResponseMetricsBarProps) {
  const sizeFormatted = formatBytes(sizeBytes);
  const [isHistoryOpen, setIsHistoryOpen] = useState(false);
  const historyDropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (
        historyDropdownRef.current &&
        !historyDropdownRef.current.contains(e.target as Node)
      ) {
        setIsHistoryOpen(false);
      }
    }
    if (isHistoryOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [isHistoryOpen]);

  return (
    <div
      className={`flex items-center gap-3 border-b border-border bg-panel text-sm shrink-0 ${
        isMobile ? "flex-wrap gap-2 px-3 py-2" : "px-4 py-2"
      }`}
    >
      {/* Status Pill */}
      {(statusText || statusCode !== undefined) && (
        <div className="flex items-center gap-1.5 font-medium">
          <span
            className={`inline-block h-2 w-2 rounded-full ${
              isStreaming
                ? "bg-secondary animate-pulse"
                : statusClass.includes("success")
                ? "bg-success"
                : statusClass.includes("error")
                ? "bg-error"
                : statusClass.includes("warning")
                ? "bg-warning"
                : "bg-text-muted"
            }`}
          />
          <span className={`font-semibold ${statusClass}`}>
            {statusCode !== undefined && `${statusCode} `}
            {statusText}
          </span>
        </div>
      )}

      {/* Latency / Round-trip Time */}
      {timeMs !== undefined && timeMs !== null && Number(timeMs) >= 0 && (
        <span className="flex items-center gap-1 text-xs text-text-secondary">
          <Clock size={11} />
          {Number(timeMs)} ms
        </span>
      )}

      {/* Response Size */}
      {sizeFormatted && (
        <span className="text-xs text-text-muted">{sizeFormatted}</span>
      )}

      {/* Item / Message / Event count */}
      {itemCount !== undefined && itemCount > 0 && (
        <span className="text-xs text-text-muted">
          {itemCount} {itemLabel}
        </span>
      )}

      {/* Response History Selector Dropdown */}
      {historyEntries && historyEntries.length > 0 && (
        <div className="relative" ref={historyDropdownRef}>
          <button
            onClick={() => setIsHistoryOpen(!isHistoryOpen)}
            className={`flex items-center gap-1.5 px-2 py-0.5 rounded-md text-xs font-medium border transition-colors cursor-pointer ${
              activeHistoryId
                ? "bg-primary/15 text-primary border-primary/30"
                : "border-border bg-panel-raised hover:bg-borderMuted text-text-secondary hover:text-text-primary"
            }`}
            title="Switch between past responses for this request"
          >
            <Clock
              size={12}
              className={activeHistoryId ? "text-primary" : "text-text-muted"}
            />
            <span>
              {activeHistoryId
                ? "Viewing Historical"
                : `History (${historyEntries.length})`}
            </span>
            <ChevronDown
              size={11}
              className={`transition-transform duration-150 ${
                isHistoryOpen ? "rotate-180" : ""
              }`}
            />
          </button>

          {isHistoryOpen && (
            <div className="absolute left-0 top-full mt-1.5 z-50 w-72 rounded-xl border border-border bg-panel-raised shadow-elevated overflow-hidden py-1 animate-in fade-in zoom-in-95 duration-100">
              <div className="px-3 py-1.5 border-b border-border/70 bg-panel flex items-center justify-between text-[11px] font-semibold text-text-muted uppercase tracking-wider">
                <span>Response Runs</span>
                {activeHistoryId && onRestoreLatestHistory && (
                  <button
                    onClick={() => {
                      onRestoreLatestHistory();
                      setIsHistoryOpen(false);
                    }}
                    className="text-primary hover:underline lowercase font-normal cursor-pointer flex items-center gap-1"
                  >
                    <RotateCcw size={10} />
                    latest
                  </button>
                )}
              </div>
              <div className="max-h-60 overflow-y-auto p-1 space-y-0.5 bg-panel-raised">
                {historyEntries.map((h, idx) => {
                  const isCurrent =
                    activeHistoryId === h.id || (!activeHistoryId && idx === 0);
                  const isSuccess = h.status >= 200 && h.status < 300;
                  return (
                    <button
                      key={h.id}
                      onClick={() => {
                        onSelectHistoryEntry?.(h);
                        setIsHistoryOpen(false);
                      }}
                      className={`flex w-full items-center justify-between px-2.5 py-1.5 rounded-lg text-left text-xs transition-colors cursor-pointer ${
                        isCurrent
                          ? "bg-primary/20 text-text-primary font-medium"
                          : "hover:bg-panel text-text-secondary hover:text-text-primary"
                      }`}
                    >
                      <div className="flex items-center gap-2">
                        <span
                          className={`font-mono text-[11px] font-bold ${
                            isSuccess ? "text-success" : "text-error"
                          }`}
                        >
                          {h.status}
                        </span>
                        <span className="text-[11px] font-mono text-text-muted">
                          {Number(h.durationMs)}ms
                        </span>
                      </div>
                      <span className="text-[10px] text-text-muted font-mono">
                        {new Date(h.createdAt).toLocaleTimeString([], {
                          hour: "2-digit",
                          minute: "2-digit",
                        })}
                      </span>
                    </button>
                  );
                })}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Filter search bar if enabled */}
      {showFilter && onFilterChange && (
        <div className="relative flex items-center ml-1">
          <Search
            size={11}
            className="pointer-events-none absolute left-2 text-text-muted"
          />
          <input
            value={filterText ?? ""}
            onChange={(e) => onFilterChange(e.target.value)}
            placeholder="Search payload…"
            className="h-6.5 w-32 sm:w-44 rounded-md border border-border bg-bg pl-6 pr-2 font-mono text-xs text-text-primary placeholder:text-text-muted outline-none focus:border-primary"
          />
        </div>
      )}

      {/* Right Actions */}
      <div className={`flex items-center gap-1.5 ${isMobile ? "ml-auto" : "ml-auto"}`}>
        {extraActions}

        {/* Live Stream Stop */}
        {isStreaming && onStopStream && (
          <button
            onClick={onStopStream}
            className="flex items-center gap-1.5 rounded-md bg-error/20 border border-error/40 px-2.5 py-1 text-xs text-error hover:bg-error hover:text-white transition-colors cursor-pointer"
            title="Stop stream connection"
          >
            <Square size={11} fill="currentColor" />
            Stop
          </button>
        )}

        {/* Pretty / Raw toggle */}
        {showPretty && onTogglePretty && (
          <button
            onClick={onTogglePretty}
            className="flex items-center gap-1.5 rounded-md border border-border px-2 py-1 text-xs text-text-secondary hover:bg-panel-raised hover:text-text-primary transition-colors cursor-pointer"
            title={pretty ? "View unformatted raw body" : "Pretty print JSON"}
          >
            {pretty ? <AlignLeft size={12} /> : <Code2 size={12} />}
            <span className="hidden sm:inline">{pretty ? "Raw" : "Pretty"}</span>
          </button>
        )}

        {/* Copy Response */}
        {onCopy && (
          <button
            onClick={onCopy}
            className="flex items-center gap-1 rounded-md border border-border p-1.5 text-text-secondary hover:bg-panel-raised hover:text-text-primary transition-colors cursor-pointer"
            title="Copy response body"
            aria-label="Copy response"
          >
            {isCopied ? <Check size={13} className="text-success" /> : <Copy size={13} />}
          </button>
        )}

        {/* Download / Export */}
        {onDownload && (
          <button
            onClick={onDownload}
            className="flex items-center gap-1 rounded-md border border-border p-1.5 text-text-secondary hover:bg-panel-raised hover:text-text-primary transition-colors cursor-pointer"
            title="Export response"
            aria-label="Download response"
          >
            <Download size={13} />
          </button>
        )}

        {/* Clear */}
        {onClear && (
          <button
            onClick={onClear}
            className="flex items-center gap-1 rounded-md border border-border p-1.5 text-text-secondary hover:bg-panel-raised hover:text-error transition-colors cursor-pointer"
            title="Clear output"
            aria-label="Clear output"
          >
            <Trash2 size={13} />
          </button>
        )}
      </div>
    </div>
  );
}
