import { Settings, X } from "lucide-react";
import { CollectionsTree } from "./CollectionTree";
import { useSettingsStore } from "../../store/settingStore";
import { useState, useCallback, useRef, useEffect } from "react";

const DEFAULT_SIDEBAR_WIDTH = 260;
const MIN_SIDEBAR_WIDTH = 180;
const MAX_SIDEBAR_WIDTH = 500;
const STORAGE_KEY = "veyak_sidebar_width";

interface SidebarProps {
  isMobile: boolean;
  onClose?: () => void;
  width?: number;
}

export default function Sidebar({ isMobile, onClose, width }: SidebarProps) {
  const setSettingsOpen = useSettingsStore((s) => s.setSettingsOpen);

  const [sidebarWidth, setSidebarWidth] = useState<number>(() => {
    if (typeof width === "number") return width;
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const parsed = Number(saved);
        if (
          !isNaN(parsed) &&
          parsed >= MIN_SIDEBAR_WIDTH &&
          parsed <= MAX_SIDEBAR_WIDTH
        ) {
          return parsed;
        }
      }
    } catch (err) {
      console.warn("Failed to read sidebar width from localStorage", err);
    }
    return DEFAULT_SIDEBAR_WIDTH;
  });

  const [isDragging, setIsDragging] = useState(false);
  const startXRef = useRef(0);
  const startWidthRef = useRef(DEFAULT_SIDEBAR_WIDTH);

  useEffect(() => {
    if (typeof width === "number") {
      setSidebarWidth(width);
    }
  }, [width]);

  useEffect(() => {
    return () => {
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
  }, []);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      setIsDragging(true);
      startXRef.current = e.clientX;
      startWidthRef.current = sidebarWidth;

      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";

      const onMouseMove = (ev: MouseEvent) => {
        const delta = ev.clientX - startXRef.current;
        const newWidth = Math.min(
          MAX_SIDEBAR_WIDTH,
          Math.max(MIN_SIDEBAR_WIDTH, startWidthRef.current + delta)
        );
        setSidebarWidth(newWidth);
        try {
          localStorage.setItem(STORAGE_KEY, String(newWidth));
        } catch (err) {
          console.warn("Failed to persist sidebar width", err);
        }
      };

      const onMouseUp = () => {
        setIsDragging(false);
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
        window.removeEventListener("mousemove", onMouseMove);
        window.removeEventListener("mouseup", onMouseUp);
      };

      window.addEventListener("mousemove", onMouseMove);
      window.addEventListener("mouseup", onMouseUp);
    },
    [sidebarWidth]
  );

  const handleDoubleClick = useCallback(() => {
    setSidebarWidth(DEFAULT_SIDEBAR_WIDTH);
    try {
      localStorage.setItem(STORAGE_KEY, String(DEFAULT_SIDEBAR_WIDTH));
    } catch (err) {
      console.warn("Failed to reset sidebar width", err);
    }
  }, []);

  const handleSettings = () => {
    setSettingsOpen(true);
    if (isMobile && onClose) onClose();
  };

  return (
    <>
      {isDragging && (
        <div className="fixed inset-0 z-50 cursor-col-resize select-none" />
      )}
      <aside
        style={!isMobile ? { width: `${sidebarWidth}px` } : undefined}
        className={`relative flex h-full shrink-0 flex-col border-r border-border bg-bg ${
          isMobile ? "w-[85vw] max-w-[320px]" : ""
        }`}
      >
        {/* Mobile close header */}
        {isMobile && (
          <div className="flex items-center justify-between border-b border-border px-3 py-2.5">
            <span className="text-sm font-semibold bg-brand-gradient bg-clip-text text-transparent">
              Veyak
            </span>
            <button
              onClick={onClose}
              className="rounded-md p-1.5 text-text-secondary hover:bg-panel-raised hover:text-text-primary"
              aria-label="Close sidebar"
            >
              <X size={18} />
            </button>
          </div>
        )}

        {/* Collections tree */}
        <CollectionsTree />

        {/* FOOTER - Stacked Buttons */}
        <div className="flex flex-col gap-1 border-t border-border p-2">
          <button
            onClick={handleSettings}
            className="flex w-full cursor-pointer items-center gap-2 rounded-md px-3 py-2 text-sm font-medium text-text-secondary transition-colors hover:bg-borderMuted hover:text-text-primary"
          >
            <Settings size={16} />
            Settings
          </button>
        </div>

        {/* Resizer Divider between sidebar and main workspace */}
        {!isMobile && (
          <div
            onMouseDown={handleMouseDown}
            onDoubleClick={handleDoubleClick}
            className={`group absolute top-0 bottom-0 -right-1 w-2 cursor-col-resize z-20 flex items-center justify-center select-none ${
              isDragging ? "bg-primary/20" : "hover:bg-primary/10"
            }`}
            title="Drag to resize, double-click to reset (260px)"
          >
            {/* Visual indicator line on hover or drag */}
            <div
              className={`w-0.5 h-full transition-colors ${
                isDragging
                  ? "bg-primary"
                  : "bg-transparent group-hover:bg-primary/70"
              }`}
            />
          </div>
        )}
      </aside>
    </>
  );
}
