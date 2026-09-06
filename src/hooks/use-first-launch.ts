import { useEffect } from "react";
import { detectInstallation } from "../lib/latex-engine";
import { useLatexEngineStore } from "../store/latex-engine-store";

/**
 * First-launch LaTeX engine detection.
 *
 * On mount, if the engine store is still in its initial "unknown" state,
 * this hook calls `detectInstallation` and transitions the store:
 *   - detected=true  → status "ready" with the resolved path
 *   - detected=false → status "error" with a bundled-engine hint
 *   - throw          → status "error" with the error message
 *
 * If the store is already "ready" (e.g. persisted from a prior session),
 * detection is skipped.
 */
export function useFirstLaunch(): void {
  const { status, setStatus, setPdflatexPath, setError } = useLatexEngineStore.getState();

  useEffect(() => {
    if (status !== "unknown") return;

    let cancelled = false;
    setStatus("detected");

    detectInstallation()
      .then((result) => {
        if (cancelled) return;
        if (result.detected && result.path) {
          setPdflatexPath(result.path);
          setStatus("ready");
        } else {
          setError(
            "No bundled TeX Live found. Reinstall TikzForge or run the TeX Live bundling step."
          );
          setStatus("error");
        }
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        const message = err instanceof Error ? err.message : String(err);
        setError(message);
        setStatus("error");
      });

    return () => {
      cancelled = true;
    };
    // Intentionally run once on mount when status is "unknown".
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status, setStatus, setPdflatexPath, setError]);
}
