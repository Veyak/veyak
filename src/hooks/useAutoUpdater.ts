import { useState, useEffect, useRef, useCallback } from "react";
import { check, DownloadEvent, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";

export interface AutoUpdaterState {
  updateAvailable: boolean;
  isUpdating: boolean;
  progress: number;
  error: string | null;
  currentVersion: string;
  releaseVersion: string;
  changelog: string;
  releaseDate?: string;
  startUpdate: () => Promise<void>;
  dismissUpdate: () => void;
  retryUpdate: () => Promise<void>;
}

export function useAutoUpdater(): AutoUpdaterState {
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [currentVersion, setCurrentVersion] = useState("");
  const [releaseVersion, setReleaseVersion] = useState("");
  const [changelog, setChangelog] = useState("");
  const [releaseDate, setReleaseDate] = useState<string | undefined>(undefined);

  const pendingUpdateRef = useRef<Update | null>(null);

  // Initialize current app version
  useEffect(() => {
    async function loadVersion() {
      try {
        const v = await getVersion();
        setCurrentVersion(v);
      } catch {
        setCurrentVersion("0.5.0");
      }
    }
    loadVersion();
  }, []);

  const startUpdate = useCallback(async () => {
    const update = pendingUpdateRef.current;
    if (!update) {
      // In simulated preview or missing binary mode
      setIsUpdating(true);
      setProgress(10);
      const timer = setInterval(() => {
        setProgress((prev) => {
          if (prev >= 100) {
            clearInterval(timer);
            return 100;
          }
          return prev + 15;
        });
      }, 300);
      return;
    }

    try {
      setIsUpdating(true);
      setError(null);
      setProgress(0);

      let downloaded = 0;
      let contentLength = 0;

      await update.downloadAndInstall((event: DownloadEvent) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength || 0;
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            if (contentLength > 0) {
              setProgress(Math.min((downloaded / contentLength) * 100, 100));
            }
            break;
          case "Finished":
            setProgress(100);
            break;
        }
      });

      await relaunch();
    } catch (err) {
      console.error("Update failed:", err);
      setError(err instanceof Error ? err.message : String(err));
      setIsUpdating(false);
    }
  }, []);

  const dismissUpdate = useCallback(() => {
    setUpdateAvailable(false);
    setError(null);
  }, []);

  const retryUpdate = useCallback(async () => {
    setError(null);
    await startUpdate();
  }, [startUpdate]);

  useEffect(() => {
    async function checkForUpdates() {
      try {
        const update = await check();
        if (!update) return;

        pendingUpdateRef.current = update;

        let curVer = update.currentVersion;
        if (!curVer) {
          try {
            curVer = await getVersion();
          } catch {
            curVer = "0.5.0";
          }
        }

        setCurrentVersion(curVer);
        setReleaseVersion(update.version);
        setChangelog(update.body || "");
        setReleaseDate(update.date);
        setUpdateAvailable(true);
      } catch (err) {
        // In local development or unsigned environments, update check may gracefully fail
        console.warn("Update check completed or failed:", err);
      }
    }

    checkForUpdates();
  }, []);

  // Development simulation helper: window.__simulateUpdate()
  useEffect(() => {
    (window as any).__simulateUpdate = (mockData?: {
      currentVersion?: string;
      version?: string;
      body?: string;
      date?: string;
    }) => {
      setCurrentVersion(mockData?.currentVersion || "0.5.0");
      setReleaseVersion(mockData?.version || "0.6.0");
      setChangelog(
        mockData?.body ||
        "## What's Changed\n* feat: added WebSocket TLS certificate verification config by @iamdhakrey in https://github.com/Veyak/veyak/pull/55\n* feat: migrate environment selector to title bar by @iamdhakrey in https://github.com/Veyak/veyak/pull/57\n* feat: full GraphQL studio support with introspection & subscriptions by @iamdhakrey in https://github.com/Veyak/veyak/pull/59\n* fix(ci): auto-generate release notes and populate latest.json properly by @iamdhakrey in https://github.com/Veyak/veyak/pull/60\n\n**Full Changelog**: https://github.com/Veyak/veyak/compare/v0.5.0...v0.6.0"
      );
      setReleaseDate(mockData?.date || new Date().toISOString());
      setError(null);
      setProgress(0);
      setIsUpdating(false);
      setUpdateAvailable(true);
    };

    return () => {
      delete (window as any).__simulateUpdate;
    };
  }, []);

  return {
    updateAvailable,
    isUpdating,
    progress,
    error,
    currentVersion,
    releaseVersion,
    changelog,
    releaseDate,
    startUpdate,
    dismissUpdate,
    retryUpdate,
  };
}

