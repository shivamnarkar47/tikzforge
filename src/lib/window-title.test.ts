import { describe, it, expect } from "vitest";
import { formatWindowTitle } from "../lib/window-title";

describe("formatWindowTitle", () => {
  it("returns app name only when no filename", () => {
    expect(formatWindowTitle("")).toBe("TikzForge");
  });

  it("returns 'filename — TikzForge' when filename provided", () => {
    expect(formatWindowTitle("document.tex")).toBe("document.tex — TikzForge");
  });

  it("handles filenames with spaces", () => {
    expect(formatWindowTitle("my diagram.tex")).toBe("my diagram.tex — TikzForge");
  });

  it("handles filenames with paths", () => {
    expect(formatWindowTitle("/home/user/doc.tex")).toBe("/home/user/doc.tex — TikzForge");
  });
});
