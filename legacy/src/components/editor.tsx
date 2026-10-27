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
const kanagawaWaveHighlight = HighlightStyle.define([
  { tag: tags.comment, color: "#727169", fontStyle: "italic" },
  { tag: tags.keyword, color: "#957FB8" },
  { tag: tags.string, color: "#98BB6C" },
  { tag: tags.number, color: "#D27E99" },
  { tag: tags.punctuation, color: "#9CABCA" },
  { tag: tags.bracket, color: "#9CABCA" },
  { tag: tags.variableName, color: "#DCD7BA" },
  { tag: tags.operator, color: "#C34043" },
  { tag: tags.typeName, color: "#7FB4CA" },
  { tag: tags.name, color: "#7E9CD8" },
  { tag: tags.propertyName, color: "#7FB4CA" },
  { tag: tags.tagName, color: "#E46876" },
  { tag: tags.attributeName, color: "#98BB6C" },
  { tag: tags.attributeValue, color: "#98BB6C" },
]);

const kanagawaLotusHighlight = HighlightStyle.define([
  { tag: tags.comment, color: "#8a8980", fontStyle: "italic" },
  { tag: tags.keyword, color: "#624485" },
  { tag: tags.string, color: "#6f894e" },
  { tag: tags.number, color: "#b35b79" },
  { tag: tags.punctuation, color: "#6f6f6f" },
  { tag: tags.bracket, color: "#6f6f6f" },
  { tag: tags.variableName, color: "#545464" },
  { tag: tags.operator, color: "#c34043" },
  { tag: tags.typeName, color: "#4e8ca2" },
  { tag: tags.name, color: "#4d699b" },
  { tag: tags.propertyName, color: "#4e8ca2" },
  { tag: tags.tagName, color: "#c34043" },
  { tag: tags.attributeName, color: "#6f894e" },
  { tag: tags.attributeValue, color: "#6f894e" },
]);

const noOpLinter = linter(() => []);

const themeCompartment = new Compartment();

function lightExtensions() {
  return [
    syntaxHighlighting(kanagawaLotusHighlight),
    EditorView.theme({
      "&": { backgroundColor: "#f2ecbc", color: "#545464" },
      ".cm-gutters": { backgroundColor: "#e8e2b8", color: "#8a8980", border: "none" },
      ".cm-activeLine": { backgroundColor: "#e8e2b8" },
      ".cm-selectionBackground, ::selection": { backgroundColor: "#c9c39a" },
      ".cm-cursor": { borderLeftColor: "#545464" },
    }),
  ];
}

function darkExtensions() {
  return [
    syntaxHighlighting(kanagawaWaveHighlight),
    EditorView.theme(
      {
        "&": { backgroundColor: "#1F1F28", color: "#DCD7BA" },
        ".cm-gutters": { backgroundColor: "#16161D", color: "#727169", border: "none" },
        ".cm-activeLine": { backgroundColor: "#2A2A37" },
        ".cm-selectionBackground, ::selection": { backgroundColor: "#2D4F67" },
        ".cm-cursor": { borderLeftColor: "#DCD7BA" },
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
