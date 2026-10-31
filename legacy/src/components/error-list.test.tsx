import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ErrorList } from "../components/error-list";
import { useDocumentStore } from "../store/document-store";

describe("ErrorList", () => {
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

  it("renders nothing when no errors", () => {
    const { container } = render(<ErrorList />);
    expect(container.firstChild).toBeNull();
  });

  it("renders error messages", () => {
    useDocumentStore.setState({
      compilationErrors: [
        { file: "doc.tex", line: 10, message: "Undefined control sequence" },
        { file: "doc.tex", line: 25, message: "Missing $ inserted" },
      ],
    });

    render(<ErrorList />);

    expect(screen.getByText("Undefined control sequence")).toBeInTheDocument();
    expect(screen.getByText("Missing $ inserted")).toBeInTheDocument();
  });

  it("renders line numbers", () => {
    useDocumentStore.setState({
      compilationErrors: [{ file: "doc.tex", line: 42, message: "Error" }],
    });

    render(<ErrorList />);

    expect(screen.getByText(/42/)).toBeInTheDocument();
  });

  it("calls onSelect when error is clicked", () => {
    const onSelect = vi.fn();
    useDocumentStore.setState({
      compilationErrors: [{ file: "doc.tex", line: 10, message: "Error" }],
    });

    render(<ErrorList onSelect={onSelect} />);

    fireEvent.click(screen.getByText("Error"));
    expect(onSelect).toHaveBeenCalledWith(10);
  });
});
