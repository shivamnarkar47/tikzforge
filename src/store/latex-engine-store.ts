import { create } from "zustand";

export type LatexEngineStatus = "unknown" | "detected" | "installing" | "ready" | "error";

export interface LatexEngineState {
  status: LatexEngineStatus;
  installProgress: number;
  pdflatexPath: string | null;
  error: string | null;
  setStatus: (status: LatexEngineStatus) => void;
  setInstallProgress: (progress: number) => void;
  setPdflatexPath: (path: string | null) => void;
  setError: (error: string | null) => void;
  isReady: () => boolean;
  isDownloading: () => boolean;
}

export const useLatexEngineStore = create<LatexEngineState>((set, get) => ({
  status: "unknown",
  installProgress: 0,
  pdflatexPath: null,
  error: null,
  setStatus: (status) => set({ status }),
  setInstallProgress: (progress) =>
    set({ installProgress: Math.max(0, Math.min(100, progress)) }),
  setPdflatexPath: (pdflatexPath) => set({ pdflatexPath }),
  setError: (error) => set({ error }),
  isReady: () => get().status === "ready",
  isDownloading: () => get().status === "installing",
}));
