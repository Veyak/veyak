import React, { useState, useEffect, useRef } from "react";
import {
  X,
  Upload,
  FileCode,
  Sparkles,
  CheckCircle2,
  AlertCircle,
  Loader2,
  FolderInput,
  Folder,
  Layers,
  ArrowRight,
} from "lucide-react";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { useWorkspaceStore } from "../store/workspaceStore";
import { ImportFormat, ImportSummary } from "@veyak-internal/models";

interface ImportModalProps {
  isMobile?: boolean;
}

const FORMAT_OPTIONS: {
  id: ImportFormat;
  label: string;
  badge: string;
  color: string;
  borderColor: string;
  bgColor: string;
  desc: string;
}[] = [
  {
    id: "auto",
    label: "Auto Detect",
    badge: "Smart",
    color: "text-primary",
    borderColor: "border-primary/40",
    bgColor: "bg-primary/10",
    desc: "Automatically detects format",
  },
  {
    id: "postman",
    label: "Postman",
    badge: "v2.0 / v2.1",
    color: "text-amber-400",
    borderColor: "border-amber-500/40",
    bgColor: "bg-amber-500/10",
    desc: "Postman Collections & Envs",
  },
  {
    id: "insomnia",
    label: "Insomnia",
    badge: "v4 / v5 JSON/YAML",
    color: "text-purple-400",
    borderColor: "border-purple-500/40",
    bgColor: "bg-purple-500/10",
    desc: "Insomnia Export v4 & v5",
  },
  {
    id: "yaak",
    label: "Yaak",
    badge: "Schema 3",
    color: "text-emerald-400",
    borderColor: "border-emerald-500/40",
    bgColor: "bg-emerald-500/10",
    desc: "Yaak Workspace Export",
  },
  {
    id: "veyak",
    label: "Veyak",
    badge: "Native",
    color: "text-blue-400",
    borderColor: "border-blue-500/40",
    bgColor: "bg-blue-500/10",
    desc: "Native Veyak Backup",
  },
];

export const ImportModal: React.FC<ImportModalProps> = ({ isMobile = false }) => {
  const isImportModalOpen = useWorkspaceStore((s) => s.isImportModalOpen);
  const closeImportModal = useWorkspaceStore((s) => s.closeImportModal);
  const importTargetCollectionId = useWorkspaceStore(
    (s) => s.importTargetCollectionId,
  );
  const collections = useWorkspaceStore((s) => s.collections);
  const importDataContent = useWorkspaceStore((s) => s.importDataContent);
  const importFileContent = useWorkspaceStore((s) => s.importFileContent);
  const detectFormat = useWorkspaceStore((s) => s.detectFormat);

  const [inputMode, setInputMode] = useState<"file" | "paste">("file");
  const [selectedFormat, setSelectedFormat] = useState<ImportFormat>("auto");
  const [detectedFormat, setDetectedFormat] = useState<ImportFormat | null>(null);

  // File upload state
  const [selectedFilePath, setSelectedFilePath] = useState<string>("");
  const [selectedFileName, setSelectedFileName] = useState<string>("");
  const [fileContent, setFileContent] = useState<string>("");
  const [isDragging, setIsDragging] = useState(false);

  // Raw paste state
  const [rawContent, setRawContent] = useState<string>("");

  // Target collection state
  const [targetMode, setTargetMode] = useState<"new" | "existing">(
    importTargetCollectionId ? "existing" : "new",
  );
  const [selectedCollectionId, setSelectedCollectionId] = useState<string>(
    importTargetCollectionId || "",
  );

  // Status state
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [summary, setSummary] = useState<ImportSummary | null>(null);

  const fileInputRef = useRef<HTMLInputElement>(null);

  // Reset state when opening
  useEffect(() => {
    if (isImportModalOpen) {
      setError(null);
      setSummary(null);
      setSelectedFilePath("");
      setSelectedFileName("");
      setFileContent("");
      setRawContent("");
      setSelectedFormat("auto");
      setDetectedFormat(null);
      if (importTargetCollectionId) {
        setTargetMode("existing");
        setSelectedCollectionId(importTargetCollectionId);
      } else {
        setTargetMode("new");
        setSelectedCollectionId(collections[0]?.id || "");
      }
    }
  }, [isImportModalOpen, importTargetCollectionId, collections]);

  // Handle escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isImportModalOpen) {
        closeImportModal();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isImportModalOpen, closeImportModal]);

  // Live auto-detect format when rawContent or fileContent changes
  useEffect(() => {
    const content = inputMode === "file" ? fileContent : rawContent;
    if (content.trim().length > 10) {
      detectFormat(content)
        .then((fmt) => setDetectedFormat(fmt))
        .catch(() => setDetectedFormat(null));
    } else {
      setDetectedFormat(null);
    }
  }, [rawContent, fileContent, inputMode, detectFormat]);

  if (!isImportModalOpen) return null;

  // File selection via native dialog
  const handlePickFileNative = async () => {
    try {
      const selected = await openFileDialog({
        multiple: false,
        filters: [
          {
            name: "API Collections & Environments",
            extensions: ["json", "yaml", "yml"],
          },
        ],
      });

      if (selected && typeof selected === "string") {
        setSelectedFilePath(selected);
        const name = selected.split(/[/\\]/).pop() || selected;
        setSelectedFileName(name);
        setError(null);
      }
    } catch (err) {
      console.warn("Native file dialog failed, falling back to HTML input:", err);
      fileInputRef.current?.click();
    }
  };

  // Fallback HTML file picker
  const handleHtmlFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setSelectedFileName(file.name);
      setSelectedFilePath("");
      const reader = new FileReader();
      reader.onload = (event) => {
        const text = event.target?.result as string;
        setFileContent(text);
      };
      reader.readAsText(file);
    }
  };

  // Drag and drop handlers
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = () => {
    setIsDragging(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const file = e.dataTransfer.files?.[0];
    if (file) {
      setSelectedFileName(file.name);
      setSelectedFilePath("");
      const reader = new FileReader();
      reader.onload = (event) => {
        const text = event.target?.result as string;
        setFileContent(text);
      };
      reader.readAsText(file);
    }
  };

  const handleExecuteImport = async () => {
    setIsProcessing(true);
    setError(null);

    const targetColId =
      targetMode === "existing" ? selectedCollectionId : undefined;
    const formatToUse =
      selectedFormat === "auto" ? undefined : selectedFormat;

    try {
      let result: ImportSummary;
      if (inputMode === "file" && selectedFilePath) {
        result = await importFileContent(
          selectedFilePath,
          formatToUse,
          targetColId,
        );
      } else {
        const content = inputMode === "file" ? fileContent : rawContent;
        if (!content.trim()) {
          throw new Error("Please select a file or paste content to import.");
        }
        result = await importDataContent(content, formatToUse, targetColId);
      }
      setSummary(result);
    } catch (err: any) {
      console.error("Import failed:", err);
      setError(
        typeof err === "string"
          ? err
          : err?.message || "Failed to parse or import data.",
      );
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/65 backdrop-blur-sm animate-fade-in"
      onClick={closeImportModal}
    >
      <div
        className={`relative w-full ${
          isMobile ? "max-w-full h-full max-h-full rounded-none" : "max-w-2xl max-h-[90vh] rounded-2xl"
        } bg-panel border border-border shadow-2xl overflow-hidden flex flex-col animate-scale-in`}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border/80 bg-panel-raised/50">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-primary/10 border border-primary/20 text-primary">
              <FolderInput className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-semibold text-text-primary flex items-center gap-2">
                Import API Collection
              </h2>
              <p className="text-xs text-text-muted">
                Import requests, folders, and environments from other clients
              </p>
            </div>
          </div>
          <button
            onClick={closeImportModal}
            className="p-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-panel transition-colors"
            aria-label="Close"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Modal Body */}
        <div className="p-6 overflow-y-auto space-y-5">
          {summary ? (
            /* Success Summary View */
            <div className="py-6 flex flex-col items-center text-center space-y-4 animate-scale-in">
              <div className="w-14 h-14 rounded-full bg-emerald-500/15 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
                <CheckCircle2 className="w-8 h-8" />
              </div>

              <div>
                <h3 className="text-lg font-semibold text-text-primary">
                  Import Completed Successfully
                </h3>
                <p className="text-xs text-text-muted mt-1">
                  Your data has been processed and saved into the workspace.
                </p>
              </div>

              {/* Metrics Grid */}
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 w-full max-w-lg mt-2">
                <div className="p-3 bg-panel-raised border border-border rounded-xl">
                  <div className="text-xl font-bold text-primary">
                    {summary.collectionsCount}
                  </div>
                  <div className="text-[11px] text-text-muted uppercase tracking-wider mt-0.5">
                    Collections
                  </div>
                </div>
                <div className="p-3 bg-panel-raised border border-border rounded-xl">
                  <div className="text-xl font-bold text-amber-400">
                    {summary.foldersCount}
                  </div>
                  <div className="text-[11px] text-text-muted uppercase tracking-wider mt-0.5">
                    Folders
                  </div>
                </div>
                <div className="p-3 bg-panel-raised border border-border rounded-xl">
                  <div className="text-xl font-bold text-emerald-400">
                    {summary.requestsCount}
                  </div>
                  <div className="text-[11px] text-text-muted uppercase tracking-wider mt-0.5">
                    Requests
                  </div>
                </div>
                <div className="p-3 bg-panel-raised border border-border rounded-xl">
                  <div className="text-xl font-bold text-purple-400">
                    {summary.environmentsCount}
                  </div>
                  <div className="text-[11px] text-text-muted uppercase tracking-wider mt-0.5">
                    Environments
                  </div>
                </div>
              </div>

              {summary.warnings.length > 0 && (
                <div className="w-full max-w-lg text-left p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 text-xs text-amber-300 space-y-1">
                  <div className="font-semibold flex items-center gap-1.5">
                    <AlertCircle className="w-3.5 h-3.5" /> Warnings:
                  </div>
                  <ul className="list-disc list-inside space-y-0.5 text-text-muted">
                    {summary.warnings.map((w, i) => (
                      <li key={i}>{w}</li>
                    ))}
                  </ul>
                </div>
              )}

              <div className="pt-2">
                <button
                  onClick={closeImportModal}
                  className="px-6 py-2 rounded-xl bg-primary text-white text-xs font-semibold hover:bg-primary-hover transition-colors shadow-sm"
                >
                  Done
                </button>
              </div>
            </div>
          ) : (
            /* Normal Import Form */
            <>
              {/* Format Pills */}
              <div>
                <label className="block text-xs font-medium text-text-secondary mb-2">
                  Source Format
                </label>
                <div className="grid grid-cols-2 sm:grid-cols-5 gap-2">
                  {FORMAT_OPTIONS.map((opt) => {
                    const isSelected = selectedFormat === opt.id;
                    return (
                      <button
                        key={opt.id}
                        type="button"
                        onClick={() => setSelectedFormat(opt.id)}
                        className={`p-2.5 rounded-xl border text-left transition-all flex flex-col justify-between ${
                          isSelected
                            ? `${opt.borderColor} ${opt.bgColor} ring-1 ring-primary/40`
                            : "border-border/60 bg-panel-raised/40 hover:bg-panel-raised hover:border-border"
                        }`}
                      >
                        <div className="flex items-center justify-between w-full">
                          <span
                            className={`text-xs font-semibold ${
                              isSelected ? opt.color : "text-text-primary"
                            }`}
                          >
                            {opt.label}
                          </span>
                        </div>
                        <span className="text-[10px] text-text-muted mt-1">
                          {opt.badge}
                        </span>
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Mode Tabs */}
              <div>
                <div className="flex items-center justify-between mb-2">
                  <label className="text-xs font-medium text-text-secondary">
                    Import Source
                  </label>
                  {detectedFormat && detectedFormat !== "auto" && (
                    <span className="inline-flex items-center gap-1 text-[11px] font-medium text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-md border border-emerald-500/20">
                      <Sparkles className="w-3 h-3" />
                      Detected: {detectedFormat.toUpperCase()}
                    </span>
                  )}
                </div>

                <div className="flex rounded-lg bg-panel-raised p-1 border border-border mb-3">
                  <button
                    type="button"
                    onClick={() => setInputMode("file")}
                    className={`flex-1 py-1.5 text-xs font-medium rounded-md transition-all flex items-center justify-center gap-1.5 ${
                      inputMode === "file"
                        ? "bg-panel text-text-primary shadow-xs"
                        : "text-text-muted hover:text-text-primary"
                    }`}
                  >
                    <Upload className="w-3.5 h-3.5" />
                    File Upload
                  </button>
                  <button
                    type="button"
                    onClick={() => setInputMode("paste")}
                    className={`flex-1 py-1.5 text-xs font-medium rounded-md transition-all flex items-center justify-center gap-1.5 ${
                      inputMode === "paste"
                        ? "bg-panel text-text-primary shadow-xs"
                        : "text-text-muted hover:text-text-primary"
                    }`}
                  >
                    <FileCode className="w-3.5 h-3.5" />
                    Paste Raw Content
                  </button>
                </div>

                {inputMode === "file" ? (
                  /* File Dropzone */
                  <div
                    onDragOver={handleDragOver}
                    onDragLeave={handleDragLeave}
                    onDrop={handleDrop}
                    className={`border-2 border-dashed rounded-xl p-8 flex flex-col items-center justify-center text-center transition-all ${
                      isDragging
                        ? "border-primary bg-primary/10 scale-[1.01]"
                        : selectedFileName
                          ? "border-emerald-500/40 bg-emerald-500/5"
                          : "border-border hover:border-primary/40 bg-panel-raised/30 hover:bg-panel-raised/50"
                    }`}
                  >
                    <input
                      ref={fileInputRef}
                      type="file"
                      accept=".json,.yaml,.yml"
                      className="hidden"
                      onChange={handleHtmlFileChange}
                    />

                    {selectedFileName ? (
                      <div className="space-y-2">
                        <div className="w-12 h-12 rounded-xl bg-emerald-500/15 border border-emerald-500/30 flex items-center justify-center text-emerald-400 mx-auto">
                          <CheckCircle2 className="w-6 h-6" />
                        </div>
                        <div className="text-xs font-semibold text-text-primary">
                          {selectedFileName}
                        </div>
                        <p className="text-[11px] text-text-muted">
                          File ready to import.
                        </p>
                        <button
                          type="button"
                          onClick={handlePickFileNative}
                          className="text-xs text-primary hover:underline font-medium pt-1"
                        >
                          Change file
                        </button>
                      </div>
                    ) : (
                      <div className="space-y-3">
                        <div className="w-12 h-12 rounded-xl bg-panel-raised border border-border flex items-center justify-center text-text-muted mx-auto">
                          <Upload className="w-5 h-5" />
                        </div>
                        <div>
                          <p className="text-xs font-medium text-text-primary">
                            Drag and drop your file here, or browse
                          </p>
                          <p className="text-[11px] text-text-muted mt-0.5">
                            Supports .json, .yaml, and .yml files
                          </p>
                        </div>
                        <button
                          type="button"
                          onClick={handlePickFileNative}
                          className="px-4 py-1.5 rounded-lg bg-panel-raised hover:bg-panel border border-border text-xs font-medium text-text-primary transition-colors shadow-xs"
                        >
                          Browse Files
                        </button>
                      </div>
                    )}
                  </div>
                ) : (
                  /* Raw Paste Textarea */
                  <div className="space-y-2">
                    <textarea
                      rows={9}
                      value={rawContent}
                      onChange={(e) => setRawContent(e.target.value)}
                      placeholder={`Paste Postman collection JSON, Insomnia export JSON/YAML, or Yaak JSON here...`}
                      className="w-full p-3 rounded-xl bg-panel-raised border border-border text-text-primary text-xs font-mono placeholder:text-text-muted focus:outline-none focus:border-primary/60 transition-colors resize-none"
                    />
                    <div className="flex justify-between items-center text-[11px] text-text-muted px-1">
                      <span>{rawContent.length.toLocaleString()} characters</span>
                      {rawContent.length > 0 && (
                        <button
                          type="button"
                          onClick={() => setRawContent("")}
                          className="hover:text-text-primary"
                        >
                          Clear
                        </button>
                      )}
                    </div>
                  </div>
                )}
              </div>

              {/* Target Collection Selection */}
              <div className="pt-2 border-t border-border/80">
                <label className="block text-xs font-medium text-text-secondary mb-2">
                  Destination Collection
                </label>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                  <button
                    type="button"
                    onClick={() => setTargetMode("new")}
                    className={`p-3 rounded-xl border text-left transition-all flex items-start gap-2.5 ${
                      targetMode === "new"
                        ? "border-primary/50 bg-primary/5 ring-1 ring-primary/30"
                        : "border-border/60 bg-panel-raised/40 hover:bg-panel-raised"
                    }`}
                  >
                    <Layers
                      className={`w-4 h-4 mt-0.5 shrink-0 ${
                        targetMode === "new" ? "text-primary" : "text-text-muted"
                      }`}
                    />
                    <div>
                      <div className="text-xs font-semibold text-text-primary">
                        Create New Collection
                      </div>
                      <div className="text-[11px] text-text-muted mt-0.5">
                        Uses collection name defined in the import
                      </div>
                    </div>
                  </button>

                  <button
                    type="button"
                    onClick={() => setTargetMode("existing")}
                    disabled={collections.length === 0}
                    className={`p-3 rounded-xl border text-left transition-all flex items-start gap-2.5 ${
                      targetMode === "existing"
                        ? "border-primary/50 bg-primary/5 ring-1 ring-primary/30"
                        : collections.length === 0
                          ? "opacity-50 cursor-not-allowed border-border/40 bg-panel-raised/20"
                          : "border-border/60 bg-panel-raised/40 hover:bg-panel-raised"
                    }`}
                  >
                    <Folder
                      className={`w-4 h-4 mt-0.5 shrink-0 ${
                        targetMode === "existing"
                          ? "text-primary"
                          : "text-text-muted"
                      }`}
                    />
                    <div className="flex-1 min-w-0">
                      <div className="text-xs font-semibold text-text-primary">
                        Existing Collection
                      </div>
                      {targetMode === "existing" && collections.length > 0 ? (
                        <select
                          value={selectedCollectionId}
                          onChange={(e) => setSelectedCollectionId(e.target.value)}
                          className="mt-1.5 w-full bg-panel border border-border text-text-primary rounded-md px-2 py-1 text-xs focus:outline-none focus:border-primary/60"
                          onClick={(e) => e.stopPropagation()}
                        >
                          {collections.map((col) => (
                            <option key={col.id} value={col.id}>
                              {col.name}
                            </option>
                          ))}
                        </select>
                      ) : (
                        <div className="text-[11px] text-text-muted mt-0.5">
                          Merge items into an existing collection
                        </div>
                      )}
                    </div>
                  </button>
                </div>
              </div>

              {/* Error Message */}
              {error && (
                <div className="p-3 rounded-xl bg-error/10 border border-error/30 text-xs text-error flex items-start gap-2 animate-shake">
                  <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
                  <div className="flex-1">{error}</div>
                </div>
              )}
            </>
          )}
        </div>

        {/* Modal Footer */}
        {!summary && (
          <div className="flex items-center justify-end gap-2.5 px-6 py-4 border-t border-border/80 bg-panel-raised/30">
            <button
              type="button"
              onClick={closeImportModal}
              disabled={isProcessing}
              className="px-4 py-2 rounded-xl text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-panel transition-colors"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={handleExecuteImport}
              disabled={
                isProcessing ||
                (inputMode === "file" && !selectedFileName && !selectedFilePath) ||
                (inputMode === "paste" && !rawContent.trim())
              }
              className="px-5 py-2 rounded-xl bg-primary text-white text-xs font-semibold hover:bg-primary-hover transition-colors shadow-sm disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
            >
              {isProcessing ? (
                <>
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  Importing...
                </>
              ) : (
                <>
                  Import
                  <ArrowRight className="w-3.5 h-3.5" />
                </>
              )}
            </button>
          </div>
        )}
      </div>
    </div>
  );
};
