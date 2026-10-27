import { describe, it, expect } from "vitest";
import { parseSynctex, findSourcePosition } from "../lib/synctex-parser";

describe("parseSynctex", () => {
  it("returns empty array for empty input", () => {
    expect(parseSynctex("")).toEqual([]);
  });

  it("parses synctex records", () => {
    const synctex = `Input:1:doc.tex
Output:1:doc.pdf
SyncTeX Version:1
1 100 200 5 10
1 150 250 6 20
2 100 200 3 15`;

    const records = parseSynctex(synctex);
    expect(records.length).toBeGreaterThanOrEqual(2);
    expect(records[0]).toHaveProperty("line");
    expect(records[0]).toHaveProperty("page");
  });

  it("skips comment lines", () => {
    const synctex = `SyncTeX Version:1
% comment
1 100 200 5 10`;

    const records = parseSynctex(synctex);
    expect(records.length).toBe(1);
  });
});

describe("findSourcePosition", () => {
  const records = [
    { page: 1, x: 100, y: 200, line: 10, column: 5 },
    { page: 1, x: 150, y: 250, line: 20, column: 6 },
    { page: 2, x: 100, y: 200, line: 15, column: 3 },
  ];

  it("returns closest line for a given position", () => {
    const result = findSourcePosition(records, 1, 105, 205);
    expect(result).toBe(10);
  });

  it("returns null when page has no records", () => {
    const result = findSourcePosition(records, 3, 100, 200);
    expect(result).toBeNull();
  });
});
