import { Plus, Upload } from "lucide-react";
import { useVartaStore } from "../store/vartaStore";
import { useWorkspaceStore } from "../store/workspaceStore";

interface EmptyStateProps {
  isMobile?: boolean;
}

export default function EmptyState({ isMobile = false }: EmptyStateProps) {
  const newTab = useVartaStore((s) => s.newTab);
  const openImportModal = useWorkspaceStore((s) => s.openImportModal);

  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 text-center px-4">
      <div
        className={`flex items-center justify-center rounded-2xl bg-brand-gradient ${
          isMobile ? "h-12 w-12" : "h-16 w-16"
        }`}
      >
        <svg
          width={isMobile ? "24" : "32"}
          height={isMobile ? "24" : "32"}
          viewBox="0 0 24 24"
          fill="none"
        >
          <path d="M4 4h16v11H9l-5 5V4z" fill="white" />
          <circle cx="9" cy="9" r="1.3" fill="#3B82F6" />
          <circle cx="14" cy="9" r="1.3" fill="#3B82F6" />
          <circle cx="14" cy="13" r="1.3" fill="#3B82F6" />
          <line
            x1="9"
            y1="9"
            x2="14"
            y2="9"
            stroke="#3B82F6"
            strokeWidth="0.8"
          />
          <line
            x1="14"
            y1="9"
            x2="14"
            y2="13"
            stroke="#3B82F6"
            strokeWidth="0.8"
          />
        </svg>
      </div>
      <p className={`text-text-secondary ${isMobile ? "text-xs" : "text-sm"}`}>
        Start a conversation with your API
      </p>
      <div
        className={`flex gap-2 ${isMobile ? "flex-col w-full max-w-65" : ""}`}
      >
        <button
          onClick={newTab}
          className={`flex items-center justify-center gap-1.5 rounded-md bg-primary text-sm font-medium text-white hover:bg-primary-hover ${
            isMobile ? "w-full px-4 py-2.5" : "px-4 py-2"
          }`}
        >
          <Plus size={14} />
          Create request
        </button>
        <button
          onClick={() => openImportModal()}
          className={`flex items-center justify-center gap-1.5 rounded-md border border-border text-sm text-text-secondary hover:bg-panel-raised transition-colors cursor-pointer ${
            isMobile ? "w-full px-4 py-2.5" : "px-4 py-2"
          }`}
        >
          <Upload size={14} />
          Import collection
        </button>
      </div>
    </div>
  );
}
