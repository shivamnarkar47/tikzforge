import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import App from "./App";
import { useLatexEngineStore } from "./store/latex-engine-store";
import { useThemeStore } from "./store/theme-store";

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
    useThemeStore.setState({ theme: "light", systemTheme: "light" });
    document.documentElement.classList.remove("dark");
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

  it("toggles the compilation log panel from the header", () => {
    render(<App />);
    expect(
      screen.queryByRole("region", { name: /compilation log/i })
    ).not.toBeInTheDocument();
    expect(screen.queryByText(/no compilation log yet/i)).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /^logs$/i }));

    expect(screen.getByText(/no compilation log yet/i)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /^logs$/i }));

    expect(screen.queryByText(/no compilation log yet/i)).not.toBeInTheDocument();
  });

  it("toggles the dark class on the document element", () => {
    render(<App />);
    expect(document.documentElement.classList.contains("dark")).toBe(false);

    fireEvent.click(screen.getByRole("button", { name: /toggle theme/i }));
    expect(document.documentElement.classList.contains("dark")).toBe(true);

    fireEvent.click(screen.getByRole("button", { name: /toggle theme/i }));
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });
});

describe("App P0 wiring", () => {
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
    useThemeStore.setState({ theme: "light", systemTheme: "light" });
    document.documentElement.classList.remove("dark");
  });

  it("renders auto-compile toggle", () => {
    render(<App />);
    const toggle = screen.getByRole("button", { name: /toggle auto-compile/i });
    expect(toggle).toBeInTheDocument();
  });

  it("toggles auto-compile pressed state", () => {
    render(<App />);
    const toggle = screen.getByRole("button", { name: /toggle auto-compile/i });
    expect(toggle).toHaveAttribute("aria-pressed", "true");

    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-pressed", "false");

    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-pressed", "true");
  });

  it("renders recent files selector", () => {
    render(<App />);
    const select = screen.getByLabelText("Recent files");
    expect(select).toBeInTheDocument();
  });

  it("renders update status", async () => {
    render(<App />);
    const status = await screen.findByText(/up to date/i, {}, { timeout: 3000 });
    expect(status).toBeInTheDocument();
  });

  it("renders document tabs after load", async () => {
    render(<App />);
    const closeBtn = await screen.findByRole(
      "button",
      { name: /close untitled\.tex/i },
      { timeout: 3000 }
    );
    expect(closeBtn).toBeInTheDocument();
  });

  it("handles Ctrl+S without crashing", async () => {
    render(<App />);
    await screen.findByText(/up to date/i, {}, { timeout: 3000 });
    expect(() => {
      fireEvent.keyDown(window, { key: "s", ctrlKey: true });
    }).not.toThrow();
  });
});
