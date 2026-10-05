import { create } from "zustand";
import {
  RequestTab,
  WsMessage,
  GrpcMessage,
  GrpcService,
  GrpcMethod,
  GrpcMetadataRow,
  GrpcCallStatus,
  GraphQlCallStatus,
  GraphQlHeaderRow,
  GraphQlResponse,
  GraphQlSchema,
  GraphQlSubscriptionMessage,
} from "../types";
import { invoke } from "@tauri-apps/api/core";
import { sendNativeRequest } from "../services/rest";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useWorkspaceStore } from "./workspaceStore";
import {
  HistoryEntry,
  RequestItem,
  WsSavedMessage,
} from "@veyak-internal/models";

interface VartaState {
  tabs: RequestTab[];
  activeTabId: string | null;
  activeTab: RequestTab | null;
  isCommandPaletteOpen: boolean;
  isHistoryOpen: boolean;
  activeEnvId: string;
  isEnvEditorOpen: boolean;
  isSidebarOpen: boolean;
  historyEntries: HistoryEntry[];
  isNewReqSaveOpen: boolean;

  openRequest: (request: RequestItem) => void;
  newTab: () => void;
  closeTab: (tabId: string) => void;
  setActiveTab: (tabId: string) => void;
  updateActiveRequest: (patch: Partial<RequestItem>) => void;
  sendActiveRequest: () => void;
  saveActiveRequest: () => void;
  toggleCommandPalette: (open?: boolean) => void;
  toggleHistory: (open?: boolean) => void;
  setEnv: (id: string) => void;
  toggleSidebar: (open?: boolean) => void;
  setSidebarOpen: (open: boolean) => void;

  setIsNewReqSaveOpen: (open: boolean) => void;
  closeNewReqSave: () => void;

  openEnvEditor: () => void;
  closeEnvEditor: () => void;

  // WebSocket actions
  connectWebSocket: () => Promise<void>;
  disconnectWebSocket: () => Promise<void>;
  sendWsMessage: (message: string) => Promise<void>;
  addWsMessage: (msg: WsMessage) => void;
  clearWsMessages: (tabId?: string) => void;
  loadSavedMessages: (requestId: string) => Promise<void>;
  addSavedMessage: (
    requestId: string,
    name: string,
    data: string,
  ) => Promise<void>;
  deleteSavedMessage: (requestId: string, messageId: string) => Promise<void>;
  setWsProtocol: (protocol: "raw" | "graphql-ws") => void;
  initWsListener: () => Promise<UnlistenFn>;
  // HistoryEntry
  fetchHistory: () => Promise<void>;
  deleteHistoryEntry: (id: string) => Promise<void>;
  clearHistory: () => Promise<void>;

  // ── gRPC state ────────────────────────────────────────────────────────
  // grpcServerAddress: string;
  grpcTlsEnabled: boolean;
  grpcCallStatus: GrpcCallStatus;
  grpcMessages: GrpcMessage[];
  grpcServices: GrpcService[];
  grpcSelectedService: GrpcService | null;
  grpcSelectedMethod: GrpcMethod | null;
  grpcRequestBody: string;
  grpcMetadata: GrpcMetadataRow[];
  grpcReflectionLoading: boolean;
  grpcLastLatencyMs: number | null;
  grpcConnectionId: string | null;
  grpcResponseMetadata: Record<string, string>;
  grpcResponseStatus: { code: number; text: string } | null;

  // ── gRPC actions ────────────────
  setGrpcTlsEnabled: (enabled: boolean) => void;
  setGrpcSelectedService: (service: GrpcService | null) => void;
  setGrpcSelectedMethod: (method: GrpcMethod | null) => void;
  setGrpcRequestBody: (body: string) => void;
  setGrpcMetadata: (rows: GrpcMetadataRow[]) => void;
  setGrpcResponseMetadata: (metadata: Record<string, string>) => void;
  addGrpcMessage: (msg: GrpcMessage) => void;
  clearGrpcMessages: () => void;
  setGrpcCallStatus: (status: GrpcCallStatus) => void;
  setGrpcServices: (services: GrpcService[]) => void;
  setGrpcReflectionLoading: (loading: boolean) => void;
  invokeGrpc: () => Promise<void>;
  sendGrpcMessage: (msg: string) => Promise<void>;
  cancelGrpcCall: () => Promise<void>;
  loadGrpcReflection: () => Promise<void>;
  initGrpcListener: () => Promise<UnlistenFn>;

  // ── GraphQL state ─────────────────────────────────────────────────────
  graphqlCallStatus: GraphQlCallStatus;
  graphqlResponse: GraphQlResponse | null;
  graphqlSchema: GraphQlSchema | null;
  graphqlSchemaLoading: boolean;
  graphqlSubscriptionMessages: GraphQlSubscriptionMessage[];
  graphqlConnectionId: string | null;
  graphqlHeaders: GraphQlHeaderRow[];

  // ── GraphQL actions ──────────────────────────────────────────────────
  setGraphqlCallStatus: (s: GraphQlCallStatus) => void;
  setGraphqlResponse: (r: GraphQlResponse | null) => void;
  setGraphqlSchema: (schema: GraphQlSchema | null) => void;
  setGraphqlHeaders: (rows: GraphQlHeaderRow[]) => void;
  addGraphqlSubscriptionMessage: (msg: GraphQlSubscriptionMessage) => void;
  clearGraphqlMessages: (tabId?: string) => void;
  loadGraphqlSchema: (targetUrl?: string) => Promise<void>;
  invokeGraphql: (targetTabId?: string) => Promise<void>;
  subscribeGraphql: (targetTabId?: string) => Promise<void>;
  cancelGraphqlSubscription: (targetTabId?: string) => Promise<void>;
  initGraphqlListener: () => Promise<UnlistenFn>;
}

let tabCounter = 0;

function blankRequest(): RequestItem {
  tabCounter += 1;
  return {
    id: `new-${tabCounter}`,
    name: "Untitled request",
    method: "GET",
    url: "",
    type: "http",
    params: [{ id: "p1", key: "", value: "", enabled: true }],
    headers: [{ id: "h1", key: "", value: "", enabled: true }],
    cookies: [],
    auth: { type: "none", basic: null, bearer: null, apiKey: null },
    body: { raw: "", mode: null, formData: [], urlEncoded: [], files: [] },
    collectionId: "",
    folderId: null,
    sortOrder: 0 as any,
  };
}

export const useVartaStore = create<VartaState>((set, get) => ({
  tabs: [],
  activeTabId: null,
  isCommandPaletteOpen: false,
  isHistoryOpen: false,
  activeEnvId: "env-staging",
  isEnvEditorOpen: false,
  isSidebarOpen: false,
  activeTab: null,
  historyEntries: [],
  isNewReqSaveOpen: false,

  // ── gRPC initial state ───────────────────────────────────────────────
  grpcTlsEnabled: false,
  grpcCallStatus: "idle",
  grpcMessages: [],
  grpcServices: [],
  grpcSelectedService: null,
  grpcSelectedMethod: null,
  grpcRequestBody: "{\n  \n}",
  grpcMetadata: [{ id: "m1", key: "", value: "", enabled: true }],
  grpcReflectionLoading: false,
  grpcLastLatencyMs: null,
  grpcConnectionId: null,
  grpcResponseMetadata: {},
  grpcResponseStatus: null,

  // ── GraphQL initial state ────────────────────────────────────────────
  graphqlCallStatus: "idle",
  graphqlResponse: null,
  graphqlSchema: null,
  graphqlSchemaLoading: false,
  graphqlSubscriptionMessages: [],
  graphqlConnectionId: null,
  graphqlHeaders: [{ id: "gh1", key: "", value: "", enabled: true }],

  setIsNewReqSaveOpen: (open) => set({ isNewReqSaveOpen: open }),
  closeNewReqSave: () => set({ isNewReqSaveOpen: false }),

  toggleSidebar: (open) =>
    set((s) => ({
      isSidebarOpen: open !== undefined ? open : !s.isSidebarOpen,
    })),
  setSidebarOpen: (open) => set({ isSidebarOpen: open }),

  openEnvEditor: () => set({ isEnvEditorOpen: true }),

  closeEnvEditor: () => set({ isEnvEditorOpen: false }),

  openRequest: (request) => {
    const existing = get().tabs.find((t) => t.request.id === request.id);
    if (existing) {
      set({ activeTabId: existing.id, activeTab: existing });
      return;
    }
    const tab: RequestTab = {
      id: request.id,
      request,
      isDirty: false,
      isSending: false,
      wsMessages: [],
      wsStatus: "disconnected",
      wsSavedMessages: [],
      wsProtocol: "raw",
      wsGqlSubscriptionIds: [],
      // Initialize tab-isolated GraphQL state:
      graphqlCallStatus: "idle",
      graphqlResponse: null,
      graphqlSubscriptionMessages: [],
      graphqlConnectionId: null,
    };
    set((s) => ({
      tabs: [...s.tabs, tab],
      activeTabId: tab.id,
      activeTab: tab,
    }));
  },

  newTab: () => {
    const request = blankRequest();
    const tab: RequestTab = {
      id: request.id,
      request,
      isDirty: false,
      isSending: false,
      wsMessages: [],
      wsStatus: "disconnected",
      wsSavedMessages: [],
      wsProtocol: "raw",
      wsGqlSubscriptionIds: [],
      // Initialize tab-isolated GraphQL state:
      graphqlCallStatus: "idle",
      graphqlResponse: null,
      graphqlSubscriptionMessages: [],
      graphqlConnectionId: null,
    };
    set((s) => ({ tabs: [...s.tabs, tab], activeTabId: tab.id }));
  },

  closeTab: (tabId) => {
    set((s) => {
      const remaining = s.tabs.filter((t) => t.id !== tabId);
      const wasActive = s.activeTabId === tabId;
      return {
        tabs: remaining,
        activeTabId: wasActive
          ? (remaining[remaining.length - 1]?.id ?? null)
          : s.activeTabId,
      };
    });
  },

  setActiveTab: (tabId) => set({ activeTabId: tabId }),

  updateActiveRequest: (patch) => {
    set((s) => {
      const updatedTabs = s.tabs.map((t) =>
        t.id === s.activeTabId
          ? {
            ...t,
            isDirty: true,
            request: { ...t.request, ...patch } as RequestItem,
          }
          : t,
      );

      return {
        tabs: updatedTabs,
        // Keep the activeTab object in sync with the array
        activeTab: updatedTabs.find((t) => t.id === s.activeTabId) || null,
      };
    });
  },

  sendActiveRequest: () => {
    const { activeTabId } = get();
    if (!activeTabId) return;

    set((s) => {
      const updatedTabs = s.tabs.map((t) =>
        t.id === activeTabId ? { ...t, isSending: true } : t,
      );
      return {
        tabs: updatedTabs,
        activeTab: updatedTabs.find((t) => t.id === s.activeTabId) || null,
      };
    });

    // Placeholder for the real call — in the Tauri app this invokes a Rust
    // command (e.g. `invoke("send_request", { request })`) instead of fetch,
    // so requests aren't subject to browser CORS restrictions.
    setTimeout(async () => {
      const { tabs, activeTabId } = get();
      if (!activeTabId) return;
      const payload = tabs.find((t) => t.id === activeTabId)?.request;
      if (!payload) return;

      try {
        const res = await sendNativeRequest(payload);
        console.log(res);
        set((s) => {
          const updatedTabs = s.tabs.map((t) =>
            t.id === activeTabId
              ? { ...t, isSending: false, response: res, error: undefined }
              : t,
          );
          return {
            tabs: updatedTabs,
            activeTab: updatedTabs.find((t) => t.id === s.activeTabId) || null,
          };
        });
        // setResponse(res);
        get().fetchHistory();
      } catch (err) {
        console.error(err);

        const errorMessage = err instanceof Error ? err.message : String(err);
        set((s) => {
          const updatedTabs = s.tabs.map((t) =>
            t.id === activeTabId
              ? {
                ...t,
                isSending: false,
                response: undefined,
                error: errorMessage,
              }
              : t,
          );
          return {
            tabs: updatedTabs,
            activeTab: updatedTabs.find((t) => t.id === s.activeTabId) || null,
          };
        });
        // setError(String(err));
      }
    }, 600);
  },

  toggleCommandPalette: (open) =>
    set((s) => ({ isCommandPaletteOpen: open ?? !s.isCommandPaletteOpen })),

  toggleHistory: (open) =>
    set((s) => ({ isHistoryOpen: open ?? !s.isHistoryOpen })),

  setEnv: (id) => set({ activeEnvId: id }),

  saveActiveRequest: async () => {
    const state = get();
    const activeTab = state.tabs.find((t) => t.id === state.activeTabId);

    if (!activeTab || !activeTab.isDirty) return;
    if (activeTab?.request.id.startsWith("new-")) {
      if (activeTab.request.collectionId === "") {
        set({ activeTab: activeTab, isNewReqSaveOpen: true });
        return;
      } else {
        activeTab.request.id = crypto.randomUUID();
      }
    }

    try {
      // Assuming you have this command in your Rust backend
      const res = await invoke("save_request", { request: activeTab.request });
      console.log("Request saved:", res);
      // Clear dirty flag on success
      set((state) => ({
        tabs: state.tabs.map((t) =>
          t.id === state.activeTabId ? { ...t, isDirty: false } : t,
        ),
      }));
      // Refresh the specific collection tree or collection list
      const targetColId = activeTab.request.collectionId;
      if (targetColId) {
        useWorkspaceStore.getState().fetchCollectionTree(targetColId);
      } else {
        useWorkspaceStore.getState().fetchCollections();
      }
    } catch (error) {
      console.error("Failed to save request:", error);
      // Handle error toast here
    }
  },

  // ── WebSocket actions ──────────────────────────────────────────────

  connectWebSocket: async () => {
    const { activeTabId, tabs } = get();
    if (!activeTabId) return;
    const tab = tabs.find((t) => t.id === activeTabId);
    if (!tab) return;
    if (tab.request.type === "grpc") return;

    // Set connecting state
    set((s) => ({
      tabs: s.tabs.map((t) =>
        t.id === activeTabId
          ? {
            ...t,
            wsStatus: "connecting" as const,
            wsMessages: [],
            wsGqlSubscriptionIds: [],
          }
          : t,
      ),
    }));

    try {
      // Build headers from the request
      const headers: [string, string][] = tab.request.headers
        .filter((h) => h.enabled && h.key)
        .map((h) => [h.key, h.value] as [string, string]);

      // Auto-inject graphql-transport-ws sub-protocol header
      if (tab.wsProtocol === "graphql-ws") {
        const hasProto = headers.some(
          ([k]) => k.toLowerCase() === "sec-websocket-protocol",
        );
        if (!hasProto) {
          headers.push(["Sec-WebSocket-Protocol", "graphql-transport-ws"]);
        }
      }

      const connectionId = await invoke<string>("ws_connect", {
        url: tab.request.url,
        headers,
      });

      set((s) => ({
        tabs: s.tabs.map((t) =>
          t.id === activeTabId
            ? {
              ...t,
              wsConnectionId: connectionId,
              wsStatus: "connected" as const,
            }
            : t,
        ),
      }));

      // Send connection_init for graphql-ws protocol
      if (tab.wsProtocol === "graphql-ws") {
        await invoke("ws_send", {
          connectionId,
          message: JSON.stringify({ type: "connection_init" }),
        });
      }
    } catch (err) {
      console.error("WS connect failed:", err);
      set((s) => ({
        tabs: s.tabs.map((t) =>
          t.id === activeTabId
            ? { ...t, wsStatus: "disconnected" as const, error: String(err) }
            : t,
        ),
      }));
    }
  },

  disconnectWebSocket: async () => {
    const { activeTabId, tabs } = get();
    if (!activeTabId) return;
    const tab = tabs.find((t) => t.id === activeTabId);
    if (!tab?.wsConnectionId) return;

    try {
      await invoke("ws_disconnect", { connectionId: tab.wsConnectionId });
    } catch (err) {
      console.error("WS disconnect failed:", err);
    }
    set((s) => ({
      tabs: s.tabs.map((t) =>
        t.id === activeTabId
          ? {
            ...t,
            wsConnectionId: undefined,
            wsStatus: "disconnected" as const,
          }
          : t,
      ),
    }));
  },

  sendWsMessage: async (message: string) => {
    const { activeTabId, tabs } = get();
    if (!activeTabId) return;
    const tab = tabs.find((t) => t.id === activeTabId);
    if (!tab?.wsConnectionId) return;

    try {
      let payload = message;

      // Wrap in graphql-ws framing if using the protocol
      if (tab.wsProtocol === "graphql-ws") {
        const subId = crypto.randomUUID();
        // Track subscription ID for cleanup
        set((s) => ({
          tabs: s.tabs.map((t) =>
            t.id === activeTabId
              ? {
                ...t,
                wsGqlSubscriptionIds: [...t.wsGqlSubscriptionIds, subId],
              }
              : t,
          ),
        }));

        try {
          const parsed = JSON.parse(message);
          payload = JSON.stringify({
            id: subId,
            type: "subscribe",
            payload: parsed,
          });
        } catch {
          // If message isn't valid JSON, send it as-is
          payload = message;
        }
      }

      await invoke("ws_send", {
        connectionId: tab.wsConnectionId,
        message: payload,
      });
    } catch (err) {
      console.error("WS send failed:", err);
    }
  },

  setWsProtocol: (protocol: "raw" | "graphql-ws") => {
    const { activeTabId } = get();
    if (!activeTabId) return;
    set((s) => ({
      tabs: s.tabs.map((t) =>
        t.id === activeTabId ? { ...t, wsProtocol: protocol } : t,
      ),
    }));
  },

  addWsMessage: (msg: WsMessage) => {
    set((s) => ({
      tabs: s.tabs.map((t) => {
        if (t.wsConnectionId === msg.connectionId) {
          // If it's a close event, also update status
          if (msg.direction === "closed") {
            return {
              ...t,
              wsMessages: [...t.wsMessages, msg],
              wsStatus: "disconnected" as const,
              wsConnectionId: undefined,
            };
          }
          return { ...t, wsMessages: [...t.wsMessages, msg] };
        }
        return t;
      }),
    }));
  },

  clearWsMessages: (tabId?: string) => {
    const targetId = tabId || get().activeTabId;
    if (!targetId) return;
    set((s) => {
      const updatedTabs = s.tabs.map((t) =>
        t.id === targetId ? { ...t, wsMessages: [] } : t,
      );
      return {
        tabs: updatedTabs,
        activeTab: updatedTabs.find((t) => t.id === s.activeTabId) || null,
      };
    });
  },

  loadSavedMessages: async (requestId: string) => {
    try {
      const messages = await invoke<WsSavedMessage[]>(
        "ws_list_saved_messages",
        { requestId },
      );
      set((s) => ({
        tabs: s.tabs.map((t) =>
          t.request.id === requestId ? { ...t, wsSavedMessages: messages } : t,
        ),
      }));
    } catch (err) {
      console.error("Failed to load saved messages:", err);
    }
  },

  addSavedMessage: async (requestId: string, name: string, data: string) => {
    try {
      const msg = await invoke<WsSavedMessage>("ws_add_saved_message", {
        requestId,
        name,
        data,
      });
      set((s) => ({
        tabs: s.tabs.map((t) =>
          t.request.id === requestId
            ? { ...t, wsSavedMessages: [...t.wsSavedMessages, msg] }
            : t,
        ),
      }));
    } catch (err) {
      console.error("Failed to add saved message:", err);
    }
  },

  deleteSavedMessage: async (requestId: string, messageId: string) => {
    try {
      await invoke("ws_delete_saved_message", { requestId, messageId });
      set((s) => ({
        tabs: s.tabs.map((t) =>
          t.request.id === requestId
            ? {
              ...t,
              wsSavedMessages: t.wsSavedMessages.filter(
                (m) => m.id !== messageId,
              ),
            }
            : t,
        ),
      }));
    } catch (err) {
      console.error("Failed to delete saved message:", err);
    }
  },

  initWsListener: async () => {
    const unlisten = await listen<WsMessage>("ws://message", (event) => {
      get().addWsMessage(event.payload);
    });
    // Also listen for status changes (disconnect from server side)
    const unlistenStatus = await listen<{
      connectionId: string;
      status: string;
    }>("ws://status", (event) => {
      if (event.payload.status === "disconnected") {
        set((s) => ({
          tabs: s.tabs.map((t) =>
            t.wsConnectionId === event.payload.connectionId
              ? {
                ...t,
                wsConnectionId: undefined,
                wsStatus: "disconnected" as const,
              }
              : t,
          ),
        }));
      }
    });
    return () => {
      unlisten();
      unlistenStatus();
    };
  },

  fetchHistory: async () => {
    try {
      const historyEntries = await invoke<HistoryEntry[]>("list_history", {
        limit: 100,
      });
      console.log("Fetched history entries:", historyEntries);
      set({ historyEntries: historyEntries });
    } catch (error) {
      console.error("Failed to load history:", error);
    }
  },

  clearHistory: async () => {
    try {
      await invoke("clear_history");
      set({ historyEntries: [] });
    } catch (error) {
      console.error("Failed to clear history:", error);
    }
  },

  deleteHistoryEntry: async (id: string) => {
    try {
      await invoke("delete_history_entry", { id });
      set((state) => ({
        historyEntries: state.historyEntries.filter((entry) => entry.id !== id),
      }));
    } catch (error) {
      console.error("Failed to delete history entry:", error);
    }
  },

  // ── gRPC action implementations ──────────────────────────────────────
  // setGrpcServerAddress: (addr) => set({ grpcServerAddress: addr }),
  setGrpcTlsEnabled: (enabled) => set({ grpcTlsEnabled: enabled }),
  setGrpcSelectedService: (service) =>
    set({ grpcSelectedService: service, grpcSelectedMethod: null }),
  setGrpcSelectedMethod: (method) => set({ grpcSelectedMethod: method }),
  setGrpcRequestBody: (body) => set({ grpcRequestBody: body }),
  setGrpcMetadata: (rows) => set({ grpcMetadata: rows }),
  setGrpcResponseMetadata: (metadata) =>
    set({ grpcResponseMetadata: metadata }),
  addGrpcMessage: (msg) =>
    set((s) => ({ grpcMessages: [...s.grpcMessages, msg] })),
  clearGrpcMessages: () =>
    set({
      grpcMessages: [],
      grpcLastLatencyMs: null,
      grpcResponseMetadata: {},
      grpcResponseStatus: null,
    }),
  setGrpcCallStatus: (status) => set({ grpcCallStatus: status }),
  setGrpcServices: (services) => set({ grpcServices: services }),
  setGrpcReflectionLoading: (loading) =>
    set({ grpcReflectionLoading: loading }),

  invokeGrpc: async () => {
    const {
      grpcSelectedMethod,
      grpcCallStatus,
      grpcSelectedService,
      grpcRequestBody,
      grpcMetadata,
      activeTab,
    } = get();
    if (
      !grpcSelectedMethod ||
      !activeTab?.request.url ||
      !grpcSelectedService ||
      grpcCallStatus === "invoking" ||
      grpcCallStatus === "streaming"
    )
      return;

    const isStreaming =
      grpcSelectedMethod.streamType === "server_stream" ||
      grpcSelectedMethod.streamType === "client_stream" ||
      grpcSelectedMethod.streamType === "bidi_stream";

    set({
      grpcCallStatus: isStreaming ? "streaming" : "invoking",
      grpcMessages: [],
      grpcLastLatencyMs: null,
      grpcConnectionId: null,
      grpcResponseMetadata: {},
      grpcResponseStatus: null,
    });

    // Map the local stream type to the Rust GrpcMethodType enum
    const streamToMethodType: Record<string, string> = {
      unary: "Unary",
      server_stream: "ServerStreaming",
      client_stream: "ClientStreaming",
      bidi_stream: "BidirectionalStreaming",
    };

    try {
      const request = {
        id: "temp-id",
        collectionId: "temp-collection",
        folderId: null,
        name: "temp-name",
        url: activeTab.request.url,
        service: grpcSelectedService.fullName,
        method: grpcSelectedMethod.name,
        methodType: streamToMethodType[grpcSelectedMethod.streamType] || "Unary",
        metadata: grpcMetadata,
        auth: { type: "none", basic: null, bearer: null, apiKey: null },
        message: grpcRequestBody,
        useReflection: true,
        protoFileIds: [],
      };

      // If client streaming or bidi streaming and an initial message is provided, log it as sent
      if (
        (grpcSelectedMethod.streamType === "client_stream" ||
          grpcSelectedMethod.streamType === "bidi_stream") &&
        grpcRequestBody.trim() &&
        grpcRequestBody.trim() !== "{}"
      ) {
        get().addGrpcMessage({
          id: crypto.randomUUID(),
          direction: "sent",
          data: grpcRequestBody,
          timestamp: new Date().toISOString(),
        });
      }

      const response = await invoke<any>("grpc_invoke", { request });

      if (isStreaming) {
        const connectionId = response.metadata?.["x-veyak-connection-id"];
        set({
          grpcConnectionId: connectionId || null,
          grpcCallStatus: "streaming",
          grpcLastLatencyMs: Number(response.timeMs),
          grpcResponseMetadata: response.metadata || {},
          grpcResponseStatus: {
            code: response.status ?? 0,
            text: response.statusText || "Streaming",
          },
        });
      } else {
        const isError = response.status !== 0;
        set({
          grpcCallStatus: isError ? "error" : "ok",
          grpcLastLatencyMs: Number(response.timeMs),
          grpcResponseMetadata: response.metadata || {},
          grpcResponseStatus: {
            code: response.status ?? 0,
            text: response.statusText || "OK",
          },
          grpcMessages: [
            {
              id: crypto.randomUUID(),
              direction: "received",
              data: response.message,
              timestamp: new Date().toISOString(),
              statusCode: response.statusText || (isError ? "ERROR" : "OK"),
              latencyMs: Number(response.timeMs),
              isError,
            },
          ],
        });
      }
    } catch (e: any) {
      console.error("[gRPC] invokeGrpc error:", e);
      set({
        grpcCallStatus: "error",
        grpcConnectionId: null,
        grpcResponseStatus: { code: 2, text: "ERROR" },
        grpcMessages: [
          {
            id: crypto.randomUUID(),
            direction: "received",
            data: typeof e === "string" ? e : JSON.stringify(e, null, 2),
            timestamp: new Date().toISOString(),
            isError: true,
            statusCode: "ERROR",
          },
        ],
      });
    }
  },

  sendGrpcMessage: async (msg: string) => {
    const { grpcConnectionId } = get();
    if (!grpcConnectionId) return;
    try {
      await invoke("grpc_send_message", {
        connectionId: grpcConnectionId,
        message: msg,
      });
    } catch (err: any) {
      console.error("[gRPC] sendGrpcMessage error:", err);
      get().addGrpcMessage({
        id: crypto.randomUUID(),
        direction: "received",
        data: `Error sending message: ${err}`,
        timestamp: new Date().toISOString(),
        isError: true,
        statusCode: "ERROR",
      });
    }
  },

  cancelGrpcCall: async () => {
    const { grpcConnectionId } = get();
    if (grpcConnectionId) {
      try {
        await invoke("grpc_cancel", { connectionId: grpcConnectionId });
      } catch (err) {
        console.error("[gRPC] cancel error:", err);
      }
    }
    set({
      grpcCallStatus: "cancelled",
      grpcConnectionId: null,
      grpcResponseStatus: { code: 1, text: "CANCELLED" },
    });
    setTimeout(() => set({ grpcCallStatus: "idle" }), 800);
  },

  initGrpcListener: async () => {
    const unlistenMsg = await listen<{
      connectionId: string;
      direction: string;
      message: string;
      timestamp: string;
    }>("grpc://message", (event) => {
      const { connectionId, direction, message, timestamp } = event.payload;
      const { grpcConnectionId } = get();

      // Accept message if it matches active stream connection
      if (!grpcConnectionId || grpcConnectionId === connectionId) {
        const isError =
          direction === "received" &&
          (message.startsWith("Error:") || message.includes("gRPC Error"));
        get().addGrpcMessage({
          id: crypto.randomUUID(),
          connectionId,
          direction: direction as "sent" | "received" | "closed",
          data: message,
          timestamp: timestamp || new Date().toISOString(),
          isError,
          statusCode: isError ? "ERROR" : undefined,
        });
      }
    });

    const unlistenMeta = await listen<{
      connectionId: string;
      metadata: Record<string, string>;
    }>("grpc://metadata", (event) => {
      const { connectionId, metadata } = event.payload;
      const { grpcConnectionId } = get();
      if (!grpcConnectionId || grpcConnectionId === connectionId) {
        set((s) => ({
          grpcResponseMetadata: { ...s.grpcResponseMetadata, ...metadata },
        }));
      }
    });

    const unlistenStatus = await listen<{
      connectionId: string;
      status: string;
      error?: string;
      statusCode?: number;
      statusText?: string;
      metadata?: Record<string, string>;
    }>("grpc://status", (event) => {
      const { connectionId, status, statusCode, statusText, metadata } =
        event.payload;
      const { grpcConnectionId } = get();
      if (!grpcConnectionId || grpcConnectionId === connectionId) {
        if (metadata && Object.keys(metadata).length > 0) {
          set((s) => ({
            grpcResponseMetadata: { ...s.grpcResponseMetadata, ...metadata },
          }));
        }
        if (status === "closed") {
          set({
            grpcCallStatus: "ok",
            grpcResponseStatus: {
              code: statusCode ?? 0,
              text: statusText ?? "OK",
            },
          });
        } else if (status === "cancelled") {
          set({
            grpcCallStatus: "cancelled",
            grpcResponseStatus: { code: 1, text: "CANCELLED" },
          });
        } else if (status === "error") {
          set({
            grpcCallStatus: "error",
            grpcResponseStatus: {
              code: statusCode ?? 2,
              text: statusText ?? "ERROR",
            },
          });
        }
      }
    });

    return () => {
      unlistenMsg();
      unlistenMeta();
      unlistenStatus();
    };
  },

  loadGrpcReflection: async () => {
    const { activeTab, activeTabId } = get();
    const grpcServerAddress = activeTab?.request.url;
    if (!grpcServerAddress) return;
    set({ grpcReflectionLoading: true, grpcServices: [] });
    try {
      console.log("grpc_reflect");
      const services = await invoke<GrpcService[]>("grpc_reflect", {
        address: grpcServerAddress,
      });
      set({ grpcServices: services, grpcCallStatus: "idle", grpcMessages: [] });
    } catch (e: any) {
      console.error("[gRPC] loadGrpcReflection error:", e);
      set((s) => ({
        tabs: s.tabs.map((t) =>
          t.id === activeTabId
            ? {
              ...t,
              isSending: false,
              response: undefined,
              error: e,
            }
            : t,
        ),
        grpcCallStatus: "error",
        grpcMessages: [
          {
            id: crypto.randomUUID(),
            direction: "received",
            data: typeof e === "string" ? e : JSON.stringify(e, null, 2),
            timestamp: new Date().toISOString(),
            isError: true,
            statusCode: "ERROR",
          },
        ],
      }));
    } finally {
      set({ grpcReflectionLoading: false });
    }
  },

  // ── GraphQL actions ───────────────────────────────────────────────────

  setGraphqlCallStatus: (s) => set({ graphqlCallStatus: s }),
  setGraphqlResponse: (r) => set({ graphqlResponse: r }),
  setGraphqlSchema: (schema) => set({ graphqlSchema: schema }),
  setGraphqlHeaders: (rows) => set({ graphqlHeaders: rows }),
  addGraphqlSubscriptionMessage: (msg) =>
    set((s) => ({
      graphqlSubscriptionMessages: [...s.graphqlSubscriptionMessages, msg],
    })),
  clearGraphqlMessages: (tabId?: string) => {
    const targetId = tabId || get().activeTabId;
    set((s) => {
      const updatedTabs = s.tabs.map((t) =>
        t.id === targetId
          ? { ...t, graphqlSubscriptionMessages: [], graphqlResponse: null }
          : t,
      );
      return {
        graphqlSubscriptionMessages: [],
        graphqlResponse: null,
        tabs: updatedTabs,
        activeTab: updatedTabs.find((t) => t.id === s.activeTabId) || null,
      };
    });
  },

  loadGraphqlSchema: async (targetUrl?: string) => {
    const { activeTab } = get();
    const url = targetUrl || activeTab?.request.url;
    if (!url?.trim()) return;

    // collect headers from the graphqlHeaders rows
    const headers = get().graphqlHeaders.filter(
      (h) => h.enabled && h.key.trim()
    );

    const auth = (activeTab?.request as any)?.auth;

    set({ graphqlSchemaLoading: true, graphqlSchema: null });
    try {
      const schema = await invoke<GraphQlSchema>("graphql_introspect", {
        url,
        headers,
        auth,
      });
      set({ graphqlSchema: schema });
    } catch (e: any) {
      console.error("[GraphQL] introspect error:", e);
    } finally {
      set({ graphqlSchemaLoading: false });
    }
  },

  invokeGraphql: async (targetTabId?: string) => {
    const { activeTabId, tabs } = get();
    const id = targetTabId || activeTabId;
    if (!id) return;
    const tab = tabs.find((t) => t.id === id);
    if (!tab || tab.request.type !== "graphql") return;
    const req = { ...(tab.request as any) };

    // If no operationName is set but document has named operations, pick the first one
    if (!req.operationName && !req.operation_name && req.query) {
      const match = /(?:^|\s)(?:query|mutation|subscription)\s+([A-Za-z0-9_]+)/.exec(req.query);
      if (match && match[1]) {
        req.operationName = match[1];
        req.operation_name = match[1];
      }
    }

    set((s) => {
      const nextTabs = s.tabs.map((t) =>
        t.id === id
          ? { ...t, graphqlCallStatus: "sending" as const, graphqlResponse: null }
          : t
      );
      return {
        tabs: nextTabs,
        activeTab: nextTabs.find((t) => t.id === s.activeTabId) || null,
        ...(id === s.activeTabId
          ? {
            graphqlCallStatus: "sending",
            graphqlResponse: null,
          }
          : {}),
      };
    });

    try {
      const response = await invoke<GraphQlResponse>("graphql_execute", {
        request: req,
      });
      set((s) => {
        const nextTabs = s.tabs.map((t) =>
          t.id === id
            ? {
              ...t,
              graphqlCallStatus: (response.errors ? "error" : "ok") as GraphQlCallStatus,
              graphqlResponse: response,
            }
            : t
        );
        return {
          tabs: nextTabs,
          activeTab: nextTabs.find((t) => t.id === s.activeTabId) || null,
          ...(id === s.activeTabId
            ? {
              graphqlCallStatus: (response.errors ? "error" : "ok") as GraphQlCallStatus,
              graphqlResponse: response,
            }
            : {}),
        };
      });
    } catch (e: any) {
      console.error("[GraphQL] execute error:", e);
      const errResp: GraphQlResponse = {
        status: 0,
        statusText: "Request Failed",
        timeMs: 0,
        sizeBytes: 0,
        headers: {},
        errors: JSON.stringify([{ message: typeof e === "string" ? e : String(e) }], null, 2),
      };
      set((s) => {
        const nextTabs = s.tabs.map((t) =>
          t.id === id
            ? { ...t, graphqlCallStatus: "error" as const, graphqlResponse: errResp }
            : t
        );
        return {
          tabs: nextTabs,
          activeTab: nextTabs.find((t) => t.id === s.activeTabId) || null,
          ...(id === s.activeTabId
            ? {
              graphqlCallStatus: "error",
              graphqlResponse: errResp,
            }
            : {}),
        };
      });
    }
  },

  subscribeGraphql: async (targetTabId?: string) => {
    const { activeTabId, tabs } = get();
    const id = targetTabId || activeTabId;
    if (!id) return;
    const tab = tabs.find((t) => t.id === id);
    if (!tab || tab.request.type !== "graphql") return;
    const req = { ...(tab.request as any) };

    // Resolve operation name if needed
    if (!req.operationName && !req.operation_name && req.query) {
      const match = /(?:^|\s)(?:query|mutation|subscription)\s+([A-Za-z0-9_]+)/.exec(req.query);
      if (match && match[1]) {
        req.operationName = match[1];
        req.operation_name = match[1];
      }
    }

    const reqId = tab.request.id;

    // Immediately set streaming state and connectionId on THIS tab so no early frames are dropped
    set((s) => {
      const nextTabs = s.tabs.map((t) =>
        t.id === id
          ? {
            ...t,
            graphqlCallStatus: "streaming" as const,
            graphqlSubscriptionMessages: [],
            graphqlConnectionId: reqId,
          }
          : t
      );
      return {
        tabs: nextTabs,
        activeTab: nextTabs.find((t) => t.id === s.activeTabId) || null,
        ...(id === s.activeTabId
          ? {
            graphqlCallStatus: "streaming",
            graphqlSubscriptionMessages: [],
            graphqlConnectionId: reqId,
          }
          : {}),
      };
    });

    try {
      const connectionId = await invoke<string>("graphql_subscribe", {
        request: req,
      });

      // Keep connectionId in sync if Rust returned a specific ID
      set((s) => {
        const nextTabs = s.tabs.map((t) =>
          t.id === id
            ? { ...t, graphqlConnectionId: connectionId || reqId }
            : t
        );
        return {
          tabs: nextTabs,
          activeTab: nextTabs.find((t) => t.id === s.activeTabId) || null,
          ...(id === s.activeTabId
            ? {
              graphqlConnectionId: connectionId || reqId,
            }
            : {}),
        };
      });
    } catch (e: any) {
      console.error("[GraphQL] subscribe error:", e);
      set((s) => {
        const nextTabs = s.tabs.map((t) =>
          t.id === id
            ? { ...t, graphqlCallStatus: "error" as const, graphqlConnectionId: null }
            : t
        );
        return {
          tabs: nextTabs,
          activeTab: nextTabs.find((t) => t.id === s.activeTabId) || null,
          ...(id === s.activeTabId
            ? {
              graphqlCallStatus: "error",
              graphqlConnectionId: null,
            }
            : {}),
        };
      });
    }
  },

  cancelGraphqlSubscription: async (targetTabId?: string) => {
    const { activeTabId, tabs } = get();
    const id = targetTabId || activeTabId;
    if (!id) return;
    const tab = tabs.find((t) => t.id === id);
    const cid = tab?.graphqlConnectionId || tab?.request.id;
    if (cid) {
      try {
        await invoke("graphql_unsubscribe", { connectionId: cid });
      } catch (e) {
        console.error("[GraphQL] cancel error:", e);
      }
    }
    set((s) => {
      const nextTabs = s.tabs.map((t) =>
        t.id === id
          ? { ...t, graphqlCallStatus: "cancelled" as const, graphqlConnectionId: null }
          : t
      );
      return {
        tabs: nextTabs,
        activeTab: nextTabs.find((t) => t.id === s.activeTabId) || null,
        ...(id === s.activeTabId
          ? {
            graphqlCallStatus: "cancelled",
            graphqlConnectionId: null,
          }
          : {}),
      };
    });
    setTimeout(() => {
      set((s) => {
        const nextTabs = s.tabs.map((t) =>
          t.id === id ? { ...t, graphqlCallStatus: "idle" as const } : t
        );
        return {
          tabs: nextTabs,
          activeTab: nextTabs.find((t) => t.id === s.activeTabId) || null,
          ...(id === s.activeTabId
            ? {
              graphqlCallStatus: "idle",
            }
            : {}),
        };
      });
    }, 800);
  },

  initGraphqlListener: async () => {
    const unlisten = await listen<{
      connectionId: string;
      eventType: string;
      payload: string;
      timestamp: string;
    }>("graphql://event", (event) => {
      const { connectionId, eventType, payload, timestamp } = event.payload;
      const newMsg: GraphQlSubscriptionMessage = {
        id: crypto.randomUUID(),
        connectionId,
        eventType: eventType as any,
        payload,
        timestamp,
      };

      // Route event to the matching tab and keep activeTab synchronized
      set((s) => {
        const nextTabs = s.tabs.map((t) => {
          if (
            t.graphqlConnectionId === connectionId ||
            t.request.id === connectionId
          ) {
            const isComplete = eventType === "complete";
            const isError = eventType === "error";
            return {
              ...t,
              graphqlSubscriptionMessages: [
                ...(t.graphqlSubscriptionMessages || []),
                newMsg,
              ],
              graphqlCallStatus: isComplete
                ? ("ok" as const)
                : isError
                  ? ("error" as const)
                  : ("streaming" as const),
              graphqlConnectionId: isComplete ? null : (t.graphqlConnectionId || connectionId),
            };
          }
          return t;
        });

        return {
          tabs: nextTabs,
          activeTab: nextTabs.find((t) => t.id === s.activeTabId) || null,
        };
      });
    });
    return unlisten;
  },
}));
