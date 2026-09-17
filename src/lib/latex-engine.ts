import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "./tauri";

export interface DetectionResult {
  detected: boolean;
  path: string | null;
}

export async function detectInstallation(): Promise<DetectionResult> {
  if (!isTauri()) {
    throw new Error(
      "Engine detection requires the Tauri desktop app — you are running in a browser preview. Run `bun run tauri dev` instead."
    );
  }
  try {
    const path = await invoke<string | null>("detect_engine");
    return { detected: path !== null, path };
  } catch {
    return { detected: false, path: null };
  }
}
