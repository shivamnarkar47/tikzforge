import { useCallback } from "react";
import { compileDocument, cancelCompile } from "../lib/compilation";
import { parseCompilationLog } from "../lib/compilation-parser";
import { useDocumentStore } from "../store/document-store";

/**
 * Latest compile wins: each `compile()` call supersedes the previous one.
 * A superseded (or cancelled) run that settles late must not overwrite fresh
 * state or re-light the spinner, so every settle path checks its generation.
 */
let latestCompileId = 0;

export function useCompile() {
  const compile = useCallback(async (path: string, content: string) => {
    const id = ++latestCompileId;
    // Kill a previous backend run still in flight, if any.
    void cancelCompile();

    const { setCompiling, setPdfData, setCompilationLog, setCompilationErrors } =
      useDocumentStore.getState();

    setCompiling(true);

    try {
      const result = await compileDocument(path, content);
      if (id !== latestCompileId) return;

      setCompilationLog(result.log);

      if (result.success && result.pdf) {
        setPdfData(result.pdf);
        setCompilationErrors([]);
      } else {
        const errors = parseCompilationLog(result.log);
        setCompilationErrors(errors);
        setPdfData(null);
      }
    } catch (err) {
      if (id !== latestCompileId) return;
      const message = err instanceof Error ? err.message : String(err);
      setCompilationLog(message);
      setCompilationErrors([{ file: path, line: 1, message }]);
      setPdfData(null);
    } finally {
      if (id === latestCompileId) setCompiling(false);
    }
  }, []);

  const cancel = useCallback(() => {
    latestCompileId++;
    useDocumentStore.getState().setCompiling(false);
    void cancelCompile();
  }, []);

  return { compile, cancel };
}
