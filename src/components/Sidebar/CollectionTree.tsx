import React, { useEffect, useRef, useState, useCallback } from "react";
import {
  Plus,
  Loader2,
  Folder,
  Layers,
  FolderDown,
  FolderInput,
  Download,
} from "lucide-react";
import * as Icons from "lucide-react";
import { FolderNodeItem } from "./FolderNodeItem";
import { RequestItem } from "./RequestItem";
import { useWorkspaceStore } from "../../store/workspaceStore";
import {
  useTreeDragDrop,
  TreeDragDropProvider,
  DropPosition,
  DraggedItem,
} from "../../hooks/useTreeDragDrop";
import { FolderNode as FolderNodeType, CollectionTree as CollectionTreeType } from "@veyak-internal/models";

interface ItemTreeLocation {
  id: string;
  type: "folder" | "request";
  parentFolderId: string | null;
  sortOrder: number;
}

function findFolderInTree(
  folders: FolderNodeType[],
  folderId: string,
): FolderNodeType | null {
  for (const node of folders) {
    if (node.folder.id === folderId) return node;
    if (node.children && node.children.length > 0) {
      const found = findFolderInTree(node.children, folderId);
      if (found) return found;
    }
  }
  return null;
}

function findItemInTree(
  tree: CollectionTreeType,
  targetId: string,
): ItemTreeLocation | null {
  // Check root folders
  for (const f of tree.folders) {
    if (f.folder.id === targetId) {
      return {
        id: f.folder.id,
        type: "folder",
        parentFolderId: null,
        sortOrder: Number(f.folder.sortOrder ?? 0),
      };
    }
  }

  // Check root requests
  for (const r of tree.requests) {
    if (r.id === targetId) {
      return {
        id: r.id,
        type: "request",
        parentFolderId: null,
        sortOrder: Number((r as any).sortOrder ?? 0),
      };
    }
  }

  // Check nested in folders
  const searchNested = (
    parentFolder: FolderNodeType,
  ): ItemTreeLocation | null => {
    for (const child of parentFolder.children || []) {
      if (child.folder.id === targetId) {
        return {
          id: child.folder.id,
          type: "folder",
          parentFolderId: parentFolder.folder.id,
          sortOrder: Number(child.folder.sortOrder ?? 0),
        };
      }
      const found = searchNested(child);
      if (found) return found;
    }
    for (const req of parentFolder.requests || []) {
      if (req.id === targetId) {
        return {
          id: req.id,
          type: "request",
          parentFolderId: parentFolder.folder.id,
          sortOrder: Number((req as any).sortOrder ?? 0),
        };
      }
    }
    return null;
  };

  for (const f of tree.folders) {
    const found = searchNested(f);
    if (found) return found;
  }

  return null;
}

export const CollectionsTree: React.FC = () => {
  const {
    activeWorkspaceId,
    collections,
    activeCollectionTree,
    isLoadingCollectionTree,
    fetchCollections,
    createRequest,
    createFolder,
    createWs,
    additionTypes,
    fetchAdditionTypes,
    moveItem,
    openImportModal,
    openExportModal,
  } = useWorkspaceStore();

  // Action menu & Child Item creation for active collection
  const [activeMenuOpen, setActiveMenuOpen] = useState(false);
  const [addingItem, setAddingItem] = useState<{
    collectionId: string;
    type: "folder" | "request" | "ws" | "grpc" | "graphql";
  } | null>(null);
  const [newItemName, setNewItemName] = useState("");

  const menuRef = useRef<HTMLDivElement>(null);

  // Fetch collections when the active workspace changes
  useEffect(() => {
    if (activeWorkspaceId) {
      fetchCollections();
    }
  }, [activeWorkspaceId, fetchCollections]);

  // Fetch addition types on mount
  useEffect(() => {
    fetchAdditionTypes();
  }, [fetchAdditionTypes]);

  // Click-outside listener for item creation menu
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        menuRef.current &&
        !menuRef.current.contains(event.target as Node)
      ) {
        setActiveMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const handleCreateItem = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newItemName.trim() || !addingItem) return;

    if (addingItem.type === "folder") {
      await createFolder(addingItem.collectionId, null, newItemName);
    } else if (addingItem.type === "ws") {
      await createWs(addingItem.collectionId, null, newItemName);
    } else if (addingItem.type === "grpc") {
      await createRequest(addingItem.collectionId, null, newItemName, "GRPC");
    } else if (addingItem.type === "graphql") {
      await createRequest(addingItem.collectionId, null, newItemName, "GRAPHQL");
    } else {
      await createRequest(addingItem.collectionId, null, newItemName, "REST");
    }

    setAddingItem(null);
    setNewItemName("");
  };

  const handleMoveNode = useCallback(
    async (
      source: DraggedItem,
      targetId: string,
      position: DropPosition,
    ) => {
      if (!activeCollectionTree) return;

      // 1. Move to root level (via bottom root drop zone or collection header)
      if (
        position === "root" ||
        targetId === "root" ||
        targetId === activeCollectionTree.collection.id
      ) {
        const targetFolderId = null;
        const newSortOrder = 999999;
        await moveItem(source.id, source.type, targetFolderId, newSortOrder);
        return;
      }

      // 2. Move inside a folder
      if (position === "inside") {
        const targetFolderNode = findFolderInTree(
          activeCollectionTree.folders,
          targetId,
        );
        if (!targetFolderNode) return;
        const targetFolderId = targetFolderNode.folder.id;
        const newSortOrder = 999999;
        await moveItem(source.id, source.type, targetFolderId, newSortOrder);
        return;
      }

      // 3. Move before or after an existing node (folder or request)
      const loc = findItemInTree(activeCollectionTree, targetId);
      if (!loc) return;

      const targetFolderId = loc.parentFolderId;
      const newSortOrder =
        position === "before" ? loc.sortOrder - 5 : loc.sortOrder + 5;

      await moveItem(source.id, source.type, targetFolderId, newSortOrder);
    },
    [activeCollectionTree, moveItem],
  );

  const dnd = useTreeDragDrop({
    folders: activeCollectionTree?.folders || [],
    rootRequests: activeCollectionTree?.requests || [],
    onMoveNode: handleMoveNode,
  });

  const isRootDropTarget = dnd.dropTarget?.targetId === "root";

  return (
    <TreeDragDropProvider value={dnd}>
      <div className="mt-2 flex-1 overflow-y-auto flex flex-col">
        {/* ── Active Collection Actions Bar & Tree View ── */}
        <div className="flex-1 flex flex-col">
          {collections.length === 0 ? (
            <div className="px-4 py-8 text-center text-xs text-text-muted italic flex flex-col items-center gap-2">
              <Layers className="w-6 h-6 text-text-muted/40 mb-1" />
              <span>No collections in this workspace.</span>
              <span className="text-[11px] text-text-muted">
                Use the top bar to create a collection or import one.
              </span>
              <button
                onClick={() => openImportModal()}
                className="mt-2 flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-primary/10 border border-primary/30 text-primary hover:bg-primary/20 text-xs font-medium transition-colors cursor-pointer not-italic"
              >
                <FolderInput className="w-3.5 h-3.5" />
                Import Collection
              </button>
            </div>
          ) : !activeCollectionTree ? (
            isLoadingCollectionTree ? (
              <div className="flex items-center justify-center py-8 text-xs text-text-muted gap-2">
                <Loader2 className="w-4 h-4 animate-spin text-primary" />
                <span>Loading collection...</span>
              </div>
            ) : (
              <div className="px-4 py-8 text-center text-xs text-text-muted italic flex flex-col items-center gap-1">
                <Folder className="w-6 h-6 text-text-muted/40 mb-1" />
                <span>Select a collection from the top bar to view requests.</span>
              </div>
            )
          ) : (
            <div className="flex flex-col flex-1">
              {/* Active Collection Header & Quick Add Bar (also accepts drop to root) */}
              <div
                {...dnd.getRootDropTargetProps()}
                className={`group flex items-center justify-between px-3 py-1.5 mx-1 rounded-md text-xs font-semibold transition-all ${
                  isRootDropTarget && dnd.draggedId
                    ? "bg-primary/15 ring-1 ring-primary text-primary"
                    : "text-text-primary hover:bg-panel/60"
                }`}
              >
                <div className="flex items-center gap-1.5 truncate">
                  <Folder className="w-3.5 h-3.5 text-primary shrink-0" />
                  <span className="truncate font-semibold">
                    {activeCollectionTree.collection.name}
                  </span>
                </div>

                {/* Actions: Import, Export, Add item dropdown menu */}
                <div className="flex items-center gap-0.5">
                  <button
                    onClick={() => openImportModal(activeCollectionTree.collection.id)}
                    className="p-1 rounded text-text-muted hover:text-text-primary hover:bg-borderMuted cursor-pointer transition-colors"
                    title="Import into collection..."
                  >
                    <FolderInput className="w-3.5 h-3.5" />
                  </button>

                  <button
                    onClick={() =>
                      openExportModal({
                        type: "collection",
                        id: activeCollectionTree.collection.id,
                      })
                    }
                    className="p-1 rounded text-text-muted hover:text-text-primary hover:bg-borderMuted cursor-pointer transition-colors"
                    title="Export collection..."
                  >
                    <Download className="w-3.5 h-3.5" />
                  </button>

                  <div className="relative" ref={menuRef}>
                    <button
                      onClick={() => setActiveMenuOpen(!activeMenuOpen)}
                      className="p-1 rounded text-text-muted hover:text-text-primary hover:bg-borderMuted cursor-pointer transition-colors"
                      title="Add to collection..."
                    >
                      <Plus className="w-3.5 h-3.5" />
                    </button>

                    {activeMenuOpen && (
                      <div className="absolute right-0 top-full mt-1 w-45 py-1 z-50 bg-panel-raised border border-border shadow-elevated rounded-md animate-in fade-in zoom-in-95 duration-100">
                        {additionTypes.map((type) => {
                          const IconComp = (Icons as any)[type.icon] || Icons.FilePlus;
                          return (
                            <button
                              key={type.id}
                              onClick={() => {
                                setAddingItem({
                                  collectionId: activeCollectionTree.collection.id,
                                  type: type.id as any,
                                });
                                setActiveMenuOpen(false);
                              }}
                              className="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-text-secondary hover:bg-panel hover:text-text-primary transition-colors cursor-pointer"
                            >
                              <IconComp className="w-3.5 h-3.5" />
                              {type.label}
                            </button>
                          );
                        })}

                        <div className="my-1 border-t border-border/40" />

                        <button
                          onClick={() => {
                            openImportModal(activeCollectionTree.collection.id);
                            setActiveMenuOpen(false);
                          }}
                          className="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-text-secondary hover:bg-panel hover:text-text-primary transition-colors cursor-pointer"
                        >
                          <FolderInput className="w-3.5 h-3.5 text-primary" />
                          Import into collection...
                        </button>

                        <button
                          onClick={() => {
                            openExportModal({
                              type: "collection",
                              id: activeCollectionTree.collection.id,
                            });
                            setActiveMenuOpen(false);
                          }}
                          className="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-text-secondary hover:bg-panel hover:text-text-primary transition-colors cursor-pointer"
                        >
                          <Download className="w-3.5 h-3.5 text-primary" />
                          Export collection...
                        </button>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              {/* Inline Create Form for Root Items (Requests / Folders) */}
              {addingItem?.collectionId === activeCollectionTree.collection.id && (
                <form
                  onSubmit={handleCreateItem}
                  className="pl-5 pr-3 py-1 flex gap-1 mt-1"
                >
                  <input
                    autoFocus
                    type="text"
                    placeholder={`New ${addingItem.type} name...`}
                    value={newItemName}
                    onChange={(e) => setNewItemName(e.target.value)}
                    onBlur={() => {
                      if (!newItemName.trim()) setAddingItem(null);
                    }}
                    className="input-shell w-full py-0.5 px-2 text-xs"
                  />
                </form>
              )}

              {/* Collection Tree Content */}
              <div className="flex flex-col gap-0.5 pb-4 mt-1">
                {/* Folders in Root */}
                {activeCollectionTree.folders.map((folderNode) => (
                  <FolderNodeItem key={folderNode.folder.id} node={folderNode} />
                ))}

                {/* Requests in Root */}
                {activeCollectionTree.requests.map((req) => (
                  <div key={req.id} className="pl-4">
                    <RequestItem request={req} />
                  </div>
                ))}

                {/* Empty Collection State */}
                {activeCollectionTree.folders.length === 0 &&
                  activeCollectionTree.requests.length === 0 &&
                  !addingItem && (
                    <div className="px-4 py-5 text-xs text-text-muted flex flex-col items-center gap-2">
                      <span className="italic">Collection is empty.</span>
                      <div className="flex items-center gap-2 mt-1">
                        <button
                          onClick={() => {
                            setAddingItem({
                              collectionId: activeCollectionTree.collection.id,
                              type: "request",
                            });
                          }}
                          className="flex items-center gap-1 px-2.5 py-1 rounded bg-panel border border-border hover:bg-panel-raised text-text-primary text-xs cursor-pointer"
                        >
                          <Plus className="w-3 h-3" />
                          Add Request
                        </button>
                        <button
                          onClick={() =>
                            openImportModal(activeCollectionTree.collection.id)
                          }
                          className="flex items-center gap-1 px-2.5 py-1 rounded bg-primary/10 border border-primary/30 hover:bg-primary/20 text-primary text-xs font-medium cursor-pointer"
                        >
                          <FolderInput className="w-3 h-3" />
                          Import
                        </button>
                      </div>
                    </div>
                  )}

                {/* Root Drop Zone: displayed when dragging to allow moving items back to root level */}
                {dnd.draggedId && (
                  <div
                    {...dnd.getRootDropTargetProps()}
                    className={`mx-2 mt-3 mb-2 p-3 rounded-lg border-2 border-dashed transition-all duration-150 flex items-center justify-center gap-2 text-xs select-none ${
                      isRootDropTarget
                        ? "border-primary bg-primary/10 text-primary shadow-sm scale-[1.01]"
                        : "border-borderMuted/60 bg-panel/30 text-text-muted hover:border-primary/40 hover:text-text-secondary"
                    }`}
                  >
                    <FolderDown className="w-4 h-4 shrink-0" />
                    <span>Drop here to move to root</span>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </TreeDragDropProvider>
  );
};
