import { useEffect, useRef } from "react";
import { EditorState } from "@codemirror/state";
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
  "\\documentclass",
  "\\usepackage",
  "\\begin",
  "\\end",
  "\\section",
  "\\subsection",
  "\\title",
  "\\author",
  "\\date",
  "\\maketitle",
  "\\textbf",
  "\\textit",
  "\\emph",
  "\\frac",
  "\\sqrt",
  "\\sum",
  "\\int",
  "\\alpha",
  "\\beta",
  "\\gamma",
  "\\delta",
  "\\epsilon",
  "\\theta",
  "\\lambda",
  "\\mu",
  "\\pi",
  "\\sigma",
  "\\phi",
  "\\omega",
  "\\infty",
  "\\cdot",
  "\\times",
  "\\div",
  "\\pm",
  "\\leq",
  "\\geq",
  "\\neq",
  "\\approx",
  "\\equiv",
  "\\subset",
  "\\supset",
  "\\cup",
  "\\cap",
  "\\in",
  "\\notin",
  "\\forall",
  "\\exists",
  "\\rightarrow",
  "\\leftarrow",
  "\\Rightarrow",
  "\\Leftarrow",
  "\\leftrightarrow",
  "\\mapsto",
  "\\partial",
  "\\nabla",
  "\\ell",
  "\\hbar",
  "\\vec",
  "\\hat",
  "\\bar",
  "\\tilde",
  "\\dot",
  "\\ddot",
  "\\overline",
  "\\underline",
  "\\text",
  "\\mathrm",
  "\\mathbf",
  "\\mathit",
  "\\mathcal",
  "\\mathbb",
  "\\left",
  "\\right",
  "\\big",
  "\\Big",
  "\\bigg",
  "\\Bigg",
  "\\label",
  "\\ref",
  "\\cite",
  "\\footnote",
  "\\marginpar",
  "\\newline",
  "\\noindent",
  "\\indent",
  "\\vspace",
  "\\hspace",
  "\\hfill",
  "\\vfill",
  "\\newpage",
  "\\clearpage",
  "\\pagebreak",
  "\\linebreak",
  "\\par",
  "\\centering",
  "\\raggedright",
  "\\raggedleft",
  "\\flushleft",
  "\\flushright",
  "\\begin{document}",
  "\\begin{figure}",
  "\\begin{table}",
  "\\begin{equation}",
  "\\begin{align}",
  "\\begin{itemize}",
  "\\begin{enumerate}",
  "\\begin{description}",
  "\\begin{tabular}",
  "\\begin{array}",
  "\\begin{matrix}",
  "\\begin{tikzpicture}",
  "\\draw",
  "\\node",
  "\\path",
  "\\fill",
  "\\shade",
  "\\clip",
  "\\coordinate",
  "\\foreach",
  "\\pgfmathsetmacro",
  "\\tikzset",
  "\\tikzstyle",
  "\\usetikzlibrary",
  "\\definecolor",
  "\\color",
  "\\textcolor",
  "\\colorbox",
  "\\fcolorbox",
];

const noOpLinter = linter(() => []);

interface EditorProps {
  value: string;
  onChange: (value: string) => void;
  errors?: { line: number; message: string }[];
}

export function Editor({ value, onChange, errors }: EditorProps) {
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
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
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
