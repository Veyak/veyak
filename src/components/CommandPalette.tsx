import { useEffect, useMemo, useRef, useState } from "react";
import {
  Search,
  Plus,
  History,
  Settings2,
  Zap,
  FileText,
  ChevronRight,
  Palette,
  FolderInput,
  Download,
} from "lucide-react";
import { useVartaStore } from "../store/vartaStore";
import { useSettingsStore } from "../store/settingStore";
import { useWorkspaceStore } from "../store/workspaceStore";
import { RequestItem } from "@veyak-internal/models";

// ─── Types ───────────────────────────────────────────────────────────────────

interface PaletteItem {
  id: string;
  category: "actions" | "requests";
  label: string;
  hint?: string;
  breadcrumb?: string;
  method?: string;
  icon: React.ReactNode;
  action: () => void;
}

// ─── Method badge colours matching the design token palette ─────────────────

const METHOD_COLORS: Record<string, string> = {
  GET: "text-method-get",
  POST: "text-method-post",
  PUT: "text-method-put",
  PATCH: "text-method-patch",
  DELETE: "text-method-delete",
  OPTIONS: "text-text-muted",
  HEAD: "text-text-muted",
  QUERY: "text-text-muted",
};

interface CommandPaletteProps {
  isMobile?: boolean;
}

// ─── Component ───────────────────────────────────────────────────────────────

export default function CommandPalette({
  isMobile = false,
}: CommandPaletteProps) {
  const isOpen = useVartaStore((s) => s.isCommandPaletteOpen);
  const toggle = useVartaStore((s) => s.toggleCommandPalette);
  const newTab = useVartaStore((s) => s.newTab);
  const openRequest = useVartaStore((s) => s.openRequest);
  const toggleHistory = useVartaStore((s) => s.toggleHistory);
  const setSettingsOpen = useSettingsStore((s) => s.setSettingsOpen);
  const collectionTrees = useWorkspaceStore((s) => s.collectionTrees);
  const openThemePicker = useWorkspaceStore((s) => s.openThemePicker);
  const openImportModal = useWorkspaceStore((s) => s.openImportModal);
  const openExportModal = useWorkspaceStore((s) => s.openExportModal);
  const activeWorkspaceId = useWorkspaceStore((s) => s.activeWorkspaceId);
  const activeCollectionId = useWorkspaceStore((s) => s.activeCollectionId);

  const [query, setQuery] = useState("");
  const [selectedIdx, setSelectedIdx] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // ── Build the flat items list ─────────────────────────────────────────────
  const allItems = useMemo<PaletteItem[]>(() => {
    const actions: PaletteItem[] = [
      {
        id: "action-new-tab",
        category: "actions",
        label: "New Request Tab",
        hint: isMobile ? undefined : "Ctrl+T",
        icon: <Plus className="w-3.5 h-3.5" />,
        action: () => newTab(),
      },
      {
        id: "action-import",
        category: "actions",
        label: "Import API Data (Postman, Insomnia, Yaak, Veyak)",
        hint: "Import",
        icon: <FolderInput className="w-3.5 h-3.5" />,
        action: () => {
          toggle(false);
          openImportModal();
        },
      },
      {
        id: "action-export-collection",
        category: "actions",
        label: "Export Active Collection",
        hint: "Export",
        icon: <Download className="w-3.5 h-3.5" />,
        action: () => {
          toggle(false);
          if (activeCollectionId) {
            openExportModal({ type: "collection", id: activeCollectionId });
          } else {
            openExportModal();
          }
        },
      },
      {
        id: "action-export-workspace",
        category: "actions",
        label: "Export Entire Workspace",
        icon: <Download className="w-3.5 h-3.5" />,
        action: () => {
          toggle(false);
          if (activeWorkspaceId) {
            openExportModal({ type: "workspace", id: activeWorkspaceId });
          } else {
            openExportModal({ type: "workspace" });
          }
        },
      },
      {
        id: "action-theme-picker",
        category: "actions",
        label: "Preferences: Color Theme",
        hint: isMobile ? undefined : "Ctrl+K Ctrl+T",
        icon: <Palette className="w-3.5 h-3.5" />,
        action: () => {
          toggle(false);
          openThemePicker();
        },
      },
      {
        id: "action-settings",
        category: "actions",
        label: "Open Settings",
        hint: isMobile ? undefined : "Ctrl+,",
        icon: <Settings2 className="w-3.5 h-3.5" />,
        action: () => setSettingsOpen(true),
      },
      {
        id: "action-history",
        category: "actions",
        label: "Toggle Request History",
        icon: <History className="w-3.5 h-3.5" />,
        action: () => toggleHistory(),
      },
    ];


    // Flatten all requests from all collections
    const requestItems: PaletteItem[] = [];

    for (const tree of collectionTrees) {
      const collectionName = tree.collection.name;

      const addReq = (req: RequestItem, folderName?: string) => {
        const breadcrumb = folderName
          ? `${collectionName} › ${folderName}`
          : collectionName;
        requestItems.push({
          id: `req-${req.id}`,
          category: "requests",
          label: req.name,
          method: req.method,
          breadcrumb,
          icon: <FileText className="w-3.5 h-3.5" />,
          action: () => openRequest(req),
        });
      };

      // Root-level requests
      for (const req of tree.requests) {
        addReq(req);
      }

      // Recursively traverse folders
      const traverseFolder = (
        node: (typeof tree.folders)[number],
        parentName?: string,
      ) => {
        const name = parentName
          ? `${parentName} / ${node.folder.name}`
          : node.folder.name;
        for (const req of node.requests) addReq(req, name);
        for (const child of node.children) traverseFolder(child, name);
      };
      for (const folder of tree.folders) traverseFolder(folder);
    }

    return [...actions, ...requestItems];
  }, [
    newTab,
    openRequest,
    toggleHistory,
    setSettingsOpen,
    collectionTrees,
    isMobile,
  ]);

  // ── Filter by query ───────────────────────────────────────────────────────
  const filtered = useMemo(() => {
    if (!query.trim()) return allItems;
    const q = query.toLowerCase();
    return allItems.filter(
      (item) =>
        item.label.toLowerCase().includes(q) ||
        item.breadcrumb?.toLowerCase().includes(q) ||
        item.method?.toLowerCase().includes(q),
    );
  }, [allItems, query]);

  // Group by category
  const actionItems = filtered.filter((i) => i.category === "actions");
  const requestItems = filtered.filter((i) => i.category === "requests");

  // Ordered flat list for keyboard nav
  const orderedFiltered = [...actionItems, ...requestItems];

  // ── Reset on open/close ───────────────────────────────────────────────────
  useEffect(() => {
    if (!isOpen) {
      setQuery("");
      setSelectedIdx(0);
    } else {
      // Focus input on next tick
      setTimeout(() => inputRef.current?.focus(), 10);
    }
  }, [isOpen]);

  // Reset selected index when query changes
  useEffect(() => {
    setSelectedIdx(0);
  }, [query]);

  // Scroll selected item into view
  useEffect(() => {
    const el = listRef.current?.querySelector<HTMLElement>(
      `[data-idx="${selectedIdx}"]`,
    );
    el?.scrollIntoView({ block: "nearest" });
  }, [selectedIdx]);

  // ── Keyboard navigation ───────────────────────────────────────────────────
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIdx((i) => Math.min(i + 1, orderedFiltered.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIdx((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const item = orderedFiltered[selectedIdx];
      if (item) {
        item.action();
        toggle(false);
      }
    } else if (e.key === "Escape") {
      toggle(false);
    }
  };

  const execute = (item: PaletteItem) => {
    item.action();
    toggle(false);
  };

  if (!isOpen) return null;

  return (
    <div
      className={`fixed inset-0 z-50 flex items-start justify-center bg-black/55 backdrop-blur-sm animate-in fade-in duration-150 ${isMobile ? "pt-[5vh]" : "pt-[13vh]"
        }`}
      onClick={() => toggle(false)}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className={`overflow-hidden rounded-xl border border-border bg-panel shadow-elevated animate-in zoom-in-95 duration-150 ${isMobile ? "w-[95vw]" : "w-140"
          }`}
      >
        {/* Search input */}
        <div className="flex items-center gap-2.5 border-b border-border px-3.5 py-3">
          <Search size={15} className="shrink-0 text-text-secondary" />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a command or search requests…"
            className="w-full bg-transparent text-sm text-text-primary placeholder:text-text-muted outline-none"
          />
          <kbd className="kbd shrink-0">Esc</kbd>
        </div>

        {/* Results */}
        <div ref={listRef} className="max-h-80 overflow-y-auto py-1.5">
          {orderedFiltered.length === 0 ? (
            <div className="flex flex-col items-center gap-2 py-8 text-center">
              <Zap className="w-6 h-6 text-text-muted opacity-40" />
              <p className="text-sm text-text-muted">
                No matches for "{query}"
              </p>
            </div>
          ) : (
            <>
              {/* ── Actions group ── */}
              {actionItems.length > 0 && (
                <Group
                  label="Actions"
                  items={actionItems}
                  globalOffset={0}
                  selectedIdx={selectedIdx}
                  onSelect={execute}
                />
              )}

              {/* ── Requests group ── */}
              {requestItems.length > 0 && (
                <Group
                  label="Requests"
                  items={requestItems}
                  globalOffset={actionItems.length}
                  selectedIdx={selectedIdx}
                  onSelect={execute}
                />
              )}
            </>
          )}
        </div>

        {/* Footer hint */}
        {!isMobile && (
          <div className="border-t border-border bg-panel-raised px-3.5 py-2 flex items-center gap-3 text-[10px] text-text-muted">
            <span className="flex items-center gap-1">
              <kbd className="kbd">↑↓</kbd> navigate
            </span>
            <span className="flex items-center gap-1">
              <kbd className="kbd">↵</kbd> open
            </span>
            <span className="flex items-center gap-1">
              <kbd className="kbd">Esc</kbd> close
            </span>
          </div>
        )}
      </div>
    </div>
  );
}

// ─── Group sub-component ─────────────────────────────────────────────────────

interface GroupProps {
  label: string;
  items: PaletteItem[];
  globalOffset: number;
  selectedIdx: number;
  onSelect: (item: PaletteItem) => void;
}

function Group({
  label,
  items,
  globalOffset,
  selectedIdx,
  onSelect,
}: GroupProps) {
  return (
    <div>
      <div className="px-3.5 pb-0.5 pt-2 text-[10px] font-bold tracking-widest text-text-muted uppercase select-none">
        {label}
      </div>
      {items.map((item, localIdx) => {
        const idx = globalOffset + localIdx;
        const isSelected = idx === selectedIdx;
        return (
          <button
            key={item.id}
            data-idx={idx}
            onClick={() => onSelect(item)}
            className={`flex w-full items-center gap-2.5 px-3.5 py-2 text-left transition-colors ${isSelected
                ? "bg-primary/15 text-text-primary"
                : "text-text-secondary hover:bg-panel-raised hover:text-text-primary"
              }`}
          >
            {/* Icon */}
            <span
              className={`shrink-0 ${isSelected ? "text-primary" : "text-text-muted"}`}
            >
              {item.icon}
            </span>

            {/* Label + breadcrumb */}
            <span className="flex min-w-0 flex-1 flex-col">
              <span className="truncate text-sm font-medium leading-snug text-text-primary">
                {item.label}
              </span>
              {item.breadcrumb && (
                <span className="flex items-center gap-1 truncate text-[10px] text-text-muted mt-0.5">
                  <ChevronRight className="w-2.5 h-2.5 shrink-0" />
                  {item.breadcrumb}
                </span>
              )}
            </span>

            {/* Right side: method badge or hint */}
            <span className="shrink-0 flex items-center gap-2">
              {item.method && (
                <span
                  className={`font-mono text-[10px] font-bold ${METHOD_COLORS[item.method] ?? "text-text-muted"
                    }`}
                >
                  {item.method}
                </span>
              )}
              {item.hint && <kbd className="kbd">{item.hint}</kbd>}
            </span>
          </button>
        );
      })}
    </div>
  );
}
