import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { PdfViewer } from "../components/pdf-viewer";

describe("PdfViewer", () => {
  it("renders a container element", () => {
    render(<PdfViewer pdfData={null} />);
    const container = document.querySelector(".pdf-viewer");
    expect(container).toBeTruthy();
  });

  it("shows placeholder when no PDF data", () => {
    render(<PdfViewer pdfData={null} />);
    expect(screen.getByText(/no pdf/i)).toBeTruthy();
  });
});
