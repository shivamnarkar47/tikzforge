import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useCompile } from "../hooks/use-compile";
import { useDocumentStore } from "../store/document-store";

vi.mock("../lib/compilation", () => ({
  compileDocument: vi.fn(),
  cancelCompile: vi.fn(),
}));

vi.mock("../lib/compilation-parser", () => ({
  parseCompilationLog: vi.fn(),
}));

import { compileDocument, cancelCompile } from "../lib/compilation";
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

  it("ignores a stale compile that settles after a newer one", async () => {
    let resolveStale!: (v: {
      pdf: Uint8Array | null;
      log: string;
      success: boolean;
    }) => void;
    const stale = new Promise<{
      pdf: Uint8Array | null;
      log: string;
      success: boolean;
    }>((resolve) => {
      resolveStale = resolve;
    });
    const freshPdf = new Uint8Array([9, 9, 9]);
    vi.mocked(compileDocument)
      .mockReturnValueOnce(stale)
      .mockResolvedValueOnce({ pdf: freshPdf, log: "fresh", success: true });
    vi.mocked(parseCompilationLog).mockReturnValue([]);

    const { result } = renderHook(() => useCompile());

    let first: Promise<void>;
    act(() => {
      first = result.current.compile("/path/doc.tex", "stale");
    });
    await act(async () => {
      await result.current.compile("/path/doc.tex", "fresh");
    });
    await act(async () => {
      resolveStale({ pdf: new Uint8Array([1]), log: "stale", success: true });
      await first;
    });

    const state = useDocumentStore.getState();
    expect(state.pdfData).toBe(freshPdf);
    expect(state.isCompiling).toBe(false);
  });

  it("appends streamed log lines live, then sets the final transcript", async () => {
    type CompileValue = {
      pdf: Uint8Array | null;
      log: string;
      success: boolean;
    };
    let capturedOnLog: ((line: string) => void) | undefined;
    let resolveCompile!: (v: CompileValue) => void;
    vi.mocked(compileDocument).mockImplementation((_path, _content, onLogLine) => {
      capturedOnLog = onLogLine;
      return new Promise<CompileValue>((resolve) => {
        resolveCompile = resolve;
      });
    });
    vi.mocked(parseCompilationLog).mockReturnValue([]);

    const { result } = renderHook(() => useCompile());

    let pending: Promise<void>;
    act(() => {
      pending = result.current.compile("/path/doc.tex", "content");
    });
    act(() => {
      capturedOnLog?.("note: downloading foo");
    });
    expect(useDocumentStore.getState().compilationLog).toContain(
      "note: downloading foo"
    );

    await act(async () => {
      resolveCompile({ pdf: new Uint8Array([1]), log: "full transcript", success: true });
      await pending;
    });
    expect(useDocumentStore.getState().compilationLog).toBe("full transcript");
  });

  it("cancel() clears the spinner and ignores the late result", async () => {    let resolveLate!: (v: {
      pdf: Uint8Array | null;
      log: string;
      success: boolean;
    }) => void;
    const late = new Promise<{
      pdf: Uint8Array | null;
      log: string;
      success: boolean;
    }>((resolve) => {
      resolveLate = resolve;
    });
    vi.mocked(compileDocument).mockReturnValueOnce(late);
    vi.mocked(parseCompilationLog).mockReturnValue([]);

    const { result } = renderHook(() => useCompile());

    let pending: Promise<void>;
    act(() => {
      pending = result.current.compile("/path/doc.tex", "content");
    });
    expect(useDocumentStore.getState().isCompiling).toBe(true);

    vi.mocked(cancelCompile).mockClear();
    act(() => {
      result.current.cancel();
    });
    expect(useDocumentStore.getState().isCompiling).toBe(false);
    expect(cancelCompile).toHaveBeenCalledTimes(1);

    await act(async () => {
      resolveLate({ pdf: new Uint8Array([1]), log: "late", success: true });
      await pending;
    });

    expect(useDocumentStore.getState().pdfData).toBeNull();
    expect(useDocumentStore.getState().isCompiling).toBe(false);
  });
});
