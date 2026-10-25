import { useEffect, useRef } from "react";
import { EditorState, Compartment } from "@codemirror/state";
import {
  EditorView,
  keymap,
  lineNumbers,
  highlightActiveLine,
} from "@codemirror/view";
import {
  defaultKeymap,
  history,
  indentWithTab,
} from "@codemirror/commands";
import {
  indentOnInput,
  bracketMatching,
  StreamLanguage,
  syntaxHighlighting,
  defaultHighlightStyle,
  foldGutter,
  foldKeymap,
} from "@codemirror/language";
import { tags } from "@lezer/highlight";
import { HighlightStyle } from "@codemirror/language";
import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
import { linter, lintGutter, setDiagnostics } from "@codemirror/lint";
import { search, searchKeymap, highlightSelectionMatches } from "@codemirror/search";

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

// Common LaTeX / TikZ commands offered via autocomplete.
const LATEX_COMMANDS = [
  "\\documentclass", "\\usepackage", "\\begin", "\\end", "\\section", "\\subsection",
  "\\title", "\\author", "\\date", "\\maketitle", "\\textbf", "\\textit", "\\emph",
  "\\frac", "\\sqrt", "\\sum", "\\int", "\\alpha", "\\beta", "\\gamma", "\\delta",
  "\\epsilon", "\\theta", "\\lambda", "\\mu", "\\pi", "\\sigma", "\\phi", "\\omega",
  "\\infty", "\\cdot", "\\times", "\\div", "\\pm", "\\leq", "\\geq", "\\neq",
  "\\approx", "\\equiv", "\\subset", "\\supset", "\\cup", "\\cap", "\\in", "\\notin",
  "\\forall", "\\exists", "\\rightarrow", "\\leftarrow", "\\Rightarrow", "\\Leftarrow",
  "\\leftrightarrow", "\\mapsto", "\\partial", "\\nabla", "\\ell", "\\hbar", "\\vec",
  "\\hat", "\\bar", "\\tilde", "\\dot", "\\ddot", "\\overline", "\\underline",
  "\\text", "\\mathrm", "\\mathbf", "\\mathit", "\\mathcal", "\\mathbb", "\\left",
  "\\right", "\\big", "\\Big", "\\bigg", "\\Bigg", "\\label", "\\ref", "\\cite",
  "\\footnote", "\\newline", "\\noindent", "\\indent", "\\vspace", "\\hspace",
  "\\hfill", "\\vfill", "\\newpage", "\\clearpage", "\\pagebreak", "\\linebreak",
  "\\par", "\\centering", "\\raggedright", "\\raggedleft", "\\begin{document}",
  "\\begin{figure}", "\\begin{table}", "\\begin{equation}", "\\begin{align}",
  "\\begin{itemize}", "\\begin{enumerate}", "\\begin{description}", "\\begin{tabular}",
  "\\begin{array}", "\\begin{matrix}", "\\begin{tikzpicture}", "\\draw", "\\node",
  "\\path", "\\fill", "\\shade", "\\clip", "\\coordinate", "\\foreach",
  "\\pgfmathsetmacro", "\\tikzset", "\\tikzstyle", "\\usetikzlibrary",
  "\\definecolor", "\\color", "\\textcolor", "\\colorbox", "\\fcolorbox",
];

// Monokai highlight style (dark).
const monokaiHighlight = HighlightStyle.define([
  { tag: tags.comment, color: "#75715e", fontStyle: "italic" },
  { tag: tags.keyword, color: "#f92672" },
  { tag: tags.string, color: "#e6db74" },
  { tag: tags.number, color: "#ae81ff" },
  { tag: tags.punctuation, color: "#f8f8f2" },
  { tag: tags.bracket, color: "#f8f8f2" },
  { tag: tags.variableName, color: "#f8f8f2" },
  { tag: tags.operator, color: "#f92672" },
  { tag: tags.typeName, color: "#a6e22e" },
  { tag: tags.name, color: "#66d9ef" },
  { tag: tags.number, color: "#ae81ff" },
  { tag: tags.propertyName, color: "#a6e22e" },
  { tag: tags.className, color: "#a6e22e", fontStyle: "italic" },
  { tag: tags.tagName, color: "#f92672" },
  { tag: tags.attributeName, color: "#a6e22e" },
  { tag: tags.attributeValue, color: "#e6db74" },
]);

const noOpLinter = linter(() => []);

const themeCompartment = new Compartment();

function lightExtensions() {
  return [
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    EditorView.theme({
      "&": { backgroundColor: "#ffffff", color: "#1f2937" },
      ".cm-gutters": { backgroundColor: "#f3f4f6", color: "#6b7280", border: "none" },
    }),
  ];
}

function darkExtensions() {
  return [
    syntaxHighlighting(monokaiHighlight),
    EditorView.theme(
      {
        "&": { backgroundColor: "#272822", color: "#f8f8f2" },
        ".cm-gutters": { backgroundColor: "#1e1f1c", color: "#75715e", border: "none" },
        ".cm-activeLine": { backgroundColor: "#3e3d32" },
        ".cm-selectionBackground, ::selection": { backgroundColor: "#49483e" },
        ".cm-cursor": { borderLeftColor: "#f8f8f2" },
      },
      { dark: true }
    ),
  ];
}

interface EditorProps {
  value: string;
  onChange: (value: string) => void;
  errors?: { line: number; message: string }[];
  dark?: boolean;
}

export function Editor({ value, onChange, errors, dark }: EditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);

  useEffect(() => {
    if (!editorRef.current) return;

    const state = EditorState.create({
      doc: value,
      extensions: [
        lineNumbers(),
        foldGutter(),
        highlightActiveLine(),
        history(),
        indentOnInput(),
        bracketMatching(),
        closeBrackets(),
        latexLanguage,
        autocompletion({
          override: [
            (context) => {
              const word = context.matchBefore(/\\[a-zA-Z@]*/);
              if (!word) return null;
              const options = LATEX_COMMANDS.map((label) => ({ label, type: "keyword" }));
              return {
                from: word.from,
                options,
              };
            },
          ],
        }),
        search(),
        highlightSelectionMatches(),
        noOpLinter,
        lintGutter(),
        keymap.of([
          ...closeBracketsKeymap,
          ...defaultKeymap,
          ...searchKeymap,
          ...foldKeymap,
          ...completionKeymap,
          indentWithTab,
        ]),
        themeCompartment.of(dark ? darkExtensions() : lightExtensions()),
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
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

  // Reconfigure theme when dark mode changes.
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({
      effects: themeCompartment.reconfigure(dark ? darkExtensions() : lightExtensions()),
    });
  }, [dark]);

  // Push compilation errors into the lint system when they change.
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const diagnostics = (errors ?? []).map((err) => {
      const lineNum = Math.max(1, Math.min(err.line || 1, view.state.doc.lines));
      const line = view.state.doc.line(lineNum);
      return {
        from: line.from,
        to: line.to,
        severity: "error" as const,
        message: err.message,
      };
    });
    view.dispatch(setDiagnostics(view.state, diagnostics));
  }, [errors]);

  return <div ref={editorRef} className="h-full" />;
}
