import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { Editor } from "../components/editor";

describe("Editor", () => {
  it("renders a textarea-like element", () => {
    render(<Editor value="" onChange={() => {}} />);
    const editor = document.querySelector(".cm-editor");
    expect(editor).toBeTruthy();
  });

  it("displays the initial value", () => {
    render(<Editor value="\\documentclass{article}" onChange={() => {}} />);
    const content = document.querySelector(".cm-content");
    expect(content?.textContent).toContain("documentclass{article}");
  });

  it("applies LaTeX syntax highlighting to commands", () => {
    render(<Editor value="\\documentclass{article}" onChange={() => {}} />);
    // CodeMirror wraps highlighted tokens in spans with hashed class names
    const highlightedSpans = document.querySelectorAll(".cm-content span");
    expect(highlightedSpans.length).toBeGreaterThan(0);
  });

  it("highlights comments", () => {
    render(<Editor value="% This is a comment" onChange={() => {}} />);
    const highlightedSpans = document.querySelectorAll(".cm-content span");
    expect(highlightedSpans.length).toBeGreaterThan(0);
  });
});
