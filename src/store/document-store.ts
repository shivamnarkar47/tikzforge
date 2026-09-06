import { create } from "zustand";

export interface CompilationError {
  file: string;
  line: number;
  message: string;
}

export interface DocumentState {
  filename: string;
  content: string;
  isDirty: boolean;
  pdfData: Uint8Array | null;
  isCompiling: boolean;
  compilationErrors: CompilationError[];
  compilationLog: string;
  setContent: (content: string) => void;
  setFilename: (filename: string) => void;
  markSaved: () => void;
  setPdfData: (data: Uint8Array | null) => void;
  setCompiling: (isCompiling: boolean) => void;
  setCompilationErrors: (errors: CompilationError[]) => void;
  setCompilationLog: (log: string) => void;
}

export const useDocumentStore = create<DocumentState>((set) => ({
  filename: "",
  content: "",
  isDirty: false,
  pdfData: null,
  isCompiling: false,
  compilationErrors: [],
  compilationLog: "",
  setContent: (content) => set({ content, isDirty: true }),
  setFilename: (filename) => set({ filename }),
  markSaved: () => set({ isDirty: false }),
  setPdfData: (pdfData) => set({ pdfData }),
  setCompiling: (isCompiling) => set({ isCompiling }),
  setCompilationErrors: (compilationErrors) => set({ compilationErrors }),
  setCompilationLog: (compilationLog) => set({ compilationLog }),
}));
