import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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
  content: string,
  onLogLine?: (line: string) => void
): Promise<CompileResult> {
  if (!isTauri()) {
    throw new Error(
      "Compilation requires the Tauri desktop app — you are running in a browser preview."
    );
  }
  // The backend emits one `compile-log` event per engine output line while
  // Tectonic runs; the full transcript still arrives with the final result.
  let unlisten: (() => void) | null = null;
  if (onLogLine) {
    unlisten = await listen<string>("compile-log", (event) =>
      onLogLine(event.payload)
    );
  }
  try {
    const raw = await invoke<RawCompileResult>("compile_tex", { path, content });
    return {
      pdf: raw.pdf ? Uint8Array.from(raw.pdf) : null,
      log: raw.log,
      success: raw.success,
    };
  } finally {
    unlisten?.();
  }
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
