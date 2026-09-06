import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import App from "./App";

describe("App", () => {
  it("renders the app shell with editor and PDF viewer", () => {
    render(<App />);
    const editor = document.querySelector(".cm-editor");
    const pdfViewer = document.querySelector(".pdf-viewer");
    expect(editor).toBeTruthy();
    expect(pdfViewer).toBeTruthy();
  });

  it("pre-loads the default template on first render", () => {
    render(<App />);
    const editorContent = document.querySelector(".cm-content");
    expect(editorContent?.textContent).toContain("documentclass");
  });

  it("sets the document title with app name", () => {
    render(<App />);
    expect(document.title).toContain("TikzForge");
  });
});
