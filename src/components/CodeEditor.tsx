import { useEffect, useRef, useMemo, useCallback } from "react";
import { CodeJar } from "codejar";
import Prism from "prismjs";
import "prismjs/components/prism-json";
import "prismjs/components/prism-graphql";
import "prismjs/components/prism-javascript";
import "prismjs/components/prism-python";
import "prismjs/components/prism-go";
import "prismjs/components/prism-rust";

import { useSettingsStore, DEFAULT_FONT_SETTINGS } from "../store/settingStore";

export interface CodeEditorProps {
  value?: string;
  onChange?: (value: string) => void;
  language?: "json" | "graphql" | "javascript" | "plaintext" | string;
  readOnly?: boolean;
  lineNumbers?: boolean;
  placeholder?: string;
  className?: string;
  style?: React.CSSProperties;
  wordWrap?: boolean;
  padding?: number;
  fontSize?: number;
}

export default function CodeEditor({
  value = "",
  onChange,
  language = "json",
  readOnly = false,
  lineNumbers = true,
  placeholder,
  className = "",
  style,
  wordWrap = false,
  padding = 12,
  fontSize: propFontSize,
}: CodeEditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const gutterRef = useRef<HTMLDivElement>(null);
  const jarRef = useRef<CodeJar | null>(null);
  const onChangeRef = useRef(onChange);

  // Keep latest onChange in ref to avoid re-instantiating CodeJar
  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);

  // Settings font store
  const settingsFont = useSettingsStore((s) => s.settings?.font);
  const fontConfig = settingsFont || DEFAULT_FONT_SETTINGS;
  const effectiveFontSize = propFontSize ?? fontConfig.fontSize;
  const computedLineHeight = Math.round(effectiveFontSize * fontConfig.lineHeight);

  // Syntax highlighting via Prism
  const highlight = useCallback(
    (el: HTMLElement) => {
      const code = el.textContent || "";
      const lang = language.toLowerCase();
      const grammar = Prism.languages[lang] || Prism.languages.json || Prism.languages.plain;
      el.innerHTML = Prism.highlight(code, grammar, lang);
    },
    [language]
  );

  // Initialize CodeJar when mounted
  useEffect(() => {
    const el = editorRef.current;
    if (!el) return;

    if (!readOnly) {
      const jar = CodeJar(el, highlight, {
        tab: "  ",
        spellcheck: false,
        catchTab: true,
        preserveIdent: true,
        addClosing: true,
        history: true,
      });

      jar.updateCode(value || "");
      jar.onUpdate((code) => {
        onChangeRef.current?.(code);
      });

      jarRef.current = jar;

      return () => {
        jar.destroy();
        jarRef.current = null;
      };
    } else {
      // Read-only presentation mode
      el.textContent = value || "";
      highlight(el);
    }
  }, [readOnly, highlight]);

  // Synchronize external value changes without losing caret position
  useEffect(() => {
    if (!readOnly && jarRef.current) {
      if (jarRef.current.toString() !== (value || "")) {
        const pos = jarRef.current.save();
        jarRef.current.updateCode(value || "");
        try {
          jarRef.current.restore(pos);
        } catch {
          // ignore restore errors on dramatic text changes
        }
      }
    } else if (readOnly && editorRef.current) {
      if (editorRef.current.textContent !== (value || "")) {
        editorRef.current.textContent = value || "";
        highlight(editorRef.current);
      }
    }
  }, [value, readOnly, highlight]);

  // Synchronize gutter scroll position with editor scroll position
  useEffect(() => {
    const editor = editorRef.current;
    const gutter = gutterRef.current;
    if (!editor || !gutter) return;

    const handleScroll = () => {
      gutter.scrollTop = editor.scrollTop;
    };

    editor.addEventListener("scroll", handleScroll, { passive: true });
    return () => editor.removeEventListener("scroll", handleScroll);
  }, [lineNumbers]);

  // Line count for the gutter
  const lineCount = useMemo(() => {
    return Math.max(1, (value || "").split("\n").length);
  }, [value]);

  const gutterWidth = useMemo(() => {
    const digits = String(lineCount).length;
    return Math.max(34, 16 + digits * 8);
  }, [lineCount]);

  const lineNumbersArray = useMemo(() => {
    const nums = [];
    for (let i = 1; i <= lineCount; i++) {
      nums.push(i);
    }
    return nums;
  }, [lineCount]);

  return (
    <div
      className={`codejar-wrap ${className}`}
      style={{
        fontFamily: fontConfig.fontFamily,
        fontSize: `${effectiveFontSize}px`,
        lineHeight: `${computedLineHeight}px`,
        fontFeatureSettings: fontConfig.enableLigatures
          ? '"liga" 1, "calt" 1'
          : "normal",
        ...style,
      }}
    >
      {/* Line Numbers Gutter */}
      {lineNumbers && (
        <div
          ref={gutterRef}
          aria-hidden="true"
          className="codejar-gutter"
          style={{
            width: `${gutterWidth}px`,
            paddingTop: `${padding}px`,
            paddingBottom: `${padding}px`,
            paddingRight: "8px",
            paddingLeft: "6px",
          }}
          onClick={() => editorRef.current?.focus()}
        >
          {lineNumbersArray.map((num) => (
            <span
              key={num}
              className="codejar-gutter-line"
              style={{ height: `${computedLineHeight}px`, lineHeight: `${computedLineHeight}px` }}
            >
              {num}
            </span>
          ))}
        </div>
      )}

      {/* CodeJar Editor Area */}
      <div
        ref={editorRef}
        className="codejar-content"
        contentEditable={!readOnly}
        data-placeholder={placeholder}
        style={{
          paddingTop: `${padding}px`,
          paddingBottom: `${padding}px`,
          paddingLeft: "12px",
          paddingRight: "12px",
          whiteSpace: wordWrap ? "pre-wrap" : "pre",
          wordBreak: wordWrap ? "break-word" : "normal",
        }}
      />
    </div>
  );
}
