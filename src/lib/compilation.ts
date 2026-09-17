import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "./tauri";

export interface CompileResult {
  pdf: Uint8Array | null;
  log: string;
  success: boolean;
}

interface RawCompileResult {
  pdf: number[] | null;
  log: string;
  success: boolean;
}

export async function compileDocument(
  path: string,
  content: string
): Promise<CompileResult> {
  if (!isTauri()) {
    throw new Error(
      "Compilation requires the Tauri desktop app — you are running in a browser preview."
    );
  }
  const raw = await invoke<RawCompileResult>("compile_tex", { path, content });
  return {
    pdf: raw.pdf ? Uint8Array.from(raw.pdf) : null,
    log: raw.log,
    success: raw.success,
  };
}

/// Ask the backend to kill the running compile, if any. Safe to call with no
/// compile in flight (the backend treats it as a no-op). Never throws: in a
/// browser preview there is nothing to cancel.
export async function cancelCompile(): Promise<void> {
  if (!isTauri()) return;
  try {
    await invoke("cancel_compile");
  } catch {
    // Best-effort: cancel must never break the UI (backend gone, etc.).
  }
}
