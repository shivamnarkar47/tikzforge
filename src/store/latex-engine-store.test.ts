import { describe, it, expect, beforeEach } from "vitest";
import { useLatexEngineStore } from "../store/latex-engine-store";

describe("LatexEngineStore", () => {
  beforeEach(() => {
    useLatexEngineStore.setState({
      status: "unknown",
      installProgress: 0,
      pdflatexPath: null,
      error: null,
    });
  });

  it("has default initial state", () => {
    const { status, installProgress, pdflatexPath, error } = useLatexEngineStore.getState();
    expect(status).toBe("unknown");
    expect(installProgress).toBe(0);
    expect(pdflatexPath).toBeNull();
    expect(error).toBeNull();
  });

  describe("setStatus", () => {
    it("updates status", () => {
      useLatexEngineStore.getState().setStatus("detected");
      expect(useLatexEngineStore.getState().status).toBe("detected");
    });
  });

  describe("setInstallProgress", () => {
    it("updates install progress", () => {
      useLatexEngineStore.getState().setInstallProgress(50);
      expect(useLatexEngineStore.getState().installProgress).toBe(50);
    });

    it("clamps progress to 0-100", () => {
      useLatexEngineStore.getState().setInstallProgress(150);
      expect(useLatexEngineStore.getState().installProgress).toBe(100);
      useLatexEngineStore.getState().setInstallProgress(-10);
      expect(useLatexEngineStore.getState().installProgress).toBe(0);
    });
  });

  describe("setPdflatexPath", () => {
    it("sets the pdflatex path", () => {
      useLatexEngineStore.getState().setPdflatexPath("/usr/bin/pdflatex");
      expect(useLatexEngineStore.getState().pdflatexPath).toBe("/usr/bin/pdflatex");
    });
  });

  describe("setError", () => {
    it("sets the error message", () => {
      useLatexEngineStore.getState().setError("Installation failed");
      expect(useLatexEngineStore.getState().error).toBe("Installation failed");
    });
  });

  describe("isReady", () => {
    it("returns true when status is 'ready'", () => {
      useLatexEngineStore.getState().setStatus("ready");
      expect(useLatexEngineStore.getState().isReady()).toBe(true);
    });

    it("returns false when status is not 'ready'", () => {
      useLatexEngineStore.getState().setStatus("installing");
      expect(useLatexEngineStore.getState().isReady()).toBe(false);
    });
  });

  describe("isDownloading", () => {
    it("returns true when status is 'installing'", () => {
      useLatexEngineStore.getState().setStatus("installing");
      expect(useLatexEngineStore.getState().isDownloading()).toBe(true);
    });

    it("returns false when status is not 'installing'", () => {
      useLatexEngineStore.getState().setStatus("ready");
      expect(useLatexEngineStore.getState().isDownloading()).toBe(false);
    });
  });
});
