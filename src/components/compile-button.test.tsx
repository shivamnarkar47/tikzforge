import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { CompileButton } from "../components/compile-button";
import { useDocumentStore } from "../store/document-store";

describe("CompileButton", () => {
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

  it("renders compile button", () => {
    const onCompile = vi.fn();
    render(<CompileButton onCompile={onCompile} />);
    expect(screen.getByText("Compile")).toBeInTheDocument();
  });

  it("calls onCompile when clicked", () => {
    const onCompile = vi.fn();
    render(<CompileButton onCompile={onCompile} />);
    fireEvent.click(screen.getByText("Compile"));
    expect(onCompile).toHaveBeenCalledTimes(1);
  });

  it("shows spinner when compiling", () => {
    useDocumentStore.setState({ isCompiling: true });
    const onCompile = vi.fn();
    render(<CompileButton onCompile={onCompile} />);
    expect(screen.getByTestId("compile-spinner")).toBeInTheDocument();
  });

  it("disables button when compiling", () => {
    useDocumentStore.setState({ isCompiling: true });
    const onCompile = vi.fn();
    render(<CompileButton onCompile={onCompile} />);
    expect(screen.getByRole("button")).toBeDisabled();
  });
});
