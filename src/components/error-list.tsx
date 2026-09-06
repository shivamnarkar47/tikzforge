import { useDocumentStore } from "../store/document-store";
import { ScrollArea } from "./ui/scroll-area";
import { TriangleAlert } from "lucide-react";

interface ErrorListProps {
  onSelect?: (line: number) => void;
}

export function ErrorList({ onSelect }: ErrorListProps) {
  const { compilationErrors } = useDocumentStore();

  if (compilationErrors.length === 0) return null;

  return (
    <ScrollArea className="error-list border border-destructive/30 rounded-md bg-destructive/5 max-h-40">
      <div className="p-2">
        {compilationErrors.map((error, i) => (
          <button
            key={`${error.file}-${error.line}-${i}`}
            type="button"
            className="error-item flex w-full items-center gap-2 px-2 py-1 text-sm hover:bg-destructive/10 rounded cursor-pointer text-left"
            onClick={() => onSelect?.(error.line)}
          >
            <span className="inline-flex items-center gap-1 text-destructive font-mono text-xs shrink-0">
              <TriangleAlert className="size-3" />
              L{error.line}
            </span>
            <span className="text-foreground truncate">{error.message}</span>
          </button>
        ))}
      </div>
    </ScrollArea>
  );
}
