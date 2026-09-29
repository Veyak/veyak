import React, { useState, useEffect, useRef } from "react";
import { FolderNode as FolderNodeType } from "@veyak-internal/models";
import { RequestItem } from "./RequestItem";
import {
  ChevronRight,
  Folder,
  FolderOpen,
  Trash2,
  Edit2,
  MoreHorizontal,
  Check,
  X,
} from "lucide-react";
import * as Icons from "lucide-react";
import { useWorkspaceStore } from "../../store/workspaceStore";
import { useTreeDragDropContext } from "../../hooks/useTreeDragDrop";

export const FolderNodeItem: React.FC<{
  node: FolderNodeType;
  level?: number;
}> = ({ node, level = 0 }) => {
  const [isOpen, setIsOpen] = useState(false);

  // Rename States
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editValue, setEditValue] = useState("");

  // Dropdown & Creation States
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [addingItem, setAddingItem] = useState<
    "folder" | "request" | "ws" | "grpc" | "graphql" | null
  >(null);
  const [newItemName, setNewItemName] = useState("");

  const menuRef = useRef<HTMLDivElement>(null);

  const dnd = useTreeDragDropContext();
  const isDragging = dnd?.draggedId === node.folder.id;
  const isDropTargetInside =
    dnd?.dropTarget?.targetId === node.folder.id &&
    dnd.dropTarget.position === "inside";
  const isDropTargetBefore =
    dnd?.dropTarget?.targetId === node.folder.id &&
    dnd.dropTarget.position === "before";
  const isDropTargetAfter =
    dnd?.dropTarget?.targetId === node.folder.id &&
    dnd.dropTarget.position === "after";

  const dragProps = dnd
    ? dnd.getDragSourceProps(node.folder.id, "folder", {
        disabled: Boolean(editingId),
        parentFolderId: node.folder.parentFolderId,
      })
    : { draggable: false };

  const dropProps = dnd
    ? dnd.getDropTargetProps({
        id: node.folder.id,
        isFolder: true,
        isExpanded: isOpen,
        onExpand: () => setIsOpen(true),
      })
    : {};

  // Make sure 'renameFolder' is exported from your store!
  const { createFolder, createRequest, deleteFolder, renameFolder, createWs, additionTypes } =
    useWorkspaceStore();

  // Handle click outside for the dropdown menu
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const handleCreateItem = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newItemName.trim() || !addingItem) return;

    if (addingItem === "folder") {
      await createFolder(node.folder.collectionId, node.folder.id, newItemName);
    } else if (addingItem === "ws") {
      await createWs(node.folder.collectionId, node.folder.id, newItemName);
    } else if (addingItem === "grpc") {
      await createRequest(
        node.folder.collectionId,
        node.folder.id,
        newItemName,
        "GRPC",
      );
    } else if (addingItem === "graphql") {
      await createRequest(
        node.folder.collectionId,
        node.folder.id,
        newItemName,
        "GRAPHQL",
      );
    } else {
      await createRequest(
        node.folder.collectionId,
        node.folder.id,
        newItemName,
        "REST",
      );
    }

    setNewItemName("");
    setAddingItem(null);
    setIsOpen(true); // Ensure the folder opens so the user sees their new item
  };

  const handleRenameSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (editValue.trim() && editValue !== node.folder.name) {
      console.log(
        `Renaming folder${node.folder.collectionId} ${node.folder.id} to "${editValue}"`,
      );
      await renameFolder(node.folder.collectionId, node.folder.id, editValue);
    }
    setEditingId(null);
  };

  return (
    <div className="flex flex-col relative">
      {/* Folder Row */}
      <div
        {...dragProps}
        {...dropProps}
        className={`group flex items-center justify-between px-1.5 py-1.5 mx-1 rounded-md text-sm cursor-pointer transition-all relative select-none ${
          isDropTargetInside
            ? "bg-primary/15 ring-1 ring-primary text-primary font-medium shadow-sm"
            : "text-text-secondary hover:bg-panel hover:text-text-primary"
        } ${isDragging ? "opacity-40 scale-[0.99]" : ""}`}
        style={{ paddingLeft: `${level * 12 + 6}px` }}
        onClick={() => {
          // Only toggle open/close if we aren't currently renaming this folder
          if (editingId !== node.folder.id) setIsOpen(!isOpen);
        }}
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
        {editingId === node.folder.id ? (
          // --- INLINE RENAME FORM ---
          <form
            onSubmit={handleRenameSubmit}
            className="flex items-center gap-1 w-full pr-1"
          >
            <Folder className="w-3.5 h-3.5 text-text-muted shrink-0" />
            <input
              autoFocus
              type="text"
              value={editValue}
              onChange={(e) => setEditValue(e.target.value)}
              onClick={(e) => e.stopPropagation()} // Prevent toggling folder open/close
              className="input-shell w-full py-0.5 px-2 text-xs"
            />
            <button
              type="submit"
              className="p-1 hover:text-success cursor-pointer"
              onClick={(e) => e.stopPropagation()}
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
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </form>
        ) : (
          // --- STANDARD FOLDER DISPLAY ---
          <>
            <div className="flex items-center gap-1.5 truncate">
              <ChevronRight
                className={`w-3.5 h-3.5 shrink-0 transition-transform ${isOpen ? "rotate-90" : ""}`}
              />
              {isOpen ? (
                <FolderOpen className="w-3.5 h-3.5 text-secondary shrink-0" />
              ) : (
                <Folder className="w-3.5 h-3.5 text-text-muted shrink-0" />
              )}
              <span className="truncate">{node.folder.name}</span>
            </div>

            {/* Hover Actions (Options Dropdown) */}
            <div
              className="opacity-70 group-hover:opacity-100 flex items-center transition-opacity pr-1"
              ref={menuRef}
            >
              <div className="relative">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setIsMenuOpen(!isMenuOpen);
                  }}
                  className="p-1 hover:text-text-primary hover:bg-borderMuted rounded transition-colors"
                  title="Options"
                >
                  <MoreHorizontal className="w-3.5 h-3.5" />
                </button>

                {/* Dropdown Menu */}
                {isMenuOpen && (
                  <div
                    onClick={(e) => e.stopPropagation()} // Prevent folder from toggling when clicking inside menu
                    className="absolute right-0 top-full mt-1 w-45 py-1 z-50 bg-panel-raised border border-border shadow-elevated rounded-md animate-in fade-in zoom-in-95 duration-100"
                  >
                    {additionTypes.map((type) => {
                      const IconComp = (Icons as any)[type.icon] || Icons.FilePlus;
                      return (
                        <button
                          key={type.id}
                          onClick={() => {
                            setAddingItem(type.id as any);
                            setIsMenuOpen(false);
                            setIsOpen(true);
                          }}
                          className="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-text-secondary hover:bg-panel hover:text-text-primary transition-colors cursor-pointer"
                        >
                          <IconComp className="w-3.5 h-3.5" />
                          {type.label}
                        </button>
                      );
                    })}

                    <div className="h-px bg-border my-1 mx-2" />

                    <button
                      onClick={() => {
                        setEditingId(node.folder.id);
                        setEditValue(node.folder.name);
                        setIsMenuOpen(false);
                      }}
                      className="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-text-secondary hover:bg-panel hover:text-text-primary transition-colors cursor-pointer"
                    >
                      <Edit2 className="w-3.5 h-3.5" />
                      Rename
                    </button>
                    <button
                      onClick={() => {
                        deleteFolder(node.folder.id);
                        setIsMenuOpen(false);
                      }}
                      className="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-error hover:bg-error/10 transition-colors cursor-pointer"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                      Delete
                    </button>
                  </div>
                )}
              </div>
            </div>
          </>
        )}
      </div>

      {/* Children & Requests rendering */}
      {isOpen && (
        <div className="flex flex-col relative before:content-[''] before:absolute before:left-3.5 before:top-0 before:bottom-0 before:w-px before:bg-borderMuted ml-2.5">
          {/* Inline Add Item Input */}
          {addingItem && (
            <form
              onSubmit={handleCreateItem}
              className="pl-6 pr-2 py-1 flex gap-1 mt-0.5"
            >
              <input
                autoFocus
                type="text"
                placeholder={`New ${addingItem} name...`}
                value={newItemName}
                onChange={(e) => setNewItemName(e.target.value)}
                onBlur={() => {
                  if (!newItemName.trim()) setAddingItem(null);
                }}
                className="input-shell w-full py-0.5 px-2 text-xs"
              />
            </form>
          )}

          {/* Render Subfolders */}
          {node.children.map((child) => (
            <FolderNodeItem
              key={child.folder.id}
              node={child}
              level={level + 1}
            />
          ))}

          {/* Render Requests inside this folder */}
          {node.requests.map((req) => (
            <div key={req.id} style={{ paddingLeft: `${(level + 1) * 12}px` }}>
              <RequestItem request={req} />
            </div>
          ))}

          {node.children.length === 0 &&
            node.requests.length === 0 &&
            !addingItem && (
              <div className="pl-8 py-1.5 text-xs text-text-muted italic">
                Empty
              </div>
            )}
        </div>
      )}
    </div>
  );
};
