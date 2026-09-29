import React, { useEffect, useRef, useState } from "react";
import { MethodStyles } from "../../types";
import { Check, Copy, Edit2, MoreHorizontal, Trash2, X } from "lucide-react";
import { useWorkspaceStore } from "../../store/workspaceStore";
import { useVartaStore } from "../../store/vartaStore";
import { RequestItem as Item } from "@veyak-internal/models";
import { useTreeDragDropContext } from "../../hooks/useTreeDragDrop";

export const RequestItem: React.FC<{ request: Item }> = ({ request }) => {
  const { deleteRequest, renameRequest, cloneRequest } = useWorkspaceStore();
  const activeTabId = useVartaStore((s) => s.activeTabId);
  const closeTab = useVartaStore((s) => s.closeTab);
  const openRequestTab = useVartaStore((s) => s.openRequest);

  const [editingId, setEditingId] = useState<string | null>(null);
  const [editValue, setEditValue] = useState("");
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const menuRef = useRef<HTMLDivElement>(null);

  const dnd = useTreeDragDropContext();
  const isDragging = dnd?.draggedId === request.id;
  const isDropTargetBefore =
    dnd?.dropTarget?.targetId === request.id &&
    dnd.dropTarget.position === "before";
  const isDropTargetAfter =
    dnd?.dropTarget?.targetId === request.id &&
    dnd.dropTarget.position === "after";

  const dragProps = dnd
    ? dnd.getDragSourceProps(request.id, "request", {
        disabled: Boolean(editingId),
        parentFolderId: request.folderId,
      })
    : { draggable: false };

  const dropProps = dnd
    ? dnd.getDropTargetProps({
        id: request.id,
        isFolder: false,
      })
    : {};

  const handleRenameSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (editValue.trim() && editValue !== request.name) {
      console.log(`Renaming request ${request.id} to "${editValue}"`);
      await renameRequest(request.id, editValue.trim());
      useVartaStore.setState((s) => {
        const updatedTabs = s.tabs.map((t) =>
          t.request.id === request.id
            ? { ...t, request: { ...t.request, name: editValue.trim() } }
            : t
        );
        const active = updatedTabs.find((t) => t.id === s.activeTabId) ?? null;
        return { tabs: updatedTabs, activeTab: active };
      });
    }
    setEditingId(null);
  };

  const handleDeleteRequest = async (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsMenuOpen(false);
    await deleteRequest(request.id);
    closeTab(request.id);
  };

  const handleCloneRequest = async (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsMenuOpen(false);
    await cloneRequest(request.id);
    closeTab(request.id);
  };

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  return (
    <div className="flex flex-col relative">
      {editingId === request.id ? (
        <div className="flex items-center justify-between px-2 py-1 mx-1 my-0.5 rounded-md text-sm border border-border bg-panel">
          <form
            onSubmit={handleRenameSubmit}
            className="flex items-center gap-2 w-full"
          >
            <span
              className={`text-[10px] font-bold w-10 text-right shrink-0 ${request.type === "grpc"
                ? "text-method-grpc"
                : request.type === "graphql"
                  ? "text-method-graphql"
                  : MethodStyles[request.method as string] || "text-text-muted"
                }`}
            >
              {request.type === "grpc"
                ? "gRPC"
                : request.type === "graphql"
                  ? "GQL"
                  : request.method}
            </span>
            <input
              autoFocus
              type="text"
              value={editValue}
              onChange={(e) => setEditValue(e.target.value)}
              onClick={(e) => e.stopPropagation()}
              onKeyDown={(e) => {
                if (e.key === "Escape") {
                  e.stopPropagation();
                  setEditingId(null);
                }
              }}
              className="input-shell w-full py-0.5 px-2 text-xs"
            />
            <button
              type="submit"
              className="p-1 hover:text-success cursor-pointer"
              onClick={(e) => e.stopPropagation()}
              title="Save"
            >
              <Check className="w-3.5 h-3.5" />
            </button>
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                setEditingId(null);
              }}
              className="p-1 hover:text-error cursor-pointer"
              title="Cancel"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </form>
        </div>
      ) : (
        <div
          {...dragProps}
          {...dropProps}
          // Trigger the open action when the row is clicked
          onClick={() => openRequestTab(request)}
          className={`group flex items-center justify-between px-2 py-1.5 mx-1 my-0.5 rounded-md text-sm cursor-pointer hover:bg-panel hover:text-text-primary text-text-secondary transition-all relative select-none ${activeTabId === request.id
            ? "border-2 border-primary/10 bg-panel/40"
            : "border border-transparent"
            } ${isDragging ? "opacity-40 scale-[0.99]" : ""}`}
        >
          {/* Top drop insertion line indicator */}
          {isDropTargetBefore && (
            <div className="absolute -top-[2px] left-1 right-1 h-[2px] bg-primary rounded-full z-20 pointer-events-none shadow-[0_0_6px_var(--color-primary)] animate-in fade-in duration-75">
              <div className="absolute -left-1 -top-[3px] w-2 h-2 rounded-full bg-primary" />
            </div>
          )}

          {/* Bottom drop insertion line indicator */}
          {isDropTargetAfter && (
            <div className="absolute -bottom-[2px] left-1 right-1 h-[2px] bg-primary rounded-full z-20 pointer-events-none shadow-[0_0_6px_var(--color-primary)] animate-in fade-in duration-75">
              <div className="absolute -left-1 -top-[3px] w-2 h-2 rounded-full bg-primary" />
            </div>
          )}
          <div className="flex items-center gap-2.5 truncate transition-colors">
            <span
              className={`text-[10px] font-bold w-10 text-right shrink-0 ${request.type === "grpc"
                ? "text-method-grpc"
                : request.type === "graphql"
                  ? "text-method-graphql"
                  : MethodStyles[request.method as string] || "text-text-muted"
                }`}
            >
              {request.type === "grpc"
                ? "gRPC"
                : request.type === "graphql"
                  ? "GQL"
                  : request.method}
            </span>
            <span className="truncate">{request.name}</span>
          </div>

          {/* Hover Actions (Options Dropdown) */}
          <div
            ref={menuRef}
            className={`flex items-center transition-opacity pr-1 ${isMenuOpen ? "opacity-100" : "opacity-70 group-hover:opacity-100"
              }`}
          >
            <div className="relative">
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  setIsMenuOpen(!isMenuOpen);
                }}
                className="p-1 hover:text-text-primary hover:bg-borderMuted rounded transition-colors cursor-pointer"
                title="Options"
              >
                <MoreHorizontal className="w-3.5 h-3.5" />
              </button>

              {/* Dropdown Menu */}
              {isMenuOpen && (
                <div
                  onClick={(e) => e.stopPropagation()} // Prevent opening request tab when clicking menu surface
                  className="absolute right-0 top-full mt-1 w-45 py-1 z-50 bg-panel-raised border border-border shadow-elevated rounded-md animate-in fade-in zoom-in-95 duration-100"
                >
                  <button
                    onClick={handleCloneRequest}
                    className="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-text-secondary hover:bg-panel hover:text-text-primary transition-colors cursor-pointer"
                  >
                    <Copy className="w-3.5 h-3.5" />
                    Duplicate
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setEditingId(request.id);
                      setEditValue(request.name);
                      setIsMenuOpen(false);
                    }}
                    className="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-text-secondary hover:bg-panel hover:text-text-primary transition-colors cursor-pointer"
                  >
                    <Edit2 className="w-3.5 h-3.5" />
                    Rename
                  </button>
                  <div className="h-px bg-border my-1 mx-2" />
                  <button
                    onClick={handleDeleteRequest}
                    className="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-error hover:bg-error/10 transition-colors cursor-pointer"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                    Delete
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      )
      }
    </div >
  );
};
