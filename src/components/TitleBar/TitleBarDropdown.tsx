import React, { useEffect, useRef, useState } from "react";
import {
  ChevronDown,
  Plus,
  Check,
  X,
  Edit2,
  Trash2,
  Loader2,
} from "lucide-react";

export interface DropdownItem {
  id: string;
  name: string;
}

export interface TitleBarDropdownProps<T extends DropdownItem = DropdownItem> {
  // Trigger
  triggerIcon: React.ReactNode;
  triggerLabel: string;
  triggerTitle?: string;
  triggerMaxWidth?: string;
  isLoading?: boolean;

  // Dropdown popup
  headerTitle: string;
  dropdownWidth?: string;
  emptyMessage?: string;

  // Items
  items: T[];
  activeId: string | null;
  onSelect: (id: string) => void;
  renderItemIcon?: (item: T, isActive: boolean) => React.ReactNode;

  // Special "None / Default" item (like "No Environment")
  noneOption?: {
    label: string;
    isSelected: boolean;
    onSelect: () => void;
  };

  // Actions
  onRename?: (id: string, newName: string) => Promise<void> | void;
  onDelete?: (id: string) => Promise<void> | void;
  renderExtraActions?: (item: T, closeDropdown: () => void) => React.ReactNode;

  // Creation
  createButtonLabel?: string;
  createPlaceholder?: string;
  onCreate?: (name: string) => Promise<void> | void;

  // Footer Actions (e.g. Import / Export)
  renderFooter?: (closeDropdown: () => void) => React.ReactNode;

  // Controlled Open State
  isOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export function TitleBarDropdown<T extends DropdownItem = DropdownItem>({
  triggerIcon,
  triggerLabel,
  triggerTitle,
  triggerMaxWidth = "max-w-[120px]",
  isLoading = false,
  headerTitle,
  dropdownWidth = "w-56",
  emptyMessage = "No items found",
  items,
  activeId,
  onSelect,
  renderItemIcon,
  noneOption,
  onRename,
  onDelete,
  renderExtraActions,
  createButtonLabel,
  createPlaceholder,
  onCreate,
  renderFooter,
  isOpen: controlledIsOpen,
  onOpenChange,
}: TitleBarDropdownProps<T>) {
  const [internalIsOpen, setInternalIsOpen] = useState(false);
  const isControlled = controlledIsOpen !== undefined;
  const open = isControlled ? controlledIsOpen : internalIsOpen;

  const [isCreating, setIsCreating] = useState(false);
  const [createInputValue, setCreateInputValue] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editInputValue, setEditInputValue] = useState("");

  const dropdownRef = useRef<HTMLDivElement>(null);

  const setOpen = (nextOpen: boolean) => {
    if (!nextOpen) {
      setIsCreating(false);
      setEditingId(null);
    }
    if (isControlled) {
      onOpenChange?.(nextOpen);
    } else {
      setInternalIsOpen(nextOpen);
    }
  };

  const handleClose = () => {
    setOpen(false);
  };

  // Click outside listener
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setOpen(false);
      }
    };

    if (open) {
      document.addEventListener("mousedown", handleClickOutside);
      return () =>
        document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [open]);

  const handleCreateSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (createInputValue.trim() && onCreate) {
      await onCreate(createInputValue.trim());
      setCreateInputValue("");
      setIsCreating(false);
    }
  };

  const handleRenameSubmit = async (e: React.FormEvent, id: string) => {
    e.preventDefault();
    if (editInputValue.trim() && onRename) {
      await onRename(id, editInputValue.trim());
      setEditInputValue("");
      setEditingId(null);
    }
  };

  return (
    <div className="relative" ref={dropdownRef}>
      {/* ── Trigger Button ── */}
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1.5 px-2 py-1 rounded-md text-xs font-medium text-text-secondary hover:text-text-primary hover:bg-panel transition-colors cursor-pointer"
        title={triggerTitle}
      >
        {triggerIcon}
        <span className={`truncate ${triggerMaxWidth}`}>{triggerLabel}</span>
        {isLoading ? (
          <Loader2 className="w-3 h-3 animate-spin text-text-muted shrink-0" />
        ) : (
          <ChevronDown
            className={`w-3 h-3 text-text-muted transition-transform duration-150 ${
              open ? "rotate-180" : ""
            }`}
          />
        )}
      </button>

      {/* ── Dropdown Popover ── */}
      {open && (
        <div
          className={`absolute left-0 top-full mt-1 ${dropdownWidth} z-50 rounded-lg bg-panel-raised border border-border shadow-elevated overflow-hidden animate-in fade-in zoom-in-95 duration-100`}
        >
          {/* Header */}
          <div className="px-2.5 py-1.5 text-[11px] font-semibold text-text-muted border-b border-border/40 uppercase tracking-wider">
            {headerTitle}
          </div>

          {/* List Area */}
          <div className="max-h-56 overflow-y-auto py-1">
            {/* Optional "None" Row */}
            {noneOption && (
              <button
                onClick={() => {
                  noneOption.onSelect();
                  handleClose();
                }}
                className={`group flex items-center gap-2 px-2.5 py-1.5 mx-1 my-0.5 rounded-md text-xs text-left cursor-pointer transition-all duration-150 ${
                  noneOption.isSelected
                    ? "bg-panel/60 text-text-primary font-medium"
                    : "text-text-secondary hover:bg-panel hover:text-text-primary"
                }`}
                style={{ width: "calc(100% - 8px)" }}
              >
                <div className="w-3.5 flex justify-center shrink-0">
                  {noneOption.isSelected && (
                    <Check className="w-3.5 h-3.5 text-primary" />
                  )}
                </div>
                <span className="truncate">{noneOption.label}</span>
              </button>
            )}

            {/* Empty State */}
            {items.length === 0 ? (
              <div className="px-3 py-3 text-xs text-center text-text-muted">
                {emptyMessage}
              </div>
            ) : (
              items.map((item) => {
                const isActive = item.id === activeId;
                const isEditing = editingId === item.id;

                return (
                  <div
                    key={item.id}
                    className={`group flex items-center justify-between px-2.5 py-1.5 mx-1 my-0.5 rounded-md text-xs text-text-secondary hover:bg-panel hover:text-text-primary transition-all duration-150 ${
                      isActive ? "bg-panel/60 text-text-primary font-medium" : ""
                    }`}
                  >
                    {isEditing ? (
                      /* Inline Rename Form */
                      <form
                        onSubmit={(e) => handleRenameSubmit(e, item.id)}
                        className="flex items-center gap-1 w-full"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <input
                          type="text"
                          autoFocus
                          value={editInputValue}
                          onChange={(e) => setEditInputValue(e.target.value)}
                          className="input-shell w-full py-0.5 px-2 text-xs"
                        />
                        <button
                          type="submit"
                          className="p-1 hover:text-success cursor-pointer"
                          title="Save"
                        >
                          <Check className="w-3.5 h-3.5" />
                        </button>
                        <button
                          type="button"
                          onClick={() => setEditingId(null)}
                          className="p-1 hover:text-error cursor-pointer"
                          title="Cancel"
                        >
                          <X className="w-3.5 h-3.5" />
                        </button>
                      </form>
                    ) : (
                      /* Normal Item Display */
                      <>
                        <button
                          onClick={() => {
                            onSelect(item.id);
                            handleClose();
                          }}
                          className="flex items-center gap-2 flex-1 text-left truncate cursor-pointer py-0.5"
                        >
                          {isActive ? (
                            <Check className="w-3.5 h-3.5 text-primary shrink-0" />
                          ) : renderItemIcon ? (
                            renderItemIcon(item, false)
                          ) : (
                            triggerIcon
                          )}
                          <span className="truncate">{item.name}</span>
                        </button>

                        {/* Hover Actions */}
                        <div className="opacity-0 group-hover:opacity-100 flex items-center gap-1 transition-opacity">
                          {renderExtraActions &&
                            renderExtraActions(item, handleClose)}

                          {onRename && (
                            <button
                              onClick={(e) => {
                                e.stopPropagation();
                                setEditingId(item.id);
                                setEditInputValue(item.name);
                              }}
                              className="p-1 rounded text-text-muted hover:text-text-primary hover:bg-borderMuted cursor-pointer"
                              title="Rename"
                            >
                              <Edit2 className="w-3 h-3" />
                            </button>
                          )}

                          {onDelete && (
                            <button
                              onClick={(e) => {
                                e.stopPropagation();
                                onDelete(item.id);
                              }}
                              className="p-1 rounded text-text-muted hover:text-error hover:bg-error/10 cursor-pointer"
                              title="Delete"
                            >
                              <Trash2 className="w-3 h-3" />
                            </button>
                          )}
                        </div>
                      </>
                    )}
                  </div>
                );
              })
            )}
          </div>

          {/* Creation Footer */}
          {onCreate && (
            <div className="border-t border-border/40 bg-panel/40 p-1.5">
              {isCreating ? (
                <form
                  onSubmit={handleCreateSubmit}
                  className="flex items-center gap-1"
                >
                  <input
                    type="text"
                    autoFocus
                    placeholder={
                      createPlaceholder ||
                      `New ${headerTitle.toLowerCase()} name...`
                    }
                    value={createInputValue}
                    onChange={(e) => setCreateInputValue(e.target.value)}
                    className="input-shell w-full py-0.5 px-2 text-xs"
                  />
                  <button
                    type="submit"
                    className="p-1 bg-primary hover:bg-primary-hover text-white rounded cursor-pointer"
                    title="Create"
                  >
                    <Check className="w-3.5 h-3.5" />
                  </button>
                  <button
                    type="button"
                    onClick={() => setIsCreating(false)}
                    className="p-1 bg-panel border border-border text-text-secondary hover:text-text-primary rounded cursor-pointer"
                    title="Cancel"
                  >
                    <X className="w-3.5 h-3.5" />
                  </button>
                </form>
              ) : (
                <button
                  onClick={() => {
                    setIsCreating(true);
                    setCreateInputValue("");
                  }}
                  className="flex items-center justify-center gap-1.5 w-full py-1 text-xs text-text-secondary hover:text-text-primary hover:bg-panel rounded border border-dashed border-border transition-colors cursor-pointer"
                >
                  <Plus className="w-3 h-3" />
                  {createButtonLabel || `Create ${headerTitle}`}
                </button>
              )}
            </div>
          )}

          {/* Action Footer (e.g., Import / Export) */}
          {renderFooter && (
            <div className="border-t border-border/40 bg-panel-raised/40 p-1">
              {renderFooter(handleClose)}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
