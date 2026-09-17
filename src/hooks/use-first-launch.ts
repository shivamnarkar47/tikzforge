import { useEffect } from "react";
import { detectInstallation, type DetectionResult } from "../lib/latex-engine";
import { useLatexEngineStore } from "../store/latex-engine-store";

/**
 * First-launch LaTeX engine detection.
 *
 * On mount, if the engine store has no resolved status ("ready"/"error"),
 * this hook calls `detectInstallation` and transitions the store:
 *   - detected=true  → status "ready" with the resolved path
 *   - detected=false → status "error" with a bundled-engine hint
 *   - throw          → status "error" with the error message
 *
 * The in-flight promise is shared module-wide: post-mount re-renders (App's
 * `setReady`, StrictMode remounts) tear down and re-run this effect while
 * detection is still pending. Re-attaching to the same promise instead of
 * starting over keeps exactly one `invoke` in flight and guarantees the
 * latest mount applies the result — otherwise cleanup cancels the only
 * request and the gate sticks on "detected" forever.
 */
let inFlight: Promise<DetectionResult> | null = null;

export function useFirstLaunch(): void {
  useEffect(() => {
    const { status, setStatus, setPdflatexPath, setError } =
      useLatexEngineStore.getState();
    if (status === "ready" || status === "error") return;

    let cancelled = false;
    setStatus("detected");

    const pending =
      inFlight ??
      (inFlight = detectInstallation().finally(() => {
        inFlight = null;
      }));

    pending
      .then((result) => {
        if (cancelled) return;
        if (result.detected && result.path) {
          setPdflatexPath(result.path);
          setStatus("ready");
        } else {
          setError(
            "No bundled Tectonic engine found. Reinstall TikzForge or run scripts/prepare-texlive.sh before building."
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
    // Re-runs when App re-renders; in-flight sharing keeps one request alive.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [useLatexEngineStore]);
}
