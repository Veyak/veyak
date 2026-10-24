import { useMemo } from "react";
import { AlertCircle, Loader2, Send } from "lucide-react";
import { RequestTab } from "../../../types";
import { useVartaStore } from "../../../store/vartaStore";
import StaticResponseViewer from "./StaticResponseViewer";
import StreamingTimelineViewer from "./StreamingTimelineViewer";

interface UnifiedResponsePanelProps {
  tab: RequestTab;
  isMobile?: boolean;
}

export default function UnifiedResponsePanel({
  tab,
  isMobile = false,
}: UnifiedResponsePanelProps) {
  const grpcCallStatus = useVartaStore((s) => s.grpcCallStatus);
  const grpcMessages = useVartaStore((s) => s.grpcMessages);
  const grpcSelectedMethod = useVartaStore((s) => s.grpcSelectedMethod);
  const historyEntries = useVartaStore((s) => s.historyEntries);

  const requestHistory = useMemo(() => {
    return historyEntries.filter((h) => h.requestId === tab.request.id);
  }, [historyEntries, tab.request.id]);

  const isWs = tab.request.method === "WS";
  const isGrpc = tab.request.type === "grpc";
  const isGraphQl = tab.request.type === "graphql";

  // Check if call is actively executing/sending
  const isSending =
    tab.isSending ||
    tab.graphqlCallStatus === "sending" ||
    grpcCallStatus === "invoking";

  // Check if this response falls into Mode B (Streaming Timeline)
  const isStreamingMode = useMemo(() => {
    if (isWs) return true;

    if (isGrpc) {
      const isStreamMethod =
        grpcSelectedMethod && grpcSelectedMethod.streamType !== "unary";
      const isStreamingStatus = grpcCallStatus === "streaming";
      const hasMultipleMessages = grpcMessages.length > 1;
      return Boolean(isStreamMethod || isStreamingStatus || hasMultipleMessages);
    }

    if (isGraphQl) {
      const isSub =
        (tab.request as any).requestType === "subscription" ||
        tab.graphqlCallStatus === "streaming" ||
        (tab.graphqlSubscriptionMessages && tab.graphqlSubscriptionMessages.length > 0);
      return Boolean(isSub);
    }

    return false;
  }, [isWs, isGrpc, grpcSelectedMethod, grpcCallStatus, grpcMessages.length, isGraphQl, tab.request, tab.graphqlCallStatus, tab.graphqlSubscriptionMessages]);

  // Loading state
  if (isSending) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 p-6 text-center text-text-secondary bg-bg">
        <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10 border border-primary/20">
          <Loader2 size={24} className="animate-spin text-primary" />
        </div>
        <div className="space-y-1">
          <h3 className="text-sm font-semibold text-text-primary">
            {isGrpc ? "Invoking RPC Method…" : isGraphQl ? "Executing GraphQL Query…" : "Sending Request…"}
          </h3>
          <p className="text-xs text-text-muted">
            Waiting for response from endpoint…
          </p>
        </div>
      </div>
    );
  }

  // Error view state
  if (tab.error) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 p-6 text-center bg-bg">
        <div className="flex h-12 w-12 items-center justify-center rounded-full bg-error/10 border border-error/20">
          <AlertCircle className="text-error" size={24} />
        </div>
        <div className="space-y-1">
          <h3 className="text-sm font-semibold text-text-primary">
            Request Failed
          </h3>
          <p className="max-w-md font-mono text-xs text-error bg-error/10 border border-error/20 rounded-md px-3 py-2 whitespace-pre-wrap">
            {tab.error}
          </p>
        </div>
      </div>
    );
  }

  // If streaming mode, render the streaming timeline
  if (isStreamingMode) {
    return <StreamingTimelineViewer tab={tab} isMobile={isMobile} />;
  }

  // Empty state for Static mode if no response has been received yet and no history exists
  const hasStaticResponse =
    (tab.response && (tab.response.status > 0 || tab.response.body)) ||
    tab.graphqlResponse ||
    (isGrpc && grpcMessages.length > 0) ||
    requestHistory.length > 0;

  if (!hasStaticResponse) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 p-6 text-center text-text-muted bg-bg">
        <div className="flex h-12 w-12 items-center justify-center rounded-full bg-panel border border-border">
          <Send size={20} className="text-text-muted/60" />
        </div>
        <div className="space-y-1">
          <h4 className="text-sm font-medium text-text-primary">No Response Yet</h4>
          <p className="text-xs text-text-muted max-w-xs">
            Send a request to inspect response status, headers, and payload here.
          </p>
        </div>
      </div>
    );
  }

  // Mode A: Static Single Response Viewer
  return <StaticResponseViewer tab={tab} isMobile={isMobile} />;
}
