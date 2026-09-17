import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

// Mock @tauri-apps/api/event
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

// Tests run inside Tauri in production; pretend we are in Tauri.
vi.mock("../lib/tauri", () => ({
  isTauri: () => true,
}));

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { compileDocument, cancelCompile } from "../lib/compilation";

describe("compileDocument", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("invokes compile_tex with correct args", async () => {
    vi.mocked(invoke).mockResolvedValue({
      pdf: new Uint8Array([1, 2, 3]),
      log: "Output written",
      success: true,
    });

    const result = await compileDocument("/path/to/doc.tex", "content here");

    expect(invoke).toHaveBeenCalledWith("compile_tex", {
      path: "/path/to/doc.tex",
      content: "content here",
    });
    expect(result.pdf).toBeInstanceOf(Uint8Array);
    expect(result.log).toBe("Output written");
    expect(result.success).toBe(true);
  });

  it("returns success=false when compilation fails", async () => {
    vi.mocked(invoke).mockResolvedValue({
      pdf: null,
      log: "! Undefined control sequence.",
      success: false,
    });

    const result = await compileDocument("/path/to/doc.tex", "\\badcommand");

    expect(result.success).toBe(false);
    expect(result.pdf).toBeNull();
    expect(result.log).toContain("Undefined");
  });

  it("throws when invoke fails unexpectedly", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("Tauri error"));

    await expect(compileDocument("/path/doc.tex", "content")).rejects.toThrow(
      "Tauri error"
    );
  });
});

describe("cancelCompile", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("invokes cancel_compile", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await cancelCompile();

    expect(invoke).toHaveBeenCalledWith("cancel_compile");
  });

  it("never throws when invoke fails", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("gone"));

    await expect(cancelCompile()).resolves.toBeUndefined();
  });
});

describe("compileDocument log streaming", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("forwards compile-log events to onLogLine while compiling", async () => {
    let handler!: (event: { payload: string }) => void;
    vi.mocked(listen).mockImplementation(async (_event, cb) => {
      handler = cb as (event: { payload: string }) => void;
      return () => {};
    });
    let resolveInvoke!: (v: unknown) => void;
    vi.mocked(invoke).mockReturnValue(
      new Promise((resolve) => {
        resolveInvoke = resolve;
      })
    );

    const lines: string[] = [];
    const pending = compileDocument("/path/doc.tex", "content", (line) =>
      lines.push(line)
    );
    await vi.waitFor(() => expect(listen).toHaveBeenCalled());

    handler({ payload: "note: downloading foo" });
    handler({ payload: "note: writing doc.pdf" });
    resolveInvoke({ pdf: null, log: "done", success: false });
    const result = await pending;

    expect(lines).toEqual(["note: downloading foo", "note: writing doc.pdf"]);
    expect(result.log).toBe("done");
  });

  it("unlistens when the compile settles", async () => {
    const unlisten = vi.fn();
    vi.mocked(listen).mockResolvedValue(unlisten);
    vi.mocked(invoke).mockResolvedValue({ pdf: null, log: "done", success: false });

    await compileDocument("/path/doc.tex", "content", () => {});

    expect(unlisten).toHaveBeenCalledTimes(1);
  });

  it("does not subscribe without an onLogLine callback", async () => {
    vi.mocked(invoke).mockResolvedValue({ pdf: null, log: "done", success: false });

    await compileDocument("/path/doc.tex", "content");

    expect(listen).not.toHaveBeenCalled();
  });
});
