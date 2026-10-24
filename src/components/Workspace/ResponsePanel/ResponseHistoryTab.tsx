import { useState } from "react";
import {
  Clock,
  Trash2,
  CheckCircle2,
  AlertCircle,
  Eye,
  RotateCcw,
  Calendar,
  Zap,
} from "lucide-react";
import { HistoryEntry } from "@veyak-internal/models";
import { MethodStyles } from "../../../types";
import { formatBytes } from "./ResponseMetricsBar";

interface ResponseHistoryTabProps {
  entries: HistoryEntry[];
  activeHistoryId: string | null | undefined;
  onSelectEntry: (entry: HistoryEntry) => void;
  onRestoreLatest: () => void;
  onDeleteEntry: (id: string) => void;
  onClearHistory: () => void;
  hasLatestLiveResponse: boolean;
  isMobile?: boolean;
}

function getStatusStyle(status: number) {
  if (status >= 200 && status < 300) {
    return {
      badge: "bg-success/10 text-success border-success/20",
      text: "text-success",
      icon: CheckCircle2,
    };
  }
  if (status >= 400) {
    return {
      badge: "bg-error/10 text-error border-error/20",
      text: "text-error",
      icon: AlertCircle,
    };
  }
  return {
    badge: "bg-warning/10 text-warning border-warning/20",
    text: "text-warning",
    icon: AlertCircle,
  };
}

function formatRelativeTime(dateString: string): string {
  try {
    const date = new Date(dateString);
    if (isNaN(date.getTime())) return dateString;
    const now = new Date();
    const diffSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (diffSeconds < 5) return "Just now";
    if (diffSeconds < 60) return `${diffSeconds}s ago`;
    const diffMinutes = Math.floor(diffSeconds / 60);
    if (diffMinutes < 60) return `${diffMinutes}m ago`;
    const diffHours = Math.floor(diffMinutes / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    if (diffDays < 7) return `${diffDays}d ago`;

    return date.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return dateString;
  }
}

function formatFullTime(dateString: string): string {
  try {
    const date = new Date(dateString);
    if (isNaN(date.getTime())) return dateString;
    return date.toLocaleString();
  } catch {
    return dateString;
  }
}

export default function ResponseHistoryTab({
  entries,
  activeHistoryId,
  onSelectEntry,
  onRestoreLatest,
  onDeleteEntry,
  onClearHistory,
  hasLatestLiveResponse,
  isMobile = false,
}: ResponseHistoryTabProps) {
  const [confirmClear, setConfirmClear] = useState(false);

  if (entries.length === 0) {
    return (
      <div className="flex h-full flex-col items-center justify-center p-8 text-center text-text-muted gap-3">
        <div className="flex h-12 w-12 items-center justify-center rounded-full bg-panel border border-border">
          <Clock size={22} className="text-text-muted/60" />
        </div>
        <div className="space-y-1">
          <h4 className="text-sm font-medium text-text-primary">
            No Response History
          </h4>
          <p className="text-xs text-text-muted max-w-sm">
            Previous responses for this request will appear here every time you
            execute it.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col overflow-hidden bg-bg">
      {/* Header Bar */}
      <div className="flex items-center justify-between border-b border-border bg-panel/30 px-4 py-2.5 shrink-0">
        <div className="flex items-center gap-2">
          <Clock size={14} className="text-primary" />
          <span className="text-xs font-semibold text-text-primary">
            Response History
          </span>
          <span className="rounded-full bg-primary/10 border border-primary/20 px-2 py-0.2 text-[10px] font-mono font-medium text-primary">
            {entries.length} {entries.length === 1 ? "run" : "runs"}
          </span>
        </div>

        <div className="flex items-center gap-2">
          {activeHistoryId && hasLatestLiveResponse && (
            <button
              onClick={onRestoreLatest}
              className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium text-primary hover:bg-primary/10 rounded-md transition-colors cursor-pointer"
            >
              <RotateCcw size={12} />
              <span>Back to Latest</span>
            </button>
          )}

          {confirmClear ? (
            <div className="flex items-center gap-1.5 animate-in fade-in duration-100">
              <span className="text-[11px] text-text-muted">Clear all?</span>
              <button
                onClick={() => {
                  onClearHistory();
                  setConfirmClear(false);
                }}
                className="px-2 py-0.5 rounded text-[11px] font-medium bg-error text-white hover:bg-error/90 cursor-pointer transition-colors"
              >
                Yes
              </button>
              <button
                onClick={() => setConfirmClear(false)}
                className="px-2 py-0.5 rounded text-[11px] font-medium bg-panel-raised text-text-secondary hover:text-text-primary cursor-pointer transition-colors"
              >
                Cancel
              </button>
            </div>
          ) : (
            <button
              onClick={() => setConfirmClear(true)}
              className="flex items-center gap-1 px-2.5 py-1 text-xs text-text-muted hover:text-error hover:bg-error/10 rounded-md transition-colors cursor-pointer"
              title="Clear all responses for this request"
            >
              <Trash2 size={12} />
              <span>Clear History</span>
            </button>
          )}
        </div>
      </div>

      {/* History Items List */}
      <div
        className={`flex-1 overflow-y-auto space-y-2 ${
          isMobile ? "p-2" : "p-3"
        }`}
      >
        {entries.map((entry, index) => {
          const isActive =
            activeHistoryId === entry.id || (!activeHistoryId && index === 0);
          const statusStyle = getStatusStyle(entry.status);
          const StatusIcon = statusStyle.icon;
          const sizeStr = entry.response?.sizeBytes
            ? formatBytes(entry.response.sizeBytes)
            : "";

          return (
            <div
              key={entry.id}
              onClick={() => onSelectEntry(entry)}
              className={`group relative flex flex-col gap-2 rounded-xl border p-3 cursor-pointer transition-all ${
                isActive
                  ? "border-primary/40 bg-panel-raised/90 shadow-xs ring-1 ring-primary/20"
                  : "border-border/70 bg-panel/40 hover:bg-panel-raised/60 hover:border-border"
              }`}
            >
              {/* Top Row: Method, Status, Latency, Timestamp & Actions */}
              <div className="flex items-center justify-between gap-2 min-w-0">
                <div className="flex items-center gap-2 flex-wrap min-w-0">
                  {/* Method Badge */}
                  <span
                    className={`font-mono text-[10px] font-bold px-1.5 py-0.5 rounded border border-current/20 ${
                      MethodStyles[entry.method] || "text-text-muted"
                    }`}
                  >
                    {entry.method}
                  </span>

                  {/* Status Pill */}
                  <span
                    className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-semibold border ${statusStyle.badge}`}
                  >
                    <StatusIcon size={12} />
                    <span>{entry.status}</span>
                    {entry.response?.statusText && (
                      <span className="font-normal opacity-90">
                        {entry.response.statusText}
                      </span>
                    )}
                  </span>

                  {/* Latency */}
                  <span className="flex items-center gap-1 text-[11px] font-mono text-text-muted">
                    <Zap size={11} className="text-text-muted/70" />
                    <span>{Number(entry.durationMs)} ms</span>
                  </span>

                  {/* Size if available */}
                  {sizeStr && (
                    <span className="text-[11px] font-mono text-text-muted">
                      • {sizeStr}
                    </span>
                  )}
                </div>

                {/* Right side: Timestamp, Active indicator & Delete button */}
                <div className="flex items-center gap-2 shrink-0">
                  {isActive && (
                    <span className="px-2 py-0.5 rounded-full bg-primary/20 text-primary text-[10px] font-semibold tracking-wide uppercase border border-primary/30">
                      Active
                    </span>
                  )}

                  <span
                    className="text-[11px] text-text-muted flex items-center gap-1"
                    title={formatFullTime(entry.createdAt)}
                  >
                    <Calendar size={11} className="text-text-muted/60" />
                    <span>{formatRelativeTime(entry.createdAt)}</span>
                  </span>

                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onDeleteEntry(entry.id);
                    }}
                    className="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-error/15 hover:text-error text-text-muted transition-all cursor-pointer"
                    title="Delete this historical response"
                    aria-label="Delete entry"
                  >
                    <Trash2 size={13} />
                  </button>
                </div>
              </div>

              {/* Bottom Row: URL & Snapshot preview hint */}
              <div className="flex items-center justify-between gap-3 text-xs text-text-muted min-w-0 pt-0.5">
                <span className="truncate font-mono text-[11px] text-text-muted/80">
                  {entry.url}
                </span>

                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onSelectEntry(entry);
                  }}
                  className="flex items-center gap-1 text-[11px] font-medium text-primary hover:underline shrink-0 cursor-pointer ml-auto"
                >
                  <Eye size={12} />
                  <span>Inspect</span>
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
