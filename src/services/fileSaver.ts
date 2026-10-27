import { save as saveFileDialog } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

export interface SaveFileOptions {
  content: string;
  defaultPath: string;
  filters?: { name: string; extensions: string[] }[];
  mimeType?: string;
}

/**
 * Cross-platform file saving utility.
 * In Tauri desktop mode, opens the native OS Save File dialog and writes directly to disk via Rust backend.
 * Falls back to browser link download if running in web/dev without Tauri desktop plugins.
 */
export async function saveTextFile(options: SaveFileOptions): Promise<boolean> {
  const { content, defaultPath, filters, mimeType = "application/json" } = options;
  if (!content) return false;

  // 1. Try native desktop save dialog & Tauri backend write
  try {
    const selectedPath = await saveFileDialog({
      defaultPath,
      filters: filters ?? [
        { name: "JSON", extensions: ["json"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });

    if (!selectedPath) {
      // User cancelled the save dialog
      return false;
    }

    await invoke("save_file_to_disk", {
      filePath: selectedPath,
      content,
    });
    return true;
  } catch (nativeErr) {
    console.warn(
      "Native file save dialog/command unavailable or failed, attempting browser download fallback:",
      nativeErr,
    );
  }

  // 2. Fallback: Browser DOM anchor download (with proper DOM attachment & delayed URL cleanup)
  try {
    const blob = new Blob([content], { type: mimeType });
    const fileUrl = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = fileUrl;
    link.download = defaultPath.split(/[/\\]/).pop() || defaultPath;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    setTimeout(() => URL.revokeObjectURL(fileUrl), 60000);
    return true;
  } catch (browserErr) {
    console.error("Browser download fallback failed:", browserErr);
    return false;
  }
}

