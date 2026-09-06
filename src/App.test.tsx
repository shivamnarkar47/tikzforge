import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import App from "./App";
import { useLatexEngineStore } from "./store/latex-engine-store";

// The first-launch hook calls detectInstallation on mount.
vi.mock("./lib/latex-engine", () => ({
  detectInstallation: vi.fn(),
}));

import { detectInstallation } from "./lib/latex-engine";

describe("App", () => {
  beforeEach(() => {
    useLatexEngineStore.setState({
      status: "ready",
      installProgress: 0,
      pdflatexPath: "/app/texlive/pdflatex",
      error: null,
    });
    vi.mocked(detectInstallation).mockResolvedValue({
      detected: true,
      path: "/app/texlive/pdflatex",
    });
  });

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

  it("shows the LaTeX gate when engine is not ready", () => {
    useLatexEngineStore.setState({ status: "error" });
    render(<App />);
    expect(screen.getByText("LaTeX not found")).toBeInTheDocument();
    expect(document.querySelector(".cm-editor")).toBeNull();
  });
});
