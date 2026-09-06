import { useCallback } from "react";
import { compileDocument } from "../lib/compilation";
import { parseCompilationLog } from "../lib/compilation-parser";
import { useDocumentStore } from "../store/document-store";

export function useCompile() {
  const compile = useCallback(async (path: string, content: string) => {
    const { setCompiling, setPdfData, setCompilationLog, setCompilationErrors } =
      useDocumentStore.getState();

    setCompiling(true);

    try {
      const result = await compileDocument(path, content);

      setCompilationLog(result.log);

      if (result.success && result.pdf) {
        setPdfData(result.pdf);
        setCompilationErrors([]);
      } else {
        const errors = parseCompilationLog(result.log);
        setCompilationErrors(errors);
        setPdfData(null);
      }
    } finally {
      setCompiling(false);
    }
  }, []);

  return { compile };
}
