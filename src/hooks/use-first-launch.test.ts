import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useFirstLaunch } from "../hooks/use-first-launch";
import { useLatexEngineStore } from "../store/latex-engine-store";

vi.mock("../lib/latex-engine", () => ({
  detectInstallation: vi.fn(),
}));

import { detectInstallation } from "../lib/latex-engine";

describe("useFirstLaunch", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    useLatexEngineStore.setState({
      status: "unknown",
      installProgress: 0,
      pdflatexPath: null,
      error: null,
    });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("starts detection on mount", () => {
    vi.mocked(detectInstallation).mockResolvedValue({ detected: false, path: null });

    renderHook(() => useFirstLaunch());

    expect(useLatexEngineStore.getState().status).toBe("detected");
  });

  it("transitions to ready when the bundled engine is detected", async () => {
    vi.mocked(detectInstallation).mockResolvedValue({
      detected: true,
      path: "/app/resources/tectonic/tectonic",
    });

    renderHook(() => useFirstLaunch());

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });

    const state = useLatexEngineStore.getState();
    expect(state.status).toBe("ready");
    expect(state.pdflatexPath).toBe("/app/resources/tectonic/tectonic");
  });

  it("transitions to error when detection returns null", async () => {
    vi.mocked(detectInstallation).mockResolvedValue({ detected: false, path: null });

    renderHook(() => useFirstLaunch());

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });

    const state = useLatexEngineStore.getState();
    expect(state.status).toBe("error");
    expect(state.error).toMatch(/tectonic/i);
  });

  it("transitions to error when detection throws", async () => {
    vi.mocked(detectInstallation).mockRejectedValue(new Error("Tauri unavailable"));

    renderHook(() => useFirstLaunch());

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });

    const state = useLatexEngineStore.getState();
    expect(state.status).toBe("error");
    expect(state.error).toContain("Tauri unavailable");
  });

  it("does not re-run detection if status is already ready", () => {
    useLatexEngineStore.setState({ status: "ready", pdflatexPath: "/usr/bin/pdflatex" });
    vi.mocked(detectInstallation).mockResolvedValue({ detected: true, path: "/x" });

    renderHook(() => useFirstLaunch());

    expect(detectInstallation).not.toHaveBeenCalled();
  });
});
