import { create } from "zustand";

export interface CompilationError {
  file: string;
  line: number;
  message: string;
}

export interface RecentFile {
  path: string;
  lastOpened: number;
}

export interface DocumentState {
  filename: string;
  content: string;
  isDirty: boolean;
  pdfData: Uint8Array | null;
  isCompiling: boolean;
  compilationErrors: CompilationError[];
  compilationLog: string;
  recentFiles: RecentFile[];
  setContent: (content: string) => void;
  setFilename: (filename: string) => void;
  markSaved: () => void;
  setPdfData: (data: Uint8Array | null) => void;
  setCompiling: (isCompiling: boolean) => void;
  setCompilationErrors: (errors: CompilationError[]) => void;
  setCompilationLog: (log: string) => void;
  addRecentFile: (path: string) => void;
  removeRecentFile: (path: string) => void;
  openFile: (path: string, content: string) => void;
}

const MAX_RECENT_FILES = 10;

export const useDocumentStore = create<DocumentState>((set, get) => ({
  filename: "",
  content: "",
  isDirty: false,
  pdfData: null,
  isCompiling: false,
  compilationErrors: [],
  compilationLog: "",
  recentFiles: [],
  setContent: (content) => set({ content, isDirty: true }),
  setFilename: (filename) => set({ filename }),
  markSaved: () => set({ isDirty: false }),
  setPdfData: (pdfData) => set({ pdfData }),
  setCompiling: (isCompiling) => set({ isCompiling }),
  setCompilationErrors: (compilationErrors) => set({ compilationErrors }),
  setCompilationLog: (compilationLog) => set({ compilationLog }),
  addRecentFile: (path) => {
    const { recentFiles } = get();
    const filtered = recentFiles.filter((f) => f.path !== path);
    const updated = [{ path, lastOpened: Date.now() }, ...filtered].slice(0, MAX_RECENT_FILES);
    set({ recentFiles: updated });
  },
  removeRecentFile: (path) => {
    const { recentFiles } = get();
    set({ recentFiles: recentFiles.filter((f) => f.path !== path) });
  },
  openFile: (path, content) => {
    const { recentFiles } = get();
    const filtered = recentFiles.filter((f) => f.path !== path);
    const updated = [{ path, lastOpened: Date.now() }, ...filtered].slice(0, MAX_RECENT_FILES);
    set({
      filename: path,
      content,
      isDirty: false,
      recentFiles: updated,
    });
  },
}));
