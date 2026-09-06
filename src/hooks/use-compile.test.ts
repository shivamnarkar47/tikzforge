import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useCompile } from "../hooks/use-compile";
import { useDocumentStore } from "../store/document-store";

vi.mock("../lib/compilation", () => ({
  compileDocument: vi.fn(),
}));

vi.mock("../lib/compilation-parser", () => ({
  parseCompilationLog: vi.fn(),
}));

import { compileDocument } from "../lib/compilation";
import { parseCompilationLog } from "../lib/compilation-parser";

describe("useCompile", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useDocumentStore.setState({
      filename: "",
      content: "",
      isDirty: false,
      pdfData: null,
      isCompiling: false,
      compilationErrors: [],
      compilationLog: "",
      recentFiles: [],
    });
  });

  it("sets isCompiling to true during compile", async () => {
    vi.mocked(compileDocument).mockImplementation(() => {
      expect(useDocumentStore.getState().isCompiling).toBe(true);
      return Promise.resolve({
        pdf: new Uint8Array([1, 2, 3]),
        log: "Success",
        success: true,
      });
    });
    vi.mocked(parseCompilationLog).mockReturnValue([]);

    const { result } = renderHook(() => useCompile());

    await act(async () => {
      await result.current.compile("/path/doc.tex", "content");
    });

    expect(useDocumentStore.getState().isCompiling).toBe(false);
  });

  it("sets pdfData on successful compile", async () => {
    const pdfBytes = new Uint8Array([1, 2, 3, 4, 5]);
    vi.mocked(compileDocument).mockResolvedValue({
      pdf: pdfBytes,
      log: "Output written",
      success: true,
    });
    vi.mocked(parseCompilationLog).mockReturnValue([]);

    const { result } = renderHook(() => useCompile());

    await act(async () => {
      await result.current.compile("/path/doc.tex", "content");
    });

    expect(useDocumentStore.getState().pdfData).toBe(pdfBytes);
    expect(useDocumentStore.getState().compilationLog).toBe("Output written");
  });

  it("sets errors on failed compile", async () => {
    const log = "! Undefined control sequence.";
    const errors = [{ file: "doc.tex", line: 10, message: "Undefined control sequence" }];
    vi.mocked(compileDocument).mockResolvedValue({
      pdf: null,
      log,
      success: false,
    });
    vi.mocked(parseCompilationLog).mockReturnValue(errors);

    const { result } = renderHook(() => useCompile());

    await act(async () => {
      await result.current.compile("/path/doc.tex", "\\badcommand");
    });

    expect(useDocumentStore.getState().compilationErrors).toEqual(errors);
    expect(useDocumentStore.getState().compilationLog).toBe(log);
  });

  it("clears errors on successful compile", async () => {
    useDocumentStore.setState({
      compilationErrors: [{ file: "doc.tex", line: 5, message: "Old error" }],
    });

    vi.mocked(compileDocument).mockResolvedValue({
      pdf: new Uint8Array([1]),
      log: "Success",
      success: true,
    });
    vi.mocked(parseCompilationLog).mockReturnValue([]);

    const { result } = renderHook(() => useCompile());

    await act(async () => {
      await result.current.compile("/path/doc.tex", "content");
    });

    expect(useDocumentStore.getState().compilationErrors).toEqual([]);
  });

  it("surfaces invoke failures as errors instead of throwing", async () => {
    vi.mocked(compileDocument).mockRejectedValue(
      new Error(
        "Compilation requires the Tauri desktop app — you are running in a browser preview."
      )
    );

    const { result } = renderHook(() => useCompile());

    await act(async () => {
      await result.current.compile("/path/doc.tex", "content");
    });

    const state = useDocumentStore.getState();
    expect(state.isCompiling).toBe(false);
    expect(state.pdfData).toBeNull();
    expect(state.compilationErrors).toHaveLength(1);
    expect(state.compilationErrors[0].message).toContain("browser preview");
  });
});
