import { useDocumentStore } from "../store/document-store";

/**
 * Raw transcript of the last finished compile (or the failure message).
 * The backend only returns output when Tectonic exits, so during a long
 * compile this shows the previous transcript — not a live stream.
 */
export function LogPanel() {
  const { compilationLog } = useDocumentStore();

  if (!compilationLog) {
    return (
      <p className="px-3 py-2 text-xs text-muted-foreground">
        No compilation log yet — press Compile to build your document.
      </p>
    );
  }

  return (
    <pre
      role="region"
      aria-label="Compilation log"
      className="max-h-48 overflow-auto whitespace-pre-wrap px-3 py-2 font-mono text-xs text-muted-foreground"
    >
      {compilationLog}
    </pre>
  );
}
