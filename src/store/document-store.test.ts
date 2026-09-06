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
      recentFiles: [],
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

  describe("recentFiles", () => {
    it("has empty recent files by default", () => {
      expect(useDocumentStore.getState().recentFiles).toEqual([]);
    });
  });

  describe("addRecentFile", () => {
    it("adds a file to recent files", () => {
      useDocumentStore.getState().addRecentFile("/path/to/doc.tex");
      expect(useDocumentStore.getState().recentFiles).toEqual([
        { path: "/path/to/doc.tex", lastOpened: expect.any(Number) },
      ]);
    });

    it("moves existing file to top when re-added", () => {
      useDocumentStore.getState().addRecentFile("/path/a.tex");
      useDocumentStore.getState().addRecentFile("/path/b.tex");
      useDocumentStore.getState().addRecentFile("/path/a.tex");
      const recent = useDocumentStore.getState().recentFiles;
      expect(recent[0].path).toBe("/path/a.tex");
      expect(recent).toHaveLength(2);
    });

    it("limits recent files to 10 items", () => {
      for (let i = 0; i < 15; i++) {
        useDocumentStore.getState().addRecentFile(`/path/file${i}.tex`);
      }
      expect(useDocumentStore.getState().recentFiles.length).toBe(10);
    });
  });

  describe("removeRecentFile", () => {
    it("removes a file from recent files", () => {
      useDocumentStore.getState().addRecentFile("/path/a.tex");
      useDocumentStore.getState().addRecentFile("/path/b.tex");
      useDocumentStore.getState().removeRecentFile("/path/a.tex");
      const recent = useDocumentStore.getState().recentFiles;
      expect(recent).toHaveLength(1);
      expect(recent[0].path).toBe("/path/b.tex");
    });
  });

  describe("openFile", () => {
    it("sets filename and content", () => {
      useDocumentStore.getState().openFile("/path/doc.tex", "content here");
      const { filename, content } = useDocumentStore.getState();
      expect(filename).toBe("/path/doc.tex");
      expect(content).toBe("content here");
    });

    it("clears dirty flag when opening a file", () => {
      useDocumentStore.getState().setContent("dirty");
      useDocumentStore.getState().openFile("/path/doc.tex", "clean");
      expect(useDocumentStore.getState().isDirty).toBe(false);
    });

    it("adds opened file to recent files", () => {
      useDocumentStore.getState().openFile("/path/doc.tex", "content");
      const recent = useDocumentStore.getState().recentFiles;
      expect(recent[0].path).toBe("/path/doc.tex");
    });
  });
});
