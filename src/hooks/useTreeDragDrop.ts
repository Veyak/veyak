import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { FolderNode as FolderNodeType, RequestItem } from "@veyak-internal/models";

export type DropPosition = "before" | "inside" | "after" | "root";
export type DragItemType = "folder" | "request";

export interface DraggedItem {
  id: string;
  type: DragItemType;
  parentFolderId?: string | null;
}

export interface DropTargetState {
  targetId: string;
  targetType: "folder" | "request" | "root";
  position: DropPosition;
}

export interface TreeNodeBase {
  id: string;
  isFolder: boolean;
  parentFolderId?: string | null;
  children?: TreeNodeBase[];
}

export interface UseTreeDragDropOptions {
  folders: FolderNodeType[];
  rootRequests?: RequestItem[];
  autoExpandDelayMs?: number;
  onExpandNode?: (folderId: string) => void;
  onMoveNode: (
    source: DraggedItem,
    targetId: string,
    position: DropPosition,
  ) => void | Promise<void>;
}

export interface TreeDragDropContextValue {
  draggedItem: DraggedItem | null;
  draggedId: string | null;
  draggedType: DragItemType | null;
  dropTarget: DropTargetState | null;
  getDragSourceProps: (
    nodeId: string,
    type: DragItemType,
    options?: { disabled?: boolean; parentFolderId?: string | null },
  ) => {
    draggable: boolean;
    onDragStart: (e: React.DragEvent) => void;
    onDragEnd: (e: React.DragEvent) => void;
  };
  getDropTargetProps: (options: {
    id: string;
    isFolder: boolean;
    isExpanded?: boolean;
    onExpand?: () => void;
  }) => {
    onDragOver: (e: React.DragEvent) => void;
    onDragLeave: (e: React.DragEvent) => void;
    onDrop: (e: React.DragEvent) => void;
  };
  getRootDropTargetProps: () => {
    onDragOver: (e: React.DragEvent) => void;
    onDragLeave: (e: React.DragEvent) => void;
    onDrop: (e: React.DragEvent) => void;
  };
}

/**
 * Checks whether `candidateChildId` is `parentSourceId` or resides in its subtree (circular drop guard).
 */
export function isDescendantNode(
  folders: FolderNodeType[],
  parentSourceId: string,
  candidateChildId: string,
): boolean {
  // Disallow dropping onto itself
  if (parentSourceId === candidateChildId) return true;

  const findFolder = (list: FolderNodeType[]): FolderNodeType | null => {
    for (const node of list) {
      if (node.folder.id === parentSourceId) return node;
      if (node.children && node.children.length > 0) {
        const found = findFolder(node.children);
        if (found) return found;
      }
    }
    return null;
  };

  const sourceNode = findFolder(folders);
  if (!sourceNode) return false;

  const checkSubtree = (node: FolderNodeType): boolean => {
    for (const child of node.children || []) {
      if (child.folder.id === candidateChildId) return true;
      if (checkSubtree(child)) return true;
    }
    for (const req of node.requests || []) {
      if (req.id === candidateChildId) return true;
    }
    return false;
  };

  return checkSubtree(sourceNode);
}

const TreeDragDropContext = createContext<TreeDragDropContextValue | null>(null);

export const TreeDragDropProvider = TreeDragDropContext.Provider;

export function useTreeDragDropContext(): TreeDragDropContextValue | null {
  return useContext(TreeDragDropContext);
}

export function useTreeDragDrop({
  folders,
  autoExpandDelayMs = 500,
  onExpandNode,
  onMoveNode,
}: UseTreeDragDropOptions): TreeDragDropContextValue {
  const [draggedItem, setDraggedItem] = useState<DraggedItem | null>(null);
  const [dropTarget, setDropTarget] = useState<DropTargetState | null>(null);

  const draggedItemRef = useRef<DraggedItem | null>(null);
  const expandTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const hoveredFolderIdRef = useRef<string | null>(null);
  const rafHandleRef = useRef<number | null>(null);

  draggedItemRef.current = draggedItem;

  const clearExpandTimer = useCallback(() => {
    if (expandTimerRef.current) {
      clearTimeout(expandTimerRef.current);
      expandTimerRef.current = null;
    }
    hoveredFolderIdRef.current = null;
  }, []);

  useEffect(() => {
    return () => {
      clearExpandTimer();
      if (rafHandleRef.current) {
        cancelAnimationFrame(rafHandleRef.current);
      }
    };
  }, [clearExpandTimer]);

  const handleDragStart = useCallback(
    (
      nodeId: string,
      type: DragItemType,
      parentFolderId: string | null | undefined,
      e: React.DragEvent,
    ) => {
      e.stopPropagation();
      const item: DraggedItem = { id: nodeId, type, parentFolderId };
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", nodeId);
      e.dataTransfer.setData("application/json", JSON.stringify(item));
      setDraggedItem(item);
    },
    [],
  );

  const handleDragEnd = useCallback(() => {
    clearExpandTimer();
    if (rafHandleRef.current) {
      cancelAnimationFrame(rafHandleRef.current);
      rafHandleRef.current = null;
    }
    setDraggedItem(null);
    setDropTarget(null);
  }, [clearExpandTimer]);

  const handleDragOver = useCallback(
    (
      node: {
        id: string;
        isFolder: boolean;
        isExpanded?: boolean;
        onExpand?: () => void;
      },
      e: React.DragEvent,
    ) => {
      e.preventDefault();
      e.stopPropagation();

      const source = draggedItemRef.current;
      if (!source) return;

      // 1. Circular drop guard
      if (source.type === "folder" && isDescendantNode(folders, source.id, node.id)) {
        e.dataTransfer.dropEffect = "none";
        if (dropTarget !== null) setDropTarget(null);
        clearExpandTimer();
        return;
      }

      // If dragging a request and target is itself
      if (source.type === "request" && source.id === node.id) {
        e.dataTransfer.dropEffect = "none";
        if (dropTarget !== null) setDropTarget(null);
        clearExpandTimer();
        return;
      }

      e.dataTransfer.dropEffect = "move";

      // 2. Throttle coordinate calculations using requestAnimationFrame
      const clientY = e.clientY;
      const targetElement = e.currentTarget as HTMLElement;

      if (rafHandleRef.current) cancelAnimationFrame(rafHandleRef.current);

      rafHandleRef.current = requestAnimationFrame(() => {
        const isFolder = node.isFolder;
        const rect = targetElement.getBoundingClientRect();
        const offsetY = clientY - rect.top;
        const height = rect.height;

        let calculatedPos: DropPosition;
        if (!isFolder) {
          calculatedPos = offsetY < height / 2 ? "before" : "after";
        } else {
          // 25% top/bottom thresholds for reordering around folders; middle 50% for nesting
          if (offsetY < height * 0.25) calculatedPos = "before";
          else if (offsetY > height * 0.75) calculatedPos = "after";
          else calculatedPos = "inside";
        }

        const newTarget: DropTargetState = {
          targetId: node.id,
          targetType: isFolder ? "folder" : "request",
          position: calculatedPos,
        };

        setDropTarget((prev) => {
          if (prev?.targetId === newTarget.targetId && prev?.position === newTarget.position) {
            return prev;
          }
          return newTarget;
        });

        // 3. Auto-expand on hover (500ms hover over closed folder)
        if (isFolder && !node.isExpanded && calculatedPos === "inside") {
          if (hoveredFolderIdRef.current !== node.id) {
            clearExpandTimer();
            hoveredFolderIdRef.current = node.id;
            expandTimerRef.current = setTimeout(() => {
              node.onExpand?.();
              onExpandNode?.(node.id);
              clearExpandTimer();
            }, autoExpandDelayMs);
          }
        } else {
          clearExpandTimer();
        }
      });
    },
    [folders, dropTarget, onExpandNode, autoExpandDelayMs, clearExpandTimer],
  );

  const handleDragLeave = useCallback(
    (nodeId: string, e: React.DragEvent) => {
      e.stopPropagation();
      const currentTarget = e.currentTarget as HTMLElement;
      const related = e.relatedTarget as Node | null;
      if (related && currentTarget.contains(related)) return;

      if (hoveredFolderIdRef.current === nodeId) {
        clearExpandTimer();
      }
      setDropTarget((prev) => (prev?.targetId === nodeId ? null : prev));
    },
    [clearExpandTimer],
  );

  const handleDrop = useCallback(
    (
      node: {
        id: string;
        isFolder: boolean;
      },
      e: React.DragEvent,
    ) => {
      e.preventDefault();
      e.stopPropagation();
      clearExpandTimer();

      const source = draggedItemRef.current;
      if (!source) return;

      if (source.type === "folder" && isDescendantNode(folders, source.id, node.id)) {
        setDropTarget(null);
        return;
      }

      if (source.type === "request" && source.id === node.id) {
        setDropTarget(null);
        return;
      }

      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const offsetY = e.clientY - rect.top;
      const height = rect.height;

      let position: DropPosition;
      if (!node.isFolder) {
        position = offsetY < height / 2 ? "before" : "after";
      } else {
        if (offsetY < height * 0.25) position = "before";
        else if (offsetY > height * 0.75) position = "after";
        else position = "inside";
      }

      onMoveNode(source, node.id, position);

      setDraggedItem(null);
      setDropTarget(null);
    },
    [folders, onMoveNode, clearExpandTimer],
  );

  const handleRootDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (!draggedItemRef.current) return;
    e.dataTransfer.dropEffect = "move";
    setDropTarget((prev) => {
      if (prev?.targetId === "root" && prev?.position === "root") return prev;
      return {
        targetId: "root",
        targetType: "root",
        position: "root",
      };
    });
  }, []);

  const handleRootDragLeave = useCallback((e: React.DragEvent) => {
    e.stopPropagation();
    const currentTarget = e.currentTarget as HTMLElement;
    const related = e.relatedTarget as Node | null;
    if (related && currentTarget.contains(related)) return;
    setDropTarget((prev) => (prev?.targetId === "root" ? null : prev));
  }, []);

  const handleRootDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      clearExpandTimer();
      const source = draggedItemRef.current;
      if (!source) return;
      onMoveNode(source, "root", "root");
      setDraggedItem(null);
      setDropTarget(null);
    },
    [onMoveNode, clearExpandTimer],
  );

  const getDragSourceProps = useCallback(
    (
      nodeId: string,
      type: DragItemType,
      options?: { disabled?: boolean; parentFolderId?: string | null },
    ) => {
      const disabled = Boolean(options?.disabled);
      return {
        draggable: !disabled,
        onDragStart: (e: React.DragEvent) => {
          if (!disabled) {
            handleDragStart(nodeId, type, options?.parentFolderId, e);
          }
        },
        onDragEnd: handleDragEnd,
      };
    },
    [handleDragStart, handleDragEnd],
  );

  const getDropTargetProps = useCallback(
    (options: {
      id: string;
      isFolder: boolean;
      isExpanded?: boolean;
      onExpand?: () => void;
    }) => ({
      onDragOver: (e: React.DragEvent) => handleDragOver(options, e),
      onDragLeave: (e: React.DragEvent) => handleDragLeave(options.id, e),
      onDrop: (e: React.DragEvent) => handleDrop(options, e),
    }),
    [handleDragOver, handleDragLeave, handleDrop],
  );

  const getRootDropTargetProps = useCallback(
    () => ({
      onDragOver: handleRootDragOver,
      onDragLeave: handleRootDragLeave,
      onDrop: handleRootDrop,
    }),
    [handleRootDragOver, handleRootDragLeave, handleRootDrop],
  );

  return {
    draggedItem,
    draggedId: draggedItem?.id ?? null,
    draggedType: draggedItem?.type ?? null,
    dropTarget,
    getDragSourceProps,
    getDropTargetProps,
    getRootDropTargetProps,
  };
}
