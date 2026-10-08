import { useState } from "react";
import { ScriptLanguage } from "@veyak-internal/models";
import CodeEditor from "../../../CodeEditor";
import { Code2, BookOpen, Sparkles, ChevronRight } from "lucide-react";

interface ScriptsTabProps {
  preRequestScript?: string;
  preRequestLanguage?: ScriptLanguage;
  postRequestScript?: string;
  postRequestLanguage?: ScriptLanguage;
  onChange: (updates: {
    preRequestScript?: string;
    preRequestLanguage?: ScriptLanguage;
    postRequestScript?: string;
    postRequestLanguage?: ScriptLanguage;
  }) => void;
  isMobile?: boolean;
}

type ScriptType = "pre" | "post";

const LANGUAGES: { id: ScriptLanguage; label: string; editorLang: string }[] = [
  { id: "javascript", label: "JavaScript (QuickJS)", editorLang: "javascript" },
  { id: "python", label: "Python 3", editorLang: "python" },
  { id: "go", label: "Go", editorLang: "go" },
  { id: "rust", label: "Rust", editorLang: "rust" },
];

interface Snippet {
  title: string;
  description: string;
  code: Record<ScriptLanguage, string>;
  forType?: ScriptType;
}

const SNIPPETS: Snippet[] = [
  {
    title: "Set environment variable",
    description: "Store a key-value pair in the active environment",
    code: {
      javascript: `// Save variable to active environment
veyak.environment.set("my_variable", "my_value");`,
      python: `# Save variable to active environment
veyak.environment.set("my_variable", "my_value")`,
      go: `// Save variable to active environment
v.Environment.Set("my_variable", "my_value")`,
      rust: `// Save variable to active environment
v.set_env("my_variable", "my_value");`,
    },
  },
  {
    title: "Get environment variable",
    description: "Read a variable from the active environment",
    code: {
      javascript: `// Read variable from active environment
const token = veyak.environment.get("auth_token");
console.log("Active token:", token);`,
      python: `# Read variable from active environment
token = veyak.environment.get("auth_token")
print("Active token:", token)`,
      go: `// Read variable from active environment
token := v.Environment.Get("auth_token")
v.Log("Active token: " + token)`,
      rust: `// Read variable from active environment
let token = v.get_env("auth_token");
v.log(&format!("Active token: {}", token));`,
    },
  },
  {
    title: "Extract response & save token",
    description: "Parse response JSON and save auth token to environment",
    forType: "post",
    code: {
      javascript: `// Extract access_token from response JSON and save to environment
const json = veyak.response.json();
if (json && json.token) {
    veyak.environment.set("auth_token", json.token);
    console.log("Saved auth_token to environment");
}`,
      python: `# Extract access_token from response JSON and save to environment
data = veyak.response.json()
if "token" in data:
    veyak.environment.set("auth_token", data["token"])
    print("Saved auth_token to environment")`,
      go: `// Extract token from JSON response and save
json := v.Response.JSON()
if token, ok := json["token"].(string); ok {
    v.Environment.Set("auth_token", token)
    v.Log("Saved auth_token to environment")
}`,
      rust: `// Extract token from response JSON and save
if let Some(resp) = &v.response {
    if let Some(token) = resp.get("token").and_then(|t| t.as_str()) {
        v.set_env("auth_token", token);
        v.log("Saved auth_token to environment");
    }
}`,
    },
  },
  {
    title: "Test: Status code is 200",
    description: "Assert that the response status code is 200 OK",
    forType: "post",
    code: {
      javascript: `veyak.test("Status code is 200", () => {
    veyak.response.to.have.status(200);
});`,
      python: `veyak.test("Status code is 200", lambda: veyak.response.status == 200)`,
      go: `v.Test("Status code is 200", v.Response != nil && v.Response.Status == 200)`,
      rust: `v.test("Status code is 200", v.response.as_ref().map(|r| r["status"] == 200).unwrap_or(false));`,
    },
  },
  {
    title: "Test: Response contains string",
    description: "Assert that the body text includes expected substring",
    forType: "post",
    code: {
      javascript: `veyak.test("Body contains success", () => {
    veyak.expect(veyak.response.text()).to.include("success");
});`,
      python: `veyak.test("Body contains success", lambda: "success" in veyak.response.text())`,
      go: `v.Test("Body contains success", v.Response != nil && strings.Contains(v.Response.Body, "success"))`,
      rust: `v.test("Body contains success", v.response.as_ref().map(|r| r["body"].as_str().unwrap_or("").contains("success")).unwrap_or(false));`,
    },
  },
  {
    title: "Log custom message",
    description: "Print a message to the script console output",
    code: {
      javascript: `console.log("Pre-request script executed at:", new Date().toISOString());`,
      python: `print("Python script running with request URL:", veyak.request.get("url") if veyak.request else "N/A")`,
      go: `v.Log("Executing Go script for request")`,
      rust: `v.log("Executing Rust script hook");`,
    },
  },
];

export default function ScriptsTab({
  preRequestScript = "",
  preRequestLanguage = "javascript",
  postRequestScript = "",
  postRequestLanguage = "javascript",
  onChange,
  isMobile = false,
}: ScriptsTabProps) {
  const [activeType, setActiveType] = useState<ScriptType>("pre");
  const [snippetsOpen, setSnippetsOpen] = useState(!isMobile);

  const currentScript = activeType === "pre" ? preRequestScript : postRequestScript;
  const currentLanguage = activeType === "pre" ? preRequestLanguage : postRequestLanguage;

  const currentEditorLang =
    LANGUAGES.find((l) => l.id === currentLanguage)?.editorLang || "javascript";

  function handleScriptChange(code: string) {
    if (activeType === "pre") {
      onChange({ preRequestScript: code });
    } else {
      onChange({ postRequestScript: code });
    }
  }

  function handleLanguageChange(lang: ScriptLanguage) {
    if (activeType === "pre") {
      onChange({ preRequestLanguage: lang });
    } else {
      onChange({ postRequestLanguage: lang });
    }
  }

  function insertSnippet(snippet: Snippet) {
    const codeToInsert = snippet.code[currentLanguage] || snippet.code.javascript;
    const separator = currentScript.trim().length > 0 ? "\n\n" : "";
    handleScriptChange(currentScript + separator + codeToInsert);
  }

  const filteredSnippets = SNIPPETS.filter(
    (s) => !s.forType || s.forType === activeType
  );

  return (
    <div className="flex h-full flex-col min-h-0 min-w-0 bg-bg">
      {/* Top Controls Bar */}
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-border bg-panel/30 px-3 py-2 shrink-0">
        {/* Sub-type Switch: Pre-request vs Post-request */}
        <div className="flex items-center gap-1 rounded-md bg-bg/80 p-0.5 border border-border">
          <button
            type="button"
            onClick={() => setActiveType("pre")}
            className={`px-3 py-1 rounded text-xs font-medium transition-all ${
              activeType === "pre"
                ? "bg-primary/20 text-primary font-semibold shadow-xs"
                : "text-text-muted hover:text-text-primary"
            }`}
          >
            Pre-request
          </button>
          <button
            type="button"
            onClick={() => setActiveType("post")}
            className={`px-3 py-1 rounded text-xs font-medium transition-all ${
              activeType === "post"
                ? "bg-primary/20 text-primary font-semibold shadow-xs"
                : "text-text-muted hover:text-text-primary"
            }`}
          >
            Post-request / Tests
          </button>
        </div>

        <div className="flex items-center gap-2">
          {/* Language Selector */}
          <div className="flex items-center gap-1.5 text-xs text-text-muted">
            <Code2 size={14} className="text-primary/70" />
            <span className="hidden sm:inline">Language:</span>
            <select
              value={currentLanguage}
              onChange={(e) => handleLanguageChange(e.target.value as ScriptLanguage)}
              className="rounded border border-border bg-panel px-2 py-1 text-xs text-text-primary focus:border-primary focus:outline-none"
            >
              {LANGUAGES.map((lang) => (
                <option key={lang.id} value={lang.id}>
                  {lang.label}
                </option>
              ))}
            </select>
          </div>

          {/* Toggle Snippets Button */}
          <button
            type="button"
            onClick={() => setSnippetsOpen(!snippetsOpen)}
            className={`flex items-center gap-1 rounded border border-border px-2.5 py-1 text-xs transition-colors ${
              snippetsOpen
                ? "bg-primary/10 text-primary border-primary/30"
                : "bg-panel text-text-muted hover:text-text-primary"
            }`}
            title="Toggle code snippets"
          >
            <Sparkles size={13} />
            <span className="hidden sm:inline">Snippets</span>
          </button>
        </div>
      </div>

      {/* Main Workspace Area: Editor + Snippets sidebar */}
      <div className="flex flex-1 min-h-0 min-w-0 overflow-hidden">
        {/* Code Editor */}
        <div className="flex-1 flex flex-col min-h-0 min-w-0 overflow-hidden relative">
          <div className="flex-1 min-h-0 overflow-hidden">
            <CodeEditor
              language={currentEditorLang}
              value={currentScript}
              onChange={handleScriptChange}
              placeholder={
                activeType === "pre"
                  ? `// ${currentLanguage === "javascript" ? "JavaScript" : currentLanguage} Pre-request Script\n// Runs before sending the request. Set variables or prepare headers.\n\nveyak.environment.set("timestamp", String(Date.now()));`
                  : `// ${currentLanguage === "javascript" ? "JavaScript" : currentLanguage} Post-request Script\n// Runs after response arrives. Extract data to environment or assert tests.\n\nconst data = veyak.response.json();\nveyak.environment.set("auth_token", data.token);\nveyak.test("Status 200", () => veyak.response.to.have.status(200));`
              }
              lineNumbers
              className="h-full"
            />
          </div>

          {/* Quick status footer */}
          <div className="border-t border-border/60 bg-panel/20 px-3 py-1 text-[11px] text-text-muted flex items-center justify-between">
            <span>
              Global API: <code className="text-primary font-mono">veyak</code> (or <code className="text-primary font-mono">vy</code>)
            </span>
            <span>
              {currentScript.trim().length > 0 ? (
                <span className="text-emerald-500 font-medium">● Script Active</span>
              ) : (
                <span className="text-text-muted">No script</span>
              )}
            </span>
          </div>
        </div>

        {/* Snippets Panel (Right side) */}
        {snippetsOpen && (
          <div className="w-64 sm:w-72 border-l border-border bg-panel/40 flex flex-col shrink-0 min-h-0 overflow-hidden">
            <div className="flex items-center gap-1.5 border-b border-border px-3 py-2 text-xs font-semibold text-text-primary bg-panel/60">
              <BookOpen size={14} className="text-primary" />
              <span>Snippets Library</span>
            </div>
            <div className="flex-1 overflow-y-auto p-2 space-y-1.5 scrollbar-thin">
              {filteredSnippets.map((snippet, idx) => (
                <button
                  key={idx}
                  type="button"
                  onClick={() => insertSnippet(snippet)}
                  className="group w-full text-left p-2 rounded-md border border-border/60 bg-panel/50 hover:bg-primary/10 hover:border-primary/40 transition-all cursor-pointer"
                >
                  <div className="flex items-center justify-between text-xs font-medium text-text-primary group-hover:text-primary">
                    <span>{snippet.title}</span>
                    <ChevronRight size={13} className="text-text-muted group-hover:text-primary transition-transform group-hover:translate-x-0.5" />
                  </div>
                  <p className="text-[11px] text-text-muted mt-0.5 line-clamp-2">
                    {snippet.description}
                  </p>
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
