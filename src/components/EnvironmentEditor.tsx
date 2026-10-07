import React, { useEffect, useState } from "react";
import { Plus, Trash2, Edit2, Check, X, Save, Eye, EyeOff } from "lucide-react";
import { useWorkspaceStore } from "../store/workspaceStore";
import { useVartaStore } from "../store/vartaStore";
import { EnvironmentVariable } from "@veyak-internal/models";

interface EnvironmentEditorProps {
  activeCollectionId?: string;
  activeWorkspaceId?: string;
}

export const EnvironmentEditor: React.FC<EnvironmentEditorProps> = ({
  activeCollectionId: propCollectionId,
  activeWorkspaceId: _activeWorkspaceId,
}) => {
  const storeCollectionId = useWorkspaceStore((s) => s.activeCollectionId);
  const collections = useWorkspaceStore((s) => s.collections);
  const activeCollectionId = propCollectionId || storeCollectionId;

  const currentCollection = collections.find(
    (c) => c.id === activeCollectionId,
  );

  const {
    environments,
    activeEnvironmentId,
    fetchEnvironments,
    createEnvironment,
    renameEnvironment,
    deleteEnvironment,
    saveVariables,
    setActiveEnvironment,
  } = useWorkspaceStore();

  const closeEnvEditor = useVartaStore((s) => s.closeEnvEditor);
  const [isCreating, setIsCreating] = useState(false);
  const [newEnvName, setNewEnvName] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");

  // Buffered state for variables to prevent DB thrashing on every keystroke
  const [localVariables, setLocalVariables] = useState<EnvironmentVariable[]>(
    [],
  );
  const [isDirty, setIsDirty] = useState(false);

  const activeEnv = environments.find(
    (e) => e.environment.id === activeEnvironmentId,
  );

  // Sync active environment variables to local buffer when selection changes
  useEffect(() => {
    if (activeEnv) {
      setLocalVariables(activeEnv.variables || []);
      setIsDirty(false);
    }
  }, [activeEnv?.environment.id]);

  useEffect(() => {
    if (activeCollectionId) {
      fetchEnvironments(activeCollectionId);
    }
  }, [activeCollectionId, fetchEnvironments]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (newEnvName.trim() && activeCollectionId) {
      await createEnvironment(activeCollectionId, newEnvName);
      setNewEnvName("");
      setIsCreating(false);
    }
  };

  const handleRename = async (e: React.FormEvent) => {
    e.preventDefault();
    if (editingId && editName.trim()) {
      await renameEnvironment(editingId, editName);
      setEditingId(null);
    }
  };

  const handleSaveVariables = async () => {
    if (activeEnv && isDirty) {
      // Filter out entirely empty rows before saving
      const cleanedVars = localVariables.filter(
        (v) => v.key.trim() !== "" || v.value.trim() !== "",
      );
      await saveVariables(activeEnv.environment.id, cleanedVars);
      setLocalVariables(cleanedVars);
      setIsDirty(false);
    }
  };

  const updateVar = (
    id: string,
    field: keyof EnvironmentVariable,
    value: any,
  ) => {
    setLocalVariables((prev) =>
      prev.map((v) => (v.id === id ? { ...v, [field]: value } : v)),
    );
    setIsDirty(true);
  };

  const addVarRow = () => {
    setLocalVariables([
      ...localVariables,
      {
        id: crypto.randomUUID(),
        environmentid: activeEnv?.environment.id || "",
        key: "",
        value: "",
        enabled: true,
        isSecret: false,
      },
    ]);
  };

  const removeVarRow = (id: string) => {
    setLocalVariables((prev) => prev.filter((v) => v.id !== id));
    setIsDirty(true);
  };

  return (
    <div className="flex h-full w-full bg-bg border border-border rounded-md overflow-hidden">
      {/* LEFT PANE: Environment List */}
      <div className="w-1/3 min-w-62.5 border-r border-border bg-panel flex flex-col">
        <div className="flex items-center justify-between p-3 border-b border-border">
          <div className="flex flex-col min-w-0 pr-2">
            <span className="text-[11px] font-semibold tracking-wider text-text-muted uppercase">
              Environments
            </span>
            {currentCollection && (
              <span className="text-xs font-medium text-text-primary truncate">
                {currentCollection.name}
              </span>
            )}
          </div>
          <button
            onClick={() => setIsCreating(true)}
            className="p-1 hover:text-text-primary text-text-muted transition-colors cursor-pointer shrink-0"
            title="Create Environment"
          >
            <Plus size={16} />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-2 flex flex-col gap-1">
          {isCreating && (
            <form onSubmit={handleCreate} className="flex gap-1 mb-2">
              <input
                autoFocus
                className="input-shell w-full text-sm py-1"
                placeholder="New Environment..."
                value={newEnvName}
                onChange={(e) => setNewEnvName(e.target.value)}
                onBlur={() => setIsCreating(false)}
              />
            </form>
          )}

          {environments.map((env) => (
            <div
              key={env.environment.id}
              onClick={() => setActiveEnvironment(env.environment.id)}
              className={`group flex items-center justify-between p-2 rounded-md cursor-pointer transition-colors ${
                activeEnvironmentId === env.environment.id
                  ? "bg-primary/20 text-text-primary"
                  : "text-text-secondary hover:bg-borderMuted"
              }`}
            >
              {editingId === env.environment.id ? (
                <form
                  onSubmit={handleRename}
                  className="flex gap-1 w-full"
                  onClick={(e) => e.stopPropagation()}
                >
                  <input
                    autoFocus
                    className="input-shell w-full text-xs py-0.5"
                    value={editName}
                    onChange={(e) => setEditName(e.target.value)}
                  />
                  <button
                    type="submit"
                    className="text-success hover:bg-success/20 p-1 rounded"
                  >
                    <Check size={14} />
                  </button>
                  <button
                    type="button"
                    onClick={() => setEditingId(null)}
                    className="text-error hover:bg-error/20 p-1 rounded"
                  >
                    <X size={14} />
                  </button>
                </form>
              ) : (
                <>
                  <span className="truncate text-sm font-medium">
                    {env.environment.name}
                  </span>
                  <div className="opacity-70 group-hover:opacity-100 flex gap-1">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setEditingId(env.environment.id);
                        setEditName(env.environment.name);
                      }}
                      className="p-1 text-text-muted hover:text-text-primary rounded"
                    >
                      <Edit2 size={14} />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        deleteEnvironment(env.environment.id);
                      }}
                      className="p-1 text-text-muted hover:text-error rounded"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* RIGHT PANE: Variable Editor */}
      <div className="flex-1 flex flex-col bg-bg">
        {activeEnv ? (
          <>
            <div className="flex items-center justify-between p-3 border-b border-border">
              <h3 className="font-semibold text-text-primary">
                {activeEnv.environment.name} Variables
              </h3>

              <div className="flex items-center gap-2">
                <button
                  onClick={handleSaveVariables}
                  disabled={!isDirty}
                  className={`flex items-center gap-2 px-3 py-1.5 rounded text-sm font-medium transition-all ${
                    isDirty
                      ? "bg-primary text-white hover:bg-primary-hover shadow-panel"
                      : "bg-panel text-text-muted opacity-50 cursor-default"
                  }`}
                >
                  <Save size={16} />
                  Save
                </button>
                <button
                  onClick={closeEnvEditor}
                  className="right-4 top-3 z-10 rounded-md p-1.5 text-error hover:bg-error hover:text-text-primary transition-colors cursor-pointer"
                >
                  <X size={18} />
                </button>
              </div>
            </div>

            <div className="flex-1 overflow-y-auto p-4">
              <div className="flex flex-col border border-border rounded-md overflow-hidden">
                {/* Table Header */}
                <div className="flex bg-panel border-b border-border text-xs font-semibold text-text-muted py-2">
                  <div className="w-10 text-center">En</div>
                  <div className="flex-1 px-2">VARIABLE</div>
                  <div className="flex-1 px-2">VALUE</div>
                  <div className="w-16 text-center">SECRET</div>
                  <div className="w-10"></div>
                </div>

                {/* Table Body */}
                <div className="flex flex-col divide-y divide-borderMuted">
                  {localVariables.map((v) => (
                    <div
                      key={v.id}
                      className="flex items-center py-1 bg-bg hover:bg-panel/50 group"
                    >
                      <div className="w-10 flex justify-center">
                        <input
                          type="checkbox"
                          checked={v.enabled}
                          onChange={(e) =>
                            updateVar(v.id, "enabled", e.target.checked)
                          }
                          className="accent-primary"
                        />
                      </div>
                      <div className="flex-1 px-1">
                        <input
                          className="w-full bg-transparent text-sm text-text-primary outline-none px-2 py-1 placeholder:text-text-muted/50 focus:bg-panel rounded"
                          placeholder="API_KEY"
                          value={v.key}
                          onChange={(e) =>
                            updateVar(v.id, "key", e.target.value)
                          }
                        />
                      </div>
                      <div className="flex-1 px-1 flex items-center">
                        <input
                          type={v.isSecret ? "password" : "text"}
                          className="w-full bg-transparent text-sm text-text-primary outline-none px-2 py-1 placeholder:text-text-muted/50 focus:bg-panel rounded font-mono"
                          placeholder="value"
                          value={v.value}
                          onChange={(e) =>
                            updateVar(v.id, "value", e.target.value)
                          }
                        />
                      </div>
                      <div className="w-16 flex justify-center text-text-muted hover:text-text-primary cursor-pointer">
                        <button
                          onClick={() =>
                            updateVar(v.id, "isSecret", !v.isSecret)
                          }
                        >
                          {v.isSecret ? (
                            <EyeOff size={16} />
                          ) : (
                            <Eye size={16} />
                          )}
                        </button>
                      </div>
                      <div className="w-10 flex justify-center">
                        <button
                          onClick={() => removeVarRow(v.id)}
                          className="text-text-muted hover:text-error opacity-70 group-hover:opacity-100 transition-opacity"
                        >
                          <Trash2 size={16} />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>

                {/* Add Row Button */}
                <button
                  onClick={addVarRow}
                  className="flex items-center justify-center gap-2 py-2 text-sm text-text-muted hover:text-text-primary hover:bg-panel transition-colors"
                >
                  <Plus size={16} /> Add Variable
                </button>
              </div>
            </div>
          </>
        ) : (
          <div className="flex h-full items-center justify-center text-text-muted text-sm">
            Select or create an environment to view variables.
          </div>
        )}
      </div>
    </div>
  );
};
