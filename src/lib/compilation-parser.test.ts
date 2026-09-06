import { describe, it, expect } from "vitest";
import { parseCompilationLog } from "../lib/compilation-parser";

describe("parseCompilationLog", () => {
  it("returns empty array for empty log", () => {
    expect(parseCompilationLog("")).toEqual([]);
  });

  it("parses a basic error with file:line:message", () => {
    const log = "! Undefined control sequence.\n<*> \\documentclass";
    const errors = parseCompilationLog(log);
    expect(errors.length).toBeGreaterThan(0);
    expect(errors[0]).toHaveProperty("file");
    expect(errors[0]).toHaveProperty("line");
    expect(errors[0]).toHaveProperty("message");
  });

  it("parses multiple errors", () => {
    const log = `
! Undefined control sequence.
<*> \\badcommand

! Missing $ inserted.
<*> \\end{document}
    `;
    const errors = parseCompilationLog(log);
    expect(errors.length).toBeGreaterThanOrEqual(2);
  });

  it("extracts line numbers from log", () => {
    const log = "l.42 \\badcommand";
    const errors = parseCompilationLog(log);
    const withLine = errors.find((e) => e.line === 42);
    expect(withLine).toBeTruthy();
  });

  it("extracts file names from log", () => {
    const log = "(./document.tex"
    const errors = parseCompilationLog(log);
    const withFile = errors.find((e) => e.file === "./document.tex");
    expect(withFile).toBeTruthy();
  });

  it("handles malformed lines gracefully", () => {
    const log = "random text without structure";
    expect(() => parseCompilationLog(log)).not.toThrow();
  });
});
