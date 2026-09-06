import { invoke } from "@tauri-apps/api/core";

export interface DetectionResult {
  detected: boolean;
  path: string | null;
}

export async function detectInstallation(): Promise<DetectionResult> {
  try {
    const path = await invoke<string | null>("detect_pdflatex");
    return { detected: path !== null, path };
  } catch {
    return { detected: false, path: null };
  }
}
