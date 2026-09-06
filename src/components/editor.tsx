import { useEffect, useRef } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers, highlightActiveLine } from "@codemirror/view";
import { defaultKeymap, history } from "@codemirror/commands";
import { indentOnInput, bracketMatching, StreamLanguage, syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";

// Define a LaTeX stream language for syntax highlighting
const latexLanguage = StreamLanguage.define({
  token(stream) {
    // Comment: from % to end of line
    if (stream.match(/%.*/)) return "comment";

    // LaTeX command: backslash followed by word or single non-word char
    if (stream.match(/\\[a-zA-Z@]+/)) return "keyword";
    if (stream.match(/\\[^a-zA-Z@]/)) return "keyword";

    // Math delimiter
    if (stream.match(/\$/)) return "string";

    // Braces and brackets
    if (stream.match(/[{}()\[\]]/)) return "punctuation";

    // Number
    if (stream.match(/\d+(\.\d+)?/)) return "number";

    // Plain text
    if (stream.match(/[^%\\{}[\]()$]+/)) return "content";

    stream.next();
    return null;
  },
});

interface EditorProps {
  value: string;
  onChange: (value: string) => void;
}

export function Editor({ value, onChange }: EditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);

  useEffect(() => {
    if (!editorRef.current) return;

    const state = EditorState.create({
      doc: value,
      extensions: [
        lineNumbers(),
        highlightActiveLine(),
        history(),
        indentOnInput(),
        bracketMatching(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        latexLanguage,
        keymap.of(defaultKeymap),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            onChange(update.state.doc.toString());
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
      parent: editorRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
  }, []);

  useEffect(() => {
    const view = viewRef.current;
    if (view && value !== view.state.doc.toString()) {
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: value,
        },
      });
    }
  }, [value]);

  return <div ref={editorRef} className="h-full" />;
}
