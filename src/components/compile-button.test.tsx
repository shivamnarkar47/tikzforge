import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
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

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders compile button", () => {
    render(<CompileButton onCompile={vi.fn()} onCancel={vi.fn()} />);
    expect(screen.getByText("Compile")).toBeInTheDocument();
  });

  it("calls onCompile when clicked", () => {
    const onCompile = vi.fn();
    render(<CompileButton onCompile={onCompile} onCancel={vi.fn()} />);
    fireEvent.click(screen.getByText("Compile"));
    expect(onCompile).toHaveBeenCalledTimes(1);
  });

  it("shows spinner when compiling", () => {
    useDocumentStore.setState({ isCompiling: true });
    render(<CompileButton onCompile={vi.fn()} onCancel={vi.fn()} />);
    expect(screen.getByTestId("compile-spinner")).toBeInTheDocument();
  });

  it("calls onCancel instead of onCompile when clicked while compiling", () => {
    useDocumentStore.setState({ isCompiling: true });
    const onCompile = vi.fn();
    const onCancel = vi.fn();
    render(<CompileButton onCompile={onCompile} onCancel={onCancel} />);
    fireEvent.click(screen.getByRole("button"));
    expect(onCancel).toHaveBeenCalledTimes(1);
    expect(onCompile).not.toHaveBeenCalled();
  });

  it("shows elapsed seconds while compiling", () => {
    vi.useFakeTimers();
    useDocumentStore.setState({ isCompiling: true });
    render(<CompileButton onCompile={vi.fn()} onCancel={vi.fn()} />);
    expect(screen.getByText(/Cancel \(0s\)/)).toBeInTheDocument();
    act(() => {
      vi.advanceTimersByTime(3000);
    });
    expect(screen.getByText(/Cancel \(3s\)/)).toBeInTheDocument();
  });
});
