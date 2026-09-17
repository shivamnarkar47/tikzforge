import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { LogPanel } from "../components/log-panel";
import { useDocumentStore } from "../store/document-store";

describe("LogPanel", () => {
  beforeEach(() => {
    useDocumentStore.setState({
      filename: "",
      content: "",
      isDirty: false,
      pdfData: null,
      isCompiling: false,
      compilationErrors: [],
      compilationLog: "",
      recentFiles: [],
    });
  });

  it("shows a placeholder when there is no log yet", () => {
    render(<LogPanel />);
    expect(screen.getByText(/no compilation log yet/i)).toBeInTheDocument();
  });

  it("renders the raw compilation log", () => {
    useDocumentStore.setState({
      compilationLog: "Output written on doc.pdf (1 page).",
    });
    render(<LogPanel />);
    expect(
      screen.getByText("Output written on doc.pdf (1 page).")
    ).toBeInTheDocument();
  });

  it("exposes the log as a labelled scrollable region", () => {
    useDocumentStore.setState({ compilationLog: "some log" });
    render(<LogPanel />);
    const region = screen.getByRole("region", { name: /compilation log/i });
    expect(region).toBeInTheDocument();
    expect(region.tagName).toBe("PRE");
  });
});
