import { javascript } from "@codemirror/lang-javascript";
import { EditorState } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";
import { EditorView } from "@codemirror/view";
import { basicSetup } from "codemirror";
import { useEffect, useRef } from "react";

interface RhaiEditorProps {
  value: string;
  onChange: (value: string) => void;
  /** Ctrl+S – save current script */
  onSave?: () => void;
  /** Ctrl+N – new file */
  onNew?: () => void;
}

export function RhaiEditor({ value, onChange, onSave, onNew }: RhaiEditorProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | undefined>(undefined);
  const onChangeRef = useRef(onChange);
  const onSaveRef = useRef(onSave);
  const onNewRef = useRef(onNew);
  onChangeRef.current = onChange;
  onSaveRef.current = onSave;
  onNewRef.current = onNew;

  useEffect(() => {
    if (!containerRef.current) return;

    const container = containerRef.current;

    // Intercept in capture phase — fires before CodeMirror and before the
    // browser default (F5 refresh in Firefox, newline on Enter, etc.)
    const handleKeydown = (event: KeyboardEvent) => {
      const mod = event.ctrlKey || event.metaKey;

      if (mod && event.key === "s") {
        event.preventDefault();
        event.stopPropagation();
        onSaveRef.current?.();
        return;
      }
      if (mod && event.key === "n") {
        event.preventDefault();
        event.stopPropagation();
        onNewRef.current?.();
      }
    };

    container.addEventListener("keydown", handleKeydown, { capture: true });

    const state = EditorState.create({
      doc: value,
      extensions: [
        basicSetup,
        javascript(),
        oneDark,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            onChangeRef.current(update.state.doc.toString());
          }
        }),
        EditorView.theme({
          "&": { height: "100%" },
          ".cm-scroller": { overflow: "auto" },
        }),
      ],
    });

    const view = new EditorView({
      state,
      parent: container,
    });

    viewRef.current = view;

    return () => {
      container.removeEventListener("keydown", handleKeydown, { capture: true });
      view.destroy();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Sync external value changes (e.g. loading a preset)
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const current = view.state.doc.toString();
    if (current !== value) {
      view.dispatch({
        changes: { from: 0, to: current.length, insert: value },
      });
    }
  }, [value]);

  return (
    <div
      ref={containerRef}
      className="border border-border rounded-lg overflow-hidden h-full"
    />
  );
}
