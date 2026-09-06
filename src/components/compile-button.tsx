import { useDocumentStore } from "../store/document-store";
import { Button } from "./ui/button";
import { Loader2, Play } from "lucide-react";

interface CompileButtonProps {
  onCompile: () => void;
}

export function CompileButton({ onCompile }: CompileButtonProps) {
  const { isCompiling } = useDocumentStore();

  return (
    <Button onClick={onCompile} disabled={isCompiling} size="sm">
      {isCompiling ? (
        <>
          <Loader2 data-testid="compile-spinner" className="animate-spin" />
          Compiling...
        </>
      ) : (
        <>
          <Play />
          Compile
        </>
      )}
    </Button>
  );
}
