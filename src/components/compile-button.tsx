import { useDocumentStore } from "../store/document-store";

interface CompileButtonProps {
  onCompile: () => void;
}

export function CompileButton({ onCompile }: CompileButtonProps) {
  const { isCompiling } = useDocumentStore();

  return (
    <button
      onClick={onCompile}
      disabled={isCompiling}
      className="px-4 py-2 bg-primary text-primary-foreground rounded-md disabled:opacity-50 flex items-center gap-2"
    >
      {isCompiling ? (
        <>
          <span data-testid="compile-spinner" className="animate-spin">
            ⏳
          </span>
          Compiling...
        </>
      ) : (
        "Compile"
      )}
    </button>
  );
}
