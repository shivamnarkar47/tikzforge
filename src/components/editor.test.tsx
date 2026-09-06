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
});
