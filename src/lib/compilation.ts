import { invoke } from "@tauri-apps/api/core";

export interface CompileResult {
  pdf: Uint8Array | null;
  log: string;
  success: boolean;
}

export async function compileDocument(
  path: string,
  content: string
): Promise<CompileResult> {
  return invoke<CompileResult>("compile_tex", { path, content });
}
