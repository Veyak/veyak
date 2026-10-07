import {
  ApiRequest,
  ApiResponse,
  Collection,
  CollectionTree,
  EnvironmentVariable,
  EnvironmentWithVariables,
  RequestItem,
  UploadedFile,
  WsSavedMessage,
} from "@veyak-internal/models";

export type { Collection };

export type AuthType = "none" | "basic" | "bearer" | "apiKey";

export type BodyMode =
  "json" | "form-data" | "urlencoded" | "raw" | "multipart";

export interface ApiRequestBody {
  files?: UploadedFile[];
  raw?: string;
  json?: any;
  mode?: BodyMode;
  // Extend this if you have text fields or JSON payloads
}

export interface CollectionFolder {
  id: string;
  name: string;
  requests: ApiRequest[];
}

// ── WebSocket types ──────────────────────────────────────────────────

/** Runtime log entry — ephemeral, lives only in memory. */
export interface WsMessage {
  connectionId: string;
  direction: "sent" | "received" | "closed";
  data: string;
  timestamp: string;
}

export type WsStatus = "disconnected" | "connecting" | "connected";

export interface RequestTab {
  id: string;
  request: RequestItem;
  isDirty: boolean;
  response?: ApiResponse;
  isSending: boolean;
  error?: string;
  // WebSocket state
  wsConnectionId?: string;
  wsMessages: WsMessage[];
  wsStatus: WsStatus;
  wsSavedMessages: WsSavedMessage[];
  wsProtocol: "raw" | "graphql-ws";
  wsGqlSubscriptionIds: string[];

  // ── Add GraphQL Per-Tab State ──────────────────────────────
  graphqlCallStatus?: GraphQlCallStatus;
  graphqlResponse?: GraphQlResponse | null;
  graphqlSubscriptionMessages?: GraphQlSubscriptionMessage[];
  graphqlConnectionId?: string | null;
}

export interface Workspace {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface Folder {
  id: string;
  collectionId: string;
  parentFolderId: string | null;
  name: string;
  sort_order: number;
}

export interface FolderNode {
  folder: Folder;
  children: FolderNode[];
  requests: ApiRequest[];
}

export interface WorkspaceStore {
  environments: EnvironmentWithVariables[];
  workspaces: Workspace[];
  collections: Collection[];
  activeWorkspaceId: string | null;
  activeCollectionId: string | null;
  activeCollectionTree: CollectionTree | null;
  collectionTrees: CollectionTree[];
  activeEnvironmentId: string | null;
  isLoading: boolean;
  isLoadingCollections: boolean;
  isLoadingCollectionTree: boolean;
  error: string | null;

  fetchWorkspaces: () => Promise<void>;
  createWorkspace: (name: string) => Promise<void>;
  renameWorkspace: (id: string, name: string) => Promise<void>;
  deleteWorkspace: (id: string) => Promise<void>;
  setActiveWorkspace: (id: string) => Promise<void>;
  getActiveState: () => Promise<void>;

  // Collections
  fetchCollections: () => Promise<void>;
  fetchCollectionTree: (collectionId: string) => Promise<void>;
  setActiveCollection: (id: string | null) => Promise<void>;
  createCollection: (name: string) => Promise<void>;
  renameCollection: (id: string, name: string) => Promise<void>;
  deleteCollection: (id: string) => Promise<void>;
  cloneCollection: (id: string, newName: string) => Promise<void>;

  // Folders
  createFolder: (
    collectionId: string,
    parentFolderId: string | null,
    name: string,
  ) => Promise<void>;
  renameFolder: (
    collectionId: string,
    folderId: string,
    name: string,
  ) => Promise<void>;
  deleteFolder: (folderId: string) => Promise<void>;

  // Request
  createRequest: (
    collectionId: string,
    folderId: string | null,
    name: string,
    type: "WS" | "REST" | "GRPC",
  ) => Promise<void>;
  createWs: (
    collectionId: string,
    folderId: string | null,
    name: string,
  ) => Promise<void>;
  deleteRequest: (requestId: string) => Promise<void>;
  renameRequest: (id: string, name: string) => Promise<void>;

  fetchEnvironments: (collectionId?: string) => Promise<void>;
  createEnvironment: (collectionId: string, name: string) => Promise<void>;
  renameEnvironment: (environmentid: string, name: string) => Promise<void>;
  deleteEnvironment: (environmentid: string) => Promise<void>;
  saveVariables: (
    environmentid: string,
    variables: EnvironmentVariable[],
  ) => Promise<void>;
  setActiveEnvironment: (id: string | null) => Promise<void>;
}

export const MethodStyles: Record<string, string> = {
  GET: "text-method-get",
  POST: "text-secondary",
  PUT: "text-warning",
  PATCH: "text-primary",
  DELETE: "text-error",
  OPTIONS: "text-text-muted",
  HEAD: "text-text-muted",
  WS: "text-method-ws",
  QUERY: "text-method-query",
  GRPC: "text-method-grpc",
  GRAPHQL: "text-method-graphql",
};

// ── gRPC types ───────────────────────────────────────────────────────

export type GrpcStreamType =
  "unary" | "server_stream" | "client_stream" | "bidi_stream";

export type GrpcCallStatus =
  "idle" | "invoking" | "streaming" | "ok" | "error" | "cancelled";

export interface GrpcMessage {
  id: string;
  connectionId?: string;
  direction: "sent" | "received" | "closed";
  data: string; // JSON-stringified proto message
  timestamp: string;
  statusCode?: string; // gRPC status code label e.g. "OK", "UNAVAILABLE"
  isError?: boolean;
  latencyMs?: number;
}

export interface GrpcMethod {
  name: string;
  fullName: string; // package.ServiceName/MethodName
  requestType: string;
  responseType: string;
  streamType: GrpcStreamType;
}

export interface GrpcService {
  name: string;
  fullName: string;
  methods: GrpcMethod[];
}

export interface GrpcMetadataRow {
  id: string;
  key: string;
  value: string;
  enabled: boolean;
}

// ── GraphQL types ─────────────────────────────────────────────────────

export type GraphQlOperationType = "query" | "mutation" | "subscription";

export type GraphQlCallStatus =
  "idle" | "sending" | "streaming" | "ok" | "error" | "cancelled";

export interface GraphQlHeaderRow {
  id: string;
  key: string;
  value: string;
  enabled: boolean;
}

export interface GraphQlSubscriptionMessage {
  id: string;
  connectionId: string;
  eventType: "data" | "error" | "complete" | "connecting";
  payload: string;
  timestamp: string;
}

export interface GraphQlTypeRef {
  kind: string;
  name?: string;
  ofType?: GraphQlTypeRef;
}

export interface GraphQlArg {
  name: string;
  description?: string;
  typeRef: GraphQlTypeRef;
  defaultValue?: string;
}

export interface GraphQlSchemaField {
  name: string;
  description?: string;
  typeRef: GraphQlTypeRef;
  args: GraphQlArg[];
  isDeprecated: boolean;
  deprecationReason?: string;
}

export interface GraphQlEnumValue {
  name: string;
  description?: string;
  isDeprecated: boolean;
  deprecationReason?: string;
}

export interface GraphQlSchemaType {
  name: string;
  kind: string;
  description?: string;
  fields: GraphQlSchemaField[];
  inputFields: GraphQlArg[];
  enumValues: GraphQlEnumValue[];
}

export interface GraphQlSchema {
  queryType?: string;
  mutationType?: string;
  subscriptionType?: string;
  types: GraphQlSchemaType[];
}

export interface GraphQlResponse {
  status: number;
  statusText: string;
  timeMs: number;
  sizeBytes: number;
  headers: Record<string, string>;
  data?: string;
  errors?: string;
  extensions?: string;
}

export const UI_CSS_VAR_MAP: Record<string, string> = {
  // camelCase
  colorBg: "--color-bg",
  colorPanel: "--color-panel",
  colorPanelRaised: "--color-panel-raised",
  colorBorder: "--color-border",
  colorBorderMuted: "--color-borderMuted",
  colorTextPrimary: "--color-text-primary",
  colorTextSecondary: "--color-text-secondary",
  colorTextMuted: "--color-text-muted",
  colorPrimary: "--color-primary",
  colorPrimaryHover: "--color-primary-hover",
  colorSecondary: "--color-secondary",
  colorSuccess: "--color-success",
  colorError: "--color-error",
  colorWarning: "--color-warning",
  methodGet: "--color-method-get",
  methodPost: "--color-method-post",
  methodPut: "--color-method-put",
  methodDelete: "--color-method-delete",
  methodPatch: "--color-method-patch",
  methodQuery: "--color-method-query",
  methodWs: "--color-method-ws",
  methodGrpc: "--color-method-grpc",
  methodGraphql: "--color-method-graphql",
  radiusMd: "--radius-md",
  radiusLg: "--radius-lg",
  fontSans: "--font-sans",
  fontMono: "--font-mono",

  // snake_case fallbacks
  color_bg: "--color-bg",
  color_panel: "--color-panel",
  color_panel_raised: "--color-panel-raised",
  color_border: "--color-border",
  color_border_muted: "--color-borderMuted",
  color_text_primary: "--color-text-primary",
  color_text_secondary: "--color-text-secondary",
  color_text_muted: "--color-text-muted",
  color_primary: "--color-primary",
  color_primary_hover: "--color-primary-hover",
  color_secondary: "--color-secondary",
  color_success: "--color-success",
  color_error: "--color-error",
  color_warning: "--color-warning",
  method_get: "--color-method-get",
  method_post: "--color-method-post",
  method_put: "--color-method-put",
  method_delete: "--color-method-delete",
  method_patch: "--color-method-patch",
  method_query: "--color-method-query",
  method_ws: "--color-method-ws",
  method_grpc: "--color-method-grpc",
  method_graphql: "--color-method-graphql",
  radius_md: "--radius-md",
  radius_lg: "--radius-lg",
  // font_sans: '--font-sans',
  // font_mono: '--font-mono',
};

export const SYNTAX_CSS_VAR_MAP: Record<string, string> = {
  keyword: "--syntax-keyword",
  string: "--syntax-string",
  comment: "--syntax-comment",
  property: "--syntax-property",
  punctuation: "--syntax-punctuation",
  operator: "--syntax-operator",
  number: "--syntax-number",
  boolean: "--syntax-boolean",
  null: "--syntax-null",
  function: "--syntax-function",
  variable: "--syntax-variable",
  attribute: "--syntax-attribute",
  className: "--syntax-class-name",
  class_name: "--syntax-class-name",
};
