import { useEffect, useState } from "react";
import { useDocumentStore } from "../store/document-store";
import { Button } from "./ui/button";
import { Loader2, Play } from "lucide-react";

interface CompileButtonProps {
  onCompile: () => void;
  onCancel: () => void;
}

export function CompileButton({ onCompile, onCancel }: CompileButtonProps) {
  const { isCompiling } = useDocumentStore();
  const [elapsed, setElapsed] = useState(0);

  useEffect(() => {
    if (!isCompiling) return;
    setElapsed(0);
    const timer = setInterval(() => setElapsed((s) => s + 1), 1000);
    return () => clearInterval(timer);
  }, [isCompiling]);

  return (
    <Button onClick={isCompiling ? onCancel : onCompile} size="sm">
      {isCompiling ? (
        <>
          <Loader2 data-testid="compile-spinner" className="animate-spin" />
          Cancel ({elapsed}s)
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
