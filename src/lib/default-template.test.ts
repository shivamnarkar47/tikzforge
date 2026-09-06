import { describe, it, expect } from "vitest";
import { DEFAULT_TEMPLATE } from "../lib/default-template";

describe("DEFAULT_TEMPLATE", () => {
  it("returns valid LaTeX with tikz package", () => {
    expect(DEFAULT_TEMPLATE).toContain("\\usepackage{tikz}");
  });

  it("includes a documentclass", () => {
    expect(DEFAULT_TEMPLATE).toContain("\\documentclass{article}");
  });

  it("has a tikzpicture environment", () => {
    expect(DEFAULT_TEMPLATE).toContain("\\begin{tikzpicture}");
    expect(DEFAULT_TEMPLATE).toContain("\\end{tikzpicture}");
  });

  it("is non-empty and contains actual content", () => {
    expect(DEFAULT_TEMPLATE.length).toBeGreaterThan(50);
  });
});
