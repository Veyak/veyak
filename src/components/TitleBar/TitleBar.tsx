import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Search,
  Minus,
  Square,
  Copy,
  X,
  ChevronRight,
  Layers,
  Library,
  Cloud,
  SlidersHorizontal,
  History,
  Settings,
  Sparkles,
  Check,
  Loader2,
  FolderInput,
  Download,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useVartaStore } from "../../store/vartaStore";
import { useWorkspaceStore } from "../../store/workspaceStore";
import { UserProfileMenu } from "../UserProfileMenu";
import { useSettingsStore } from "../../store/settingStore";
import { TitleBarDropdown } from "./TitleBarDropdown";

const appWindow = getCurrentWindow();
const isMac =
  typeof navigator !== "undefined" &&
  (navigator.platform.toUpperCase().indexOf("MAC") >= 0 ||
    navigator.userAgent.toUpperCase().indexOf("MAC") >= 0);

export default function Titlebar() {
  const [isMaximized, setIsMaximized] = useState(false);
  const activeTab = useVartaStore((s) => s.activeTab);
  const toggleCommandPalette = useVartaStore((s) => s.toggleCommandPalette);
  const openEnvEditor = useVartaStore((s) => s.openEnvEditor);
  const toggleHistory = useVartaStore((s) => s.toggleHistory);
  const setSettingsOpen = useSettingsStore((s) => s.setSettingsOpen);

  // Store states
  const {
    workspaces,
    activeWorkspaceId,
    fetchWorkspaces,
    createWorkspace,
    renameWorkspace,
    deleteWorkspace,
    setActiveWorkspace,
    getActiveState,
    collections,
    activeCollectionId,
    fetchCollections,
    setActiveCollection,
    createCollection,
    renameCollection,
    deleteCollection,
    isLoadingCollections,

    environments,
    activeEnvironmentId,
    fetchEnvironments,
    setActiveEnvironment,
    renameEnvironment,
    deleteEnvironment,
    createEnvironment,

    themes,
    previewedTheme,
    activeThemeId,
    setActiveThemeId,
    revertThemePreview,
    confirmInstallTheme,
    pendingTheme,
    pendingThemeId,
    isInstallingTheme,
    openImportModal,
    openExportModal,
  } = useWorkspaceStore();

  const [activeDropdown, setActiveDropdown] = useState<
    "ws" | "col" | "env" | null
  >(null);

  useEffect(() => {
    fetchWorkspaces();
    getActiveState();
  }, [fetchWorkspaces, getActiveState]);

  useEffect(() => {
    if (activeWorkspaceId) {
      fetchCollections();
      fetchEnvironments(activeWorkspaceId);
    }
  }, [activeWorkspaceId, fetchCollections, fetchEnvironments]);

  useEffect(() => {
    const unlisten = appWindow.onResized(async () => {
      setIsMaximized(await appWindow.isMaximized());
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const activeWorkspace = workspaces.find((w) => w.id === activeWorkspaceId);
  const activeCollection = collections.find((c) => c.id === activeCollectionId);
  const activeEnv = environments.find(
    (e) => e.environment.id === activeEnvironmentId,
  );

  const envItems = useMemo(
    () =>
      environments.map((e) => ({
        id: e.environment.id,
        name: e.environment.name,
      })),
    [environments],
  );

  return (
    <header
      data-tauri-drag-region
      className={`relative z-50 flex h-10 w-full select-none items-center justify-between border-b border-border/40 bg-panel/85 px-3 backdrop-blur-md ${isMac ? "pl-[78px]" : "pl-3"
        }`}
    >
      {/* ── Left Section: Workspace, Collection & Environment Selector Breadcrumb ── */}
      <div className="flex items-center gap-1.5" data-tauri-drag-region>
        {!isMac && (
          <img
            src="/icon.png"
            alt="Veyak"
            className="h-4 w-4 object-contain mr-1.5"
          />
        )}

        {/* 1. Workspace Selector */}
        <TitleBarDropdown
          triggerIcon={<Layers className="w-3.5 h-3.5 text-primary/90 shrink-0" />}
          triggerLabel={activeWorkspace?.name || "Workspace"}
          triggerTitle="Switch Workspace"
          triggerMaxWidth="max-w-[110px]"
          headerTitle="Workspaces"
          dropdownWidth="w-56"
          emptyMessage="No workspaces found"
          items={workspaces}
          activeId={activeWorkspaceId}
          onSelect={(id) => setActiveWorkspace(id)}
          onRename={(id, name) => renameWorkspace(id, name)}
          onDelete={(id) => deleteWorkspace(id)}
          createButtonLabel="Create Workspace"
          createPlaceholder="Workspace name..."
          onCreate={(name) => createWorkspace(name)}
          renderExtraActions={(item, closeDropdown) => (
            <button
              onClick={(e) => {
                e.stopPropagation();
                openExportModal({ type: "workspace", id: item.id });
                closeDropdown();
              }}
              className="p-1 rounded text-text-muted hover:text-text-primary hover:bg-borderMuted cursor-pointer"
              title="Export Workspace"
            >
              <Download className="w-3 h-3" />
            </button>
          )}
          renderFooter={(closeDropdown) => (
            <div className="flex items-center gap-1">
              <button
                onClick={() => {
                  openImportModal();
                  closeDropdown();
                }}
                className="flex items-center justify-center gap-1.5 flex-1 py-1 text-xs text-text-secondary hover:text-text-primary hover:bg-panel rounded transition-colors cursor-pointer"
              >
                <FolderInput className="w-3 h-3 text-primary" />
                Import...
              </button>
              <button
                onClick={() => {
                  if (activeWorkspaceId) {
                    openExportModal({ type: "workspace", id: activeWorkspaceId });
                    closeDropdown();
                  }
                }}
                className="flex items-center justify-center gap-1.5 flex-1 py-1 text-xs text-text-secondary hover:text-text-primary hover:bg-panel rounded transition-colors cursor-pointer"
              >
                <Download className="w-3 h-3 text-primary" />
                Export...
              </button>
            </div>
          )}
          isOpen={activeDropdown === "ws"}
          onOpenChange={(open) => setActiveDropdown(open ? "ws" : null)}
        />

        {/* Separator */}
        <ChevronRight className="w-3 h-3 text-text-muted/50 shrink-0" />

        {/* 2. Collection Selector */}
        <TitleBarDropdown
          triggerIcon={<Library className="w-3.5 h-3.5 text-primary/80 shrink-0" />}
          triggerLabel={
            activeCollection?.name ||
            (collections.length === 0 ? "No Collections" : "Select Collection")
          }
          triggerTitle="Switch Collection"
          triggerMaxWidth="max-w-[120px]"
          isLoading={isLoadingCollections}
          headerTitle="Collections"
          dropdownWidth="w-56"
          emptyMessage="No collections in this workspace"
          items={collections}
          activeId={activeCollectionId}
          onSelect={(id) => setActiveCollection(id)}
          onRename={(id, name) => renameCollection(id, name)}
          onDelete={(id) => deleteCollection(id)}
          createButtonLabel="Create Collection"
          createPlaceholder="Collection name..."
          onCreate={(name) => createCollection(name)}
          renderExtraActions={(item, closeDropdown) => (
            <button
              onClick={(e) => {
                e.stopPropagation();
                openExportModal({ type: "collection", id: item.id });
                closeDropdown();
              }}
              className="p-1 rounded text-text-muted hover:text-text-primary hover:bg-borderMuted cursor-pointer"
              title="Export Collection"
            >
              <Download className="w-3 h-3" />
            </button>
          )}
          renderFooter={(closeDropdown) => (
            <div className="flex items-center gap-1">
              <button
                onClick={() => {
                  openImportModal(activeCollectionId || undefined);
                  closeDropdown();
                }}
                className="flex items-center justify-center gap-1.5 flex-1 py-1 text-xs text-text-secondary hover:text-text-primary hover:bg-panel rounded transition-colors cursor-pointer"
              >
                <FolderInput className="w-3 h-3 text-primary" />
                Import...
              </button>
              <button
                onClick={() => {
                  if (activeCollectionId) {
                    openExportModal({ type: "collection", id: activeCollectionId });
                    closeDropdown();
                  }
                }}
                disabled={!activeCollectionId}
                className="flex items-center justify-center gap-1.5 flex-1 py-1 text-xs text-text-secondary hover:text-text-primary hover:bg-panel rounded transition-colors cursor-pointer disabled:opacity-40"
              >
                <Download className="w-3 h-3 text-primary" />
                Export...
              </button>
            </div>
          )}
          isOpen={activeDropdown === "col"}
          onOpenChange={(open) => setActiveDropdown(open ? "col" : null)}
        />

        {/* Separator */}
        <ChevronRight className="w-3 h-3 text-text-muted/50 shrink-0" />

        {/* 3. Environment Selector */}
        <TitleBarDropdown
          triggerIcon={<Cloud className="w-3.5 h-3.5 text-primary/80 shrink-0" />}
          triggerLabel={activeEnv?.environment.name || "No Environment"}
          triggerTitle="Switch Environment"
          triggerMaxWidth="max-w-[120px]"
          headerTitle="Environments"
          dropdownWidth="w-60"
          emptyMessage="No environments found"
          items={envItems}
          activeId={activeEnvironmentId}
          onSelect={(id) => setActiveEnvironment(id)}
          noneOption={{
            label: "No Environment",
            isSelected: !activeEnvironmentId,
            onSelect: () => setActiveEnvironment(null),
          }}
          onRename={(id, name) => renameEnvironment(id, name)}
          onDelete={(id) => deleteEnvironment(id)}
          renderExtraActions={(item, closeDropdown) => (
            <button
              onClick={(e) => {
                e.stopPropagation();
                setActiveEnvironment(item.id);
                openEnvEditor();
                closeDropdown();
              }}
              className="p-1 rounded text-text-muted hover:text-text-primary hover:bg-borderMuted cursor-pointer"
              title="Edit Variables"
            >
              <SlidersHorizontal className="w-3 h-3" />
            </button>
          )}
          renderFooter={(closeDropdown) => (
            <div className="flex items-center gap-1">
              <button
                onClick={() => {
                  openImportModal();
                  closeDropdown();
                }}
                className="flex items-center justify-center gap-1.5 flex-1 py-1 text-xs text-text-secondary hover:text-text-primary hover:bg-panel rounded transition-colors cursor-pointer"
              >
                <FolderInput className="w-3 h-3 text-primary" />
                Import...
              </button>
              <button
                onClick={() => {
                  if (activeEnvironmentId) {
                    openExportModal({ type: "environment", id: activeEnvironmentId });
                    closeDropdown();
                  }
                }}
                disabled={!activeEnvironmentId}
                className="flex items-center justify-center gap-1.5 flex-1 py-1 text-xs text-text-secondary hover:text-text-primary hover:bg-panel rounded transition-colors cursor-pointer disabled:opacity-40"
              >
                <Download className="w-3 h-3 text-primary" />
                Export...
              </button>
            </div>
          )}
          createButtonLabel="Create Environment"
          createPlaceholder="Environment name..."
          onCreate={async (name) => {
            if (!activeWorkspaceId) {
              console.warn(
                "Cannot create environment: No active workspace selected.",
              );
              return;
            }
            await createEnvironment(activeWorkspaceId, name);
          }}
          isOpen={activeDropdown === "env"}
          onOpenChange={(open) => setActiveDropdown(open ? "env" : null)}
        />
      </div>

      {/* ── Center Section: Antigravity Command Pill & Theme Preview Bar ── */}
      <div
        className="flex flex-1 items-center justify-center max-w-lg px-4"
        data-tauri-drag-region
      >
        {previewedTheme && previewedTheme.id !== activeThemeId ? (
          <div className="flex items-center gap-2.5 px-3 py-0.5 rounded-full bg-primary/15 border border-primary/40 text-xs shadow-xs animate-in fade-in zoom-in-95 duration-150">
            <div className="flex items-center gap-1.5 font-medium">
              <Sparkles className="w-3.5 h-3.5 text-primary animate-pulse" />
              <span className="text-[11px] text-text-muted">Preview:</span>
              <span className="text-[11px] font-semibold text-text-primary truncate max-w-[130px]">
                {previewedTheme.name}
              </span>
            </div>
            <div className="flex items-center gap-1.5 ml-1">
              <button
                onClick={async () => {
                  if (
                    pendingTheme ||
                    pendingThemeId ||
                    !themes.some((t) => t.id === previewedTheme.id)
                  ) {
                    await confirmInstallTheme();
                  } else {
                    await setActiveThemeId(previewedTheme.id);
                  }
                }}
                disabled={isInstallingTheme}
                className="flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-primary text-white text-[10.5px] font-semibold hover:bg-primary-hover transition-colors cursor-pointer shadow-xs disabled:opacity-50"
              >
                {isInstallingTheme ? (
                  <Loader2 className="w-3 h-3 animate-spin" />
                ) : (
                  <Check className="w-3 h-3" />
                )}
                {isInstallingTheme ? "Applying..." : "Apply Theme"}
              </button>
              <button
                onClick={() => revertThemePreview()}
                disabled={isInstallingTheme}
                className="flex items-center gap-1 px-2 py-0.5 rounded-full bg-panel hover:bg-error/20 hover:text-error text-text-secondary text-[10.5px] transition-colors cursor-pointer border border-border disabled:opacity-50"
              >
                <X className="w-3 h-3" />
                Revert
              </button>
            </div>
          </div>
        ) : (
          <button
            onClick={() => toggleCommandPalette(true)}
            className="group flex w-full items-center justify-between gap-2 rounded-md border border-border/50 bg-bg/60 px-3 py-1 text-xs text-text-muted transition-all duration-150 hover:border-primary/50 hover:bg-panel hover:text-text-primary hover:shadow-sm cursor-pointer"
          >
            <div className="flex items-center gap-2 truncate">
              <Search className="h-3.5 w-3.5 text-text-muted group-hover:text-primary transition-colors" />
              <span className="truncate font-normal">
                {activeTab?.request.name ? (
                  <span className="flex items-center gap-1.5">
                    <span className="font-semibold text-primary">
                      {activeTab.request.method}
                    </span>
                    <span className="text-text-secondary">
                      {activeTab.request.name}
                    </span>
                  </span>
                ) : (
                  "Search requests, commands, or tools..."
                )}
              </span>
            </div>

            <kbd className="pointer-events-none inline-flex h-4 select-none items-center gap-0.5 rounded border border-border bg-panel px-1.5 font-mono text-[10px] font-medium text-text-muted">
              <span className="text-[11px]">{isMac ? "⌘" : "Ctrl+"}</span>K
            </kbd>
          </button>
        )}
      </div>

      {/* ── Right Section: User Profile, Status Pill & Window Controls (Windows/Linux) ── */}
      <div className="flex items-center gap-2" data-tauri-drag-region>
        <button
          onClick={() => toggleHistory()}
          className="rounded-md p-1.5 text-text-secondary hover:bg-panel-raised hover:text-text-primary"
          aria-label="History"
        >
          <History size={15} />
        </button>
        <button
          onClick={() => setSettingsOpen(true)}
          className="rounded-md p-1.5 text-text-secondary hover:bg-panel-raised hover:text-text-primary"
          aria-label="Settings"
        >
          <Settings size={15} />
        </button>

        {/* User Login / Profile Menu */}
        <UserProfileMenu />

        {/* Divider */}
        <div className="h-3.5 w-px bg-border/40" />

        {/* Windows / Linux Controls */}
        {!isMac && (
          <div className="flex items-center ml-1 text-text-secondary">
            <button
              onClick={() => appWindow.minimize()}
              className="flex h-7 w-8 items-center justify-center rounded hover:bg-panel-raised hover:text-text-primary transition-colors cursor-pointer"
              title="Minimize"
            >
              <Minus size={14} />
            </button>
            <button
              onClick={() => appWindow.toggleMaximize()}
              className="flex h-7 w-8 items-center justify-center rounded hover:bg-panel-raised hover:text-text-primary transition-colors cursor-pointer"
              title={isMaximized ? "Restore" : "Maximize"}
            >
              {isMaximized ? (
                <Copy size={12} className="rotate-180" />
              ) : (
                <Square size={12} />
              )}
            </button>
            <button
              onClick={() => appWindow.close()}
              className="flex h-7 w-8 items-center justify-center rounded hover:bg-error/20 hover:text-error transition-colors cursor-pointer"
              title="Close"
            >
              <X size={14} />
            </button>
          </div>
        )}
      </div>
    </header>
  );
}
