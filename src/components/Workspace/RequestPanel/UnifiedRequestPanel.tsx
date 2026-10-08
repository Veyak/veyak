import { useState, type ReactNode } from "react";
import { RequestTab } from "../../../types";
import { useVartaStore } from "../../../store/vartaStore";
import ParamsTab from "./tabs/ParamsTab";
import HeadersTab from "./tabs/HeadersTab";
import AuthTab from "./tabs/AuthTab";
import HttpBodyTab from "./tabs/HttpBodyTab";
import GraphQlQueryTab from "./tabs/GraphQlQueryTab";
import GrpcMessageTab from "./tabs/GrpcMessageTab";
import WsSavedTab from "./tabs/WsSavedTab";
import CookiesTab from "./tabs/CookiesTab";
import ScriptsTab from "./tabs/ScriptsTab";
import CodeEditor from "../../CodeEditor";

interface UnifiedRequestPanelProps {
  tab: RequestTab;
  isMobile?: boolean;
}

type SubTabId =
  | "params"
  | "headers"
  | "cookies"
  | "auth"
  | "body"
  | "scripts"
  | "saved"
  | "query"
  | "variables"
  | "message"
  | "metadata";

interface TabDefinition {
  id: SubTabId;
  label: string;
  badge?: ReactNode;
}

export default function UnifiedRequestPanel({
  tab,
  isMobile = false,
}: UnifiedRequestPanelProps) {
  const [subTab, setSubTab] = useState<SubTabId>("params");
  const updateActiveRequest = useVartaStore((s) => s.updateActiveRequest);

  // gRPC store
  const grpcMetadata = useVartaStore((s) => s.grpcMetadata);
  const setGrpcMetadata = useVartaStore((s) => s.setGrpcMetadata);

  // GraphQL store
  const graphqlHeaders = useVartaStore((s) => s.graphqlHeaders);
  const setGraphqlHeaders = useVartaStore((s) => s.setGraphqlHeaders);

  const isWs = tab.request.method === "WS";
  const isGrpc = tab.request.type === "grpc";
  const isGraphQl = tab.request.type === "graphql";

  // Safely extract HTTP request fields (Params, Headers, Cookies, Body) from union
  const httpReq = tab.request.type === "http" ? tab.request : null;

  // Build the dynamic subtab strip based on active protocol
  const visibleTabs: TabDefinition[] = (() => {
    if (isWs) {
      const activeParamCount = httpReq?.params?.filter((p) => p.key.trim()).length || 0;
      const activeHeaderCount = httpReq?.headers?.filter((h) => h.key.trim()).length || 0;
      const savedCount = tab.wsSavedMessages?.length || 0;

      return [
        {
          id: "params",
          label: "Params",
          badge: activeParamCount > 0 ? activeParamCount : undefined,
        },
        {
          id: "headers",
          label: "Headers",
          badge: activeHeaderCount > 0 ? activeHeaderCount : undefined,
        },
        {
          id: "saved",
          label: "Saved Messages",
          badge: savedCount > 0 ? savedCount : undefined,
        },
      ];
    }

    if (isGrpc) {
      const activeMetaCount = grpcMetadata.filter((m) => m.key.trim()).length;
      return [
        { id: "message", label: "Message" },
        {
          id: "metadata",
          label: "Metadata",
          badge: activeMetaCount > 0 ? activeMetaCount : undefined,
        },
        {
          id: "auth",
          label: "Authorization",
          badge:
            tab.request.auth && tab.request.auth.type !== "none"
              ? tab.request.auth.type
              : undefined,
        },
      ];
    }

    if (isGraphQl) {
      const activeHeaderCount = graphqlHeaders.filter((h) => h.key.trim()).length;
      return [
        { id: "query", label: "Query" },
        { id: "variables", label: "Variables" },
        {
          id: "headers",
          label: "Headers",
          badge: activeHeaderCount > 0 ? activeHeaderCount : undefined,
        },
        {
          id: "auth",
          label: "Authorization",
          badge:
            tab.request.auth && tab.request.auth.type !== "none"
              ? tab.request.auth.type
              : undefined,
        },
      ];
    }

    // Default: REST HTTP
    const activeParamCount = httpReq?.params?.filter((p) => p.key.trim()).length || 0;
    const activeHeaderCount = httpReq?.headers?.filter((h) => h.key.trim()).length || 0;
    const activeCookieCount = httpReq?.cookies?.length || 0;

    const hasScripts =
      Boolean(httpReq?.preRequestScript?.trim()) ||
      Boolean(httpReq?.postRequestScript?.trim());

    return [
      {
        id: "params",
        label: "Params",
        badge: activeParamCount > 0 ? activeParamCount : undefined,
      },
      {
        id: "headers",
        label: "Headers",
        badge: activeHeaderCount > 0 ? activeHeaderCount : undefined,
      },
      {
        id: "body",
        label: "Body",
        badge:
          httpReq?.body?.mode && httpReq.body.mode !== "raw"
            ? httpReq.body.mode
            : undefined,
      },
      {
        id: "auth",
        label: "Authorization",
        badge:
          tab.request.auth && tab.request.auth.type !== "none"
            ? tab.request.auth.type
            : undefined,
      },
      {
        id: "scripts",
        label: "Scripts",
        badge: hasScripts ? "●" : undefined,
      },
      {
        id: "cookies",
        label: "Cookies",
        badge: activeCookieCount > 0 ? activeCookieCount : undefined,
      },
    ];
  })();

  // Keep active tab valid if protocol changes
  const activeSubTab = visibleTabs.some((t) => t.id === subTab)
    ? subTab
    : visibleTabs[0].id;

  return (
    <div className="flex h-full flex-col min-w-0 bg-bg">
      {/* Dynamic Sub-tab Strip */}
      <div
        className={`flex gap-1 border-b border-border bg-panel/30 shrink-0 ${
          isMobile ? "overflow-x-auto scrollbar-hide px-2" : "px-4"
        }`}
      >
        {visibleTabs.map((t) => (
          <button
            key={t.id}
            onClick={() => setSubTab(t.id)}
            className={`tab-trigger shrink-0 flex items-center gap-1.5 ${
              activeSubTab === t.id ? "tab-trigger-active" : ""
            }`}
          >
            <span>{t.label}</span>
            {t.badge && (
              <span className="rounded-full bg-primary/20 px-1.5 py-0.2 text-[10px] font-mono text-primary">
                {t.badge}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Tab Contents Area */}
      <div className="flex-1 overflow-hidden min-h-0 min-w-0">
        {/* REST / WebSocket Params */}
        {activeSubTab === "params" && (
          <ParamsTab
            params={httpReq?.params ?? []}
            onChange={(params) => updateActiveRequest({ params })}
            isMobile={isMobile}
          />
        )}

        {/* REST / WebSocket Headers */}
        {activeSubTab === "headers" && !isGraphQl && (
          <HeadersTab
            headers={httpReq?.headers ?? []}
            onChange={(headers) => updateActiveRequest({ headers })}
            isMobile={isMobile}
          />
        )}

        {/* GraphQL Headers */}
        {activeSubTab === "headers" && isGraphQl && (
          <HeadersTab
            headers={graphqlHeaders}
            onChange={setGraphqlHeaders}
            isMobile={isMobile}
          />
        )}

        {/* gRPC Metadata */}
        {activeSubTab === "metadata" && (
          <HeadersTab
            headers={grpcMetadata}
            onChange={setGrpcMetadata}
            keyPlaceholder="Metadata key"
            valuePlaceholder="Value"
            suggestKeys={false}
            isMobile={isMobile}
          />
        )}

        {/* REST Body */}
        {activeSubTab === "body" && (
          <HttpBodyTab
            body={httpReq?.body}
            onChange={(body) => updateActiveRequest({ body })}
            isMobile={isMobile}
          />
        )}

        {/* REST / gRPC / GraphQL Auth */}
        {activeSubTab === "auth" && (
          <AuthTab
            auth={tab.request.auth}
            onChange={(auth) => updateActiveRequest({ auth })}
            isMobile={isMobile}
          />
        )}

        {/* REST Scripts (Pre/Post) */}
        {activeSubTab === "scripts" && (
          <ScriptsTab
            preRequestScript={httpReq?.preRequestScript}
            preRequestLanguage={httpReq?.preRequestLanguage}
            postRequestScript={httpReq?.postRequestScript}
            postRequestLanguage={httpReq?.postRequestLanguage}
            onChange={(patch) => updateActiveRequest(patch)}
            isMobile={isMobile}
          />
        )}

        {/* REST Cookies */}
        {activeSubTab === "cookies" && (
          <CookiesTab
            rows={httpReq?.cookies ?? []}
            onChange={(cookies) => updateActiveRequest({ cookies })}
            isMobile={isMobile}
          />
        )}

        {/* WebSocket Saved Messages */}
        {activeSubTab === "saved" && (
          <WsSavedTab tab={tab} isMobile={isMobile} />
        )}

        {/* GraphQL Query */}
        {activeSubTab === "query" && isGraphQl && (
          <GraphQlQueryTab
            query={(tab.request as any).query ?? "{\n  \n}"}
            onChangeQuery={(query) => updateActiveRequest({ query } as any)}
            variables={(tab.request as any).variables ?? "{}"}
            onChangeVariables={(variables) =>
              updateActiveRequest({ variables } as any)
            }
            isMobile={isMobile}
          />
        )}

        {/* GraphQL Variables */}
        {activeSubTab === "variables" && isGraphQl && (
          <div className="flex h-full flex-col min-h-0 overflow-hidden">
            <div className="border-b border-borderMuted bg-panel/40 px-4 py-1.5 text-[11px] font-mono text-text-muted">
              GraphQL Variables (JSON)
            </div>
            <div className="flex-1 min-h-0 overflow-hidden">
              <CodeEditor
                language="json"
                value={(tab.request as any).variables ?? "{}"}
                onChange={(variables) =>
                  updateActiveRequest({ variables } as any)
                }
                fontSize={isMobile ? 12.5 : undefined}
                lineNumbers
                placeholder="{\n  \n}"
              />
            </div>
          </div>
        )}

        {/* gRPC Message Payload */}
        {activeSubTab === "message" && isGrpc && (
          <GrpcMessageTab isMobile={isMobile} />
        )}
      </div>
    </div>
  );
}
