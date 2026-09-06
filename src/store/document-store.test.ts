import { describe, it, expect, beforeEach } from "vitest";
import { useDocumentStore } from "../store/document-store";

describe("DocumentStore", () => {
  beforeEach(() => {
    useDocumentStore.setState({
      filename: "",
      content: "",
      isDirty: false,
      pdfData: null,
      isCompiling: false,
      compilationErrors: [],
      compilationLog: "",
    });
  });

  it("has default initial state", () => {
    const { filename, content, isDirty, pdfData, isCompiling } = useDocumentStore.getState();
    expect(filename).toBe("");
    expect(content).toBe("");
    expect(isDirty).toBe(false);
    expect(pdfData).toBeNull();
    expect(isCompiling).toBe(false);
  });

  describe("setContent", () => {
    it("updates content and marks document as dirty", () => {
      useDocumentStore.getState().setContent("\\documentclass{article}");
      const { content, isDirty } = useDocumentStore.getState();
      expect(content).toBe("\\documentclass{article}");
      expect(isDirty).toBe(true);
    });
  });

  describe("setFilename", () => {
    it("updates the filename", () => {
      useDocumentStore.getState().setFilename("diagram.tex");
      expect(useDocumentStore.getState().filename).toBe("diagram.tex");
    });
  });

  describe("markSaved", () => {
    it("clears the dirty flag", () => {
      useDocumentStore.getState().setContent("test");
      expect(useDocumentStore.getState().isDirty).toBe(true);
      useDocumentStore.getState().markSaved();
      expect(useDocumentStore.getState().isDirty).toBe(false);
    });
  });

  describe("setPdfData", () => {
    it("sets the PDF data", () => {
      const data = new Uint8Array([1, 2, 3]);
      useDocumentStore.getState().setPdfData(data);
      expect(useDocumentStore.getState().pdfData).toBe(data);
    });
  });

  describe("setCompiling", () => {
    it("updates compiling state", () => {
      useDocumentStore.getState().setCompiling(true);
      expect(useDocumentStore.getState().isCompiling).toBe(true);
      useDocumentStore.getState().setCompiling(false);
      expect(useDocumentStore.getState().isCompiling).toBe(false);
    });
  });

  describe("setCompilationErrors", () => {
    it("sets compilation errors", () => {
      const errors = [{ file: "test.tex", line: 10, message: "Undefined control sequence" }];
      useDocumentStore.getState().setCompilationErrors(errors);
      expect(useDocumentStore.getState().compilationErrors).toEqual(errors);
    });
  });
});
