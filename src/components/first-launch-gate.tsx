import { useLatexEngineStore } from "../store/latex-engine-store";
import { Button } from "./motion/button/base";
import { Card, CardContent, CardHeader } from "./motion/card";
import { Alert, AlertDescription, AlertTitle } from "./motion/alert";

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
      <Card className="max-w-md text-center">
        <CardHeader>
          <Alert variant="destructive">
            <AlertTitle>LaTeX not found</AlertTitle>
            <AlertDescription>
              {error ??
                "TikzForge could not find its bundled Tectonic engine. Please reinstall the app."}
            </AlertDescription>
          </Alert>
        </CardHeader>
        <CardContent className="flex flex-col items-center gap-4">
          {pdflatexPath && (
            <p className="font-mono text-xs text-muted-foreground">
              Expected at: {pdflatexPath}
            </p>
          )}
          <Button variant="outline" size="sm" onClick={() => window.location.reload()}>
            Retry
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
