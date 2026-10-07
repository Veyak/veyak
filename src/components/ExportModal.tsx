import React, { useState, useEffect } from "react";
import {
  X,
  Download,
  Copy,
  Check,
  Layers,
  Folder,
  Cloud,
  Loader2,
  AlertCircle,
  Eye,
  EyeOff,
} from "lucide-react";
import { useWorkspaceStore } from "../store/workspaceStore";
import { ExportFormat } from "@veyak-internal/models";

interface ExportModalProps {
  isMobile?: boolean;
}

const EXPORT_FORMATS: {
  id: ExportFormat;
  label: string;
  badge: string;
  color: string;
  borderColor: string;
  bgColor: string;
  desc: string;
  extension: string;
}[] = [
  {
    id: "postman",
    label: "Postman Collection",
    badge: "v2.1.0",
    color: "text-amber-400",
    borderColor: "border-amber-500/40",
    bgColor: "bg-amber-500/10",
    desc: "Postman Collection v2.1 format",
    extension: "postman_collection.json",
  },
  {
    id: "insomnia",
    label: "Insomnia Export",
    badge: "v4 JSON",
    color: "text-purple-400",
    borderColor: "border-purple-500/40",
    bgColor: "bg-purple-500/10",
    desc: "Insomnia Export v4 format",
    extension: "insomnia.json",
  },
  {
    id: "yaak",
    label: "Yaak Export",
    badge: "Schema 3",
    color: "text-emerald-400",
    borderColor: "border-emerald-500/40",
    bgColor: "bg-emerald-500/10",
    desc: "Yaak Workspace Export format",
    extension: "yaak.json",
  },
  {
    id: "veyak",
    label: "Veyak Portable",
    badge: "Native",
    color: "text-blue-400",
    borderColor: "border-blue-500/40",
    bgColor: "bg-blue-500/10",
    desc: "Full native backup bundle",
    extension: "veyak.json",
  },
];

export const ExportModal: React.FC<ExportModalProps> = ({
  isMobile = false,
}) => {
  const isExportModalOpen = useWorkspaceStore((s) => s.isExportModalOpen);
  const closeExportModal = useWorkspaceStore((s) => s.closeExportModal);
  const exportModalTarget = useWorkspaceStore((s) => s.exportModalTarget);
  const collections = useWorkspaceStore((s) => s.collections);
  const environments = useWorkspaceStore((s) => s.environments);
  const activeCollectionId = useWorkspaceStore((s) => s.activeCollectionId);

  const exportCollectionContent = useWorkspaceStore(
    (s) => s.exportCollectionContent,
  );
  const exportWorkspaceContent = useWorkspaceStore(
    (s) => s.exportWorkspaceContent,
  );
  const exportEnvironmentContent = useWorkspaceStore(
    (s) => s.exportEnvironmentContent,
  );
  const fetchEnvironments = useWorkspaceStore((s) => s.fetchEnvironments);

  // Scope selection
  const [scope, setScope] = useState<
    "collection" | "workspace" | "environment"
  >("collection");
  const [selectedCollectionId, setSelectedCollectionId] = useState<string>("");
  const [selectedEnvironmentId, setSelectedEnvironmentId] =
    useState<string>("");

  // Format selection
  const [format, setFormat] = useState<ExportFormat>("postman");

  // Output state
  const [exportedContent, setExportedContent] = useState<string>("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [showPreview, setShowPreview] = useState(true);

  // Synchronize initial target when opening
  useEffect(() => {
    if (isExportModalOpen) {
      setError(null);
      setCopied(false);
      if (exportModalTarget) {
        setScope(exportModalTarget.type);
        if (exportModalTarget.type === "collection" && exportModalTarget.id) {
          setSelectedCollectionId(exportModalTarget.id);
        } else if (
          exportModalTarget.type === "environment" &&
          exportModalTarget.id
        ) {
          setSelectedEnvironmentId(exportModalTarget.id);
        }
      } else {
        setScope("collection");
        setSelectedCollectionId(activeCollectionId || collections[0]?.id || "");
        setSelectedEnvironmentId(environments[0]?.environment.id || "");
      }
    }
  }, [
    isExportModalOpen,
    exportModalTarget,
    activeCollectionId,
    collections,
    environments,
  ]);

  // Handle escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isExportModalOpen) {
        closeExportModal();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isExportModalOpen, closeExportModal]);

  // Generate preview whenever scope, format, or selected entity changes
  useEffect(() => {
    if (!isExportModalOpen) return;

    let isCancelled = false;
    const generateExport = async () => {
      setIsLoading(true);
      setError(null);
      try {
        let content = "";
        if (scope === "collection") {
          if (!selectedCollectionId) {
            setExportedContent("");
            setIsLoading(false);
            return;
          }
          content = await exportCollectionContent(selectedCollectionId, format);
        } else if (scope === "workspace") {
          content = await exportWorkspaceContent(format);
        } else if (scope === "environment") {
          if (!selectedEnvironmentId) {
            setExportedContent("");
            setIsLoading(false);
            return;
          }
          content = await exportEnvironmentContent(
            selectedEnvironmentId,
            format,
          );
        }

        if (!isCancelled) {
          setExportedContent(content);
        }
      } catch (err: any) {
        if (!isCancelled) {
          console.error("Export generation error:", err);
          setError(
            typeof err === "string"
              ? err
              : err?.message || "Failed to generate export.",
          );
        }
      } finally {
        if (!isCancelled) {
          setIsLoading(false);
        }
      }
    };

    generateExport();

    return () => {
      isCancelled = true;
    };
  }, [
    isExportModalOpen,
    scope,
    format,
    selectedCollectionId,
    selectedEnvironmentId,
    exportCollectionContent,
    exportWorkspaceContent,
    exportEnvironmentContent,
  ]);

  if (!isExportModalOpen) return null;

  const currentFormatMeta =
    EXPORT_FORMATS.find((f) => f.id === format) || EXPORT_FORMATS[0];

  const handleCopy = async () => {
    if (!exportedContent) return;
    try {
      await navigator.clipboard.writeText(exportedContent);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  };

  const handleDownload = () => {
    if (!exportedContent) return;
    try {
      let baseName = "export";
      if (scope === "collection") {
        const col = collections.find((c) => c.id === selectedCollectionId);
        baseName = col
          ? col.name.replace(/\s+/g, "_").toLowerCase()
          : "collection";
      } else if (scope === "workspace") {
        baseName = "workspace";
      } else if (scope === "environment") {
        const env = environments.find(
          (e) => e.environment.id === selectedEnvironmentId,
        );
        baseName = env
          ? env.environment.name.replace(/\s+/g, "_").toLowerCase()
          : "environment";
      }

      const fileName = `${baseName}.${currentFormatMeta.extension}`;
      const blob = new Blob([exportedContent], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error("Failed to download file:", err);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/65 backdrop-blur-sm animate-fade-in"
      onClick={closeExportModal}
    >
      <div
        className={`relative w-full ${
          isMobile
            ? "max-w-full h-full max-h-full rounded-none"
            : "max-w-2xl max-h-[90vh] rounded-2xl"
        } bg-panel border border-border shadow-2xl overflow-hidden flex flex-col animate-scale-in`}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border/80 bg-panel-raised/50">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-primary/10 border border-primary/20 text-primary">
              <Download className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-semibold text-text-primary">
                Export API Data
              </h2>
              <p className="text-xs text-text-muted">
                Export to Postman, Insomnia, Yaak, or portable Veyak JSON
              </p>
            </div>
          </div>
          <button
            onClick={closeExportModal}
            className="p-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-panel transition-colors"
            aria-label="Close"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Modal Body */}
        <div className="p-6 overflow-y-auto space-y-5">
          {/* Scope Selection */}
          <div>
            <label className="block text-xs font-medium text-text-secondary mb-2">
              Export Scope
            </label>
            <div className="grid grid-cols-3 gap-2">
              <button
                type="button"
                onClick={() => setScope("collection")}
                className={`p-2.5 rounded-xl border text-left transition-all flex items-center gap-2 ${
                  scope === "collection"
                    ? "border-primary/50 bg-primary/10 text-primary ring-1 ring-primary/40"
                    : "border-border/60 bg-panel-raised/40 hover:bg-panel-raised text-text-primary"
                }`}
              >
                <Folder className="w-4 h-4 shrink-0" />
                <span className="text-xs font-semibold">Collection</span>
              </button>

              <button
                type="button"
                onClick={() => setScope("workspace")}
                className={`p-2.5 rounded-xl border text-left transition-all flex items-center gap-2 ${
                  scope === "workspace"
                    ? "border-primary/50 bg-primary/10 text-primary ring-1 ring-primary/40"
                    : "border-border/60 bg-panel-raised/40 hover:bg-panel-raised text-text-primary"
                }`}
              >
                <Layers className="w-4 h-4 shrink-0" />
                <span className="text-xs font-semibold">Entire Workspace</span>
              </button>

              <button
                type="button"
                onClick={() => setScope("environment")}
                className={`p-2.5 rounded-xl border text-left transition-all flex items-center gap-2 ${
                  scope === "environment"
                    ? "border-primary/50 bg-primary/10 text-primary ring-1 ring-primary/40"
                    : "border-border/60 bg-panel-raised/40 hover:bg-panel-raised text-text-primary"
                }`}
              >
                <Cloud className="w-4 h-4 shrink-0" />
                <span className="text-xs font-semibold">Environment</span>
              </button>
            </div>
          </div>

          {/* Sub-selectors for Collection or Environment */}
          {scope === "collection" && collections.length > 0 && (
            <div>
              <label className="block text-xs font-medium text-text-secondary mb-1.5">
                Select Collection
              </label>
              <select
                value={selectedCollectionId}
                onChange={(e) => setSelectedCollectionId(e.target.value)}
                className="w-full bg-panel-raised border border-border text-text-primary rounded-xl px-3 py-2 text-xs focus:outline-none focus:border-primary/60"
              >
                {collections.map((col) => (
                  <option key={col.id} value={col.id}>
                    {col.name}
                  </option>
                ))}
              </select>
            </div>
          )}

          {scope === "environment" && (
            <div className="space-y-3">
              {collections.length > 1 && (
                <div>
                  <label className="block text-xs font-medium text-text-secondary mb-1.5">
                    Select Collection
                  </label>
                  <select
                    value={selectedCollectionId}
                    onChange={(e) => {
                      const colId = e.target.value;
                      setSelectedCollectionId(colId);
                      fetchEnvironments(colId);
                    }}
                    className="w-full bg-panel-raised border border-border text-text-primary rounded-xl px-3 py-2 text-xs focus:outline-none focus:border-primary/60"
                  >
                    {collections.map((col) => (
                      <option key={col.id} value={col.id}>
                        {col.name}
                      </option>
                    ))}
                  </select>
                </div>
              )}
              {environments.length > 0 ? (
                <div>
                  <label className="block text-xs font-medium text-text-secondary mb-1.5">
                    Select Environment
                  </label>
                  <select
                    value={selectedEnvironmentId}
                    onChange={(e) => setSelectedEnvironmentId(e.target.value)}
                    className="w-full bg-panel-raised border border-border text-text-primary rounded-xl px-3 py-2 text-xs focus:outline-none focus:border-primary/60"
                  >
                    {environments.map((env) => (
                      <option
                        key={env.environment.id}
                        value={env.environment.id}
                      >
                        {env.environment.name}
                      </option>
                    ))}
                  </select>
                </div>
              ) : (
                <div className="text-xs text-text-muted py-1">
                  No environments found in this collection.
                </div>
              )}
            </div>
          )}

          {/* Format Selection Pills */}
          <div>
            <label className="block text-xs font-medium text-text-secondary mb-2">
              Export Format
            </label>
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
              {EXPORT_FORMATS.map((opt) => {
                const isSelected = format === opt.id;
                return (
                  <button
                    key={opt.id}
                    type="button"
                    onClick={() => setFormat(opt.id)}
                    className={`p-3 rounded-xl border text-left transition-all flex flex-col justify-between ${
                      isSelected
                        ? `${opt.borderColor} ${opt.bgColor} ring-1 ring-primary/40`
                        : "border-border/60 bg-panel-raised/40 hover:bg-panel-raised hover:border-border"
                    }`}
                  >
                    <div className="text-xs font-semibold text-text-primary">
                      {opt.label}
                    </div>
                    <span className="text-[10px] text-text-muted mt-1">
                      {opt.badge}
                    </span>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Error Message */}
          {error && (
            <div className="p-3 rounded-xl bg-error/10 border border-error/30 text-xs text-error flex items-start gap-2">
              <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
              <div>{error}</div>
            </div>
          )}

          {/* Preview Section */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <span className="text-xs font-medium text-text-secondary">
                  Export Preview
                </span>
                {isLoading && (
                  <Loader2 className="w-3 h-3 animate-spin text-primary" />
                )}
              </div>
              <button
                type="button"
                onClick={() => setShowPreview(!showPreview)}
                className="text-[11px] text-text-muted hover:text-text-primary flex items-center gap-1"
              >
                {showPreview ? (
                  <>
                    <EyeOff className="w-3 h-3" /> Hide Preview
                  </>
                ) : (
                  <>
                    <Eye className="w-3 h-3" /> Show Preview
                  </>
                )}
              </button>
            </div>

            {showPreview && (
              <div className="relative">
                <pre className="w-full max-h-56 p-3 rounded-xl bg-panel-raised border border-border text-text-primary font-mono text-[11px] overflow-auto select-text leading-relaxed">
                  {isLoading
                    ? "Generating export payload..."
                    : exportedContent || "// No content generated"}
                </pre>
              </div>
            )}
          </div>
        </div>

        {/* Modal Footer */}
        <div className="flex items-center justify-between px-6 py-4 border-t border-border/80 bg-panel-raised/30">
          <div className="text-[11px] text-text-muted">
            {exportedContent
              ? `${(exportedContent.length / 1024).toFixed(1)} KB JSON`
              : ""}
          </div>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={handleCopy}
              disabled={isLoading || !exportedContent}
              className="px-3.5 py-2 rounded-xl border border-border bg-panel-raised hover:bg-panel text-xs font-medium text-text-primary transition-colors flex items-center gap-1.5 shadow-xs disabled:opacity-50"
            >
              {copied ? (
                <>
                  <Check className="w-3.5 h-3.5 text-emerald-400" />
                  Copied!
                </>
              ) : (
                <>
                  <Copy className="w-3.5 h-3.5" />
                  Copy JSON
                </>
              )}
            </button>
            <button
              type="button"
              onClick={handleDownload}
              disabled={isLoading || !exportedContent}
              className="px-4 py-2 rounded-xl bg-primary text-white text-xs font-semibold hover:bg-primary-hover transition-colors shadow-sm disabled:opacity-50 flex items-center gap-1.5"
            >
              <Download className="w-3.5 h-3.5" />
              Download File
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
