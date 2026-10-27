import { useEffect, useRef } from "react";
import { useDocumentStore } from "../store/document-store";

/**
 * Raw engine transcript. Lines stream in live while Tectonic runs; when the
 * compile settles, the store holds the complete transcript.
 */
export function LogPanel() {
  const { compilationLog } = useDocumentStore();
  const preRef = useRef<HTMLPreElement>(null);

  useEffect(() => {
    const el = preRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [compilationLog]);

  if (!compilationLog) {
    return (
      <p className="px-3 py-2 text-xs text-muted-foreground">
        No compilation log yet — press Compile to build your document.
      </p>
    );
  }

  return (
    <pre
      ref={preRef}
      role="region"
      aria-label="Compilation log"
      className="max-h-48 overflow-auto whitespace-pre-wrap px-3 py-2 font-mono text-xs text-muted-foreground"
    >
      {compilationLog}
    </pre>
  );
}
