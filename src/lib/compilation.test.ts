import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { compileDocument } from "../lib/compilation";

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
