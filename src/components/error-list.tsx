import { useDocumentStore } from "../store/document-store";

interface ErrorListProps {
  onSelect?: (line: number) => void;
}

export function ErrorList({ onSelect }: ErrorListProps) {
  const { compilationErrors } = useDocumentStore();

  if (compilationErrors.length === 0) return null;

  return (
    <div className="error-list border border-destructive/30 rounded-md bg-destructive/5 p-2 max-h-40 overflow-y-auto">
      {compilationErrors.map((error, i) => (
        <div
          key={`${error.file}-${error.line}-${i}`}
          className="error-item flex items-center gap-2 px-2 py-1 text-sm hover:bg-destructive/10 rounded cursor-pointer"
          onClick={() => onSelect?.(error.line)}
        >
          <span className="text-destructive font-mono text-xs shrink-0">
            L{error.line}
          </span>
          <span className="text-foreground truncate">{error.message}</span>
        </div>
      ))}
    </div>
  );
}
