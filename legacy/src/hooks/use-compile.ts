import { useCallback, useState, useRef } from "react";
import { compileDocument, cancelCompile } from "../lib/compilation";
import { parseCompilationLog } from "../lib/compilation-parser";
import { useDocumentStore } from "../store/document-store";

export type CompilePhase = "idle" | "queued" | "compiling" | "cancelling";

/**
 * Latest compile wins: each `compile()` call supersedes the previous one.
 * A superseded (or cancelled) run that settles late must not overwrite fresh
 * state or re-light the spinner, so every settle path checks its generation.
 */
let latestCompileId = 0;

interface QueuedCompile {
  id: number;
  path: string;
  content: string;
}

export function useCompile() {
  const [phase, setPhase] = useState<CompilePhase>("idle");
  const busyRef = useRef(false);
  const queuedRef = useRef<QueuedCompile | null>(null);
  const runCompileRef = useRef<
    ((id: number, path: string, content: string) => Promise<void>) | undefined
  >(undefined);

  const runCompile = useCallback(
    async (id: number, path: string, content: string) => {
      const { setCompiling, setPdfData, setCompilationLog, setCompilationErrors } =
        useDocumentStore.getState();

      setPhase("compiling");
      setCompiling(true);
      // Fresh transcript per run; streamed lines append below as they arrive.
      setCompilationLog("");

      try {
        const result = await compileDocument(path, content, (line) => {
          if (id !== latestCompileId) return;
          useDocumentStore.getState().appendCompilationLog(line);
        });
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
        const queued = queuedRef.current;
        if (queued) {
          queuedRef.current = null;
          await runCompileRef.current?.(queued.id, queued.path, queued.content);
        } else {
          busyRef.current = false;
          setCompiling(false);
          setPhase("idle");
        }
      }
    },
    []
  );

  // Keep the recursive reference up to date.
  runCompileRef.current = runCompile;

  const compile = useCallback(
    async (path: string, content: string) => {
      const id = ++latestCompileId;
      if (busyRef.current) {
        queuedRef.current = { id, path, content };
        setPhase("queued");
        return;
      }
      busyRef.current = true;
      queuedRef.current = null;
      await runCompile(id, path, content);
    },
    [runCompile]
  );

  const cancel = useCallback(() => {
    latestCompileId++;
    queuedRef.current = null;
    setPhase("cancelling");
    useDocumentStore.getState().setCompiling(false);
    void cancelCompile();
  }, []);

  const isCompiling = phase === "compiling" || phase === "queued";

  return { compile, cancel, phase, isCompiling };
}