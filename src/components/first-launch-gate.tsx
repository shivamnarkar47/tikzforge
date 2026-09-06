import { useLatexEngineStore } from "../store/latex-engine-store";
import { Button } from "./ui/button";
import { AlertCircle } from "lucide-react";

interface FirstLaunchGateProps {
  children: React.ReactNode;
}

/**
 * Blocks the editor/PDF UI until the bundled LaTeX engine is confirmed to
 * exist. With bundling there is no download — detection is a synchronous
 * file-existence check, so this gate either passes immediately or shows a
 * static error card.
 */
export function FirstLaunchGate({ children }: FirstLaunchGateProps) {
  const { status, error, pdflatexPath } = useLatexEngineStore();

  if (status === "ready") {
    return <>{children}</>;
  }

  // "detected" is a transient state that resolves within the same tick;
  // render the error card if we're stuck there or in "error".
  return (
    <div className="flex h-full w-full items-center justify-center bg-background p-6">
      <div className="flex max-w-md flex-col items-center gap-4 rounded-lg border bg-card p-8 text-center shadow-sm">
        <AlertCircle className="size-8 text-destructive" />
        <h2 className="text-lg font-semibold">LaTeX not found</h2>
        <p className="text-sm text-muted-foreground">
          {error ??
            "TikzForge could not find its bundled TeX Live engine. Please reinstall the app."}
        </p>
        {pdflatexPath && (
          <p className="font-mono text-xs text-muted-foreground">
            Expected at: {pdflatexPath}
          </p>
        )}
        <Button variant="outline" size="sm" onClick={() => window.location.reload()}>
          Retry
        </Button>
      </div>
    </div>
  );
}
