import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { parseSynctex, findSourcePosition } from "../lib/synctex-parser";

export function useSynctex() {
  const [records, setRecords] = useState<ReturnType<typeof parseSynctex>>([]);
  const [synctexPath, setSynctexPath] = useState<string | null>(null);

  const loadSynctex = useCallback(async (pdfPath: string) => {
    const synctexFile = pdfPath.replace(/\.pdf$/, ".synctex.gz");
    try {
      // Tauri command to decompress and read .synctex.gz
      const content = await invoke<string>("read_synctex", { path: synctexFile });
      setRecords(parseSynctex(content));
      setSynctexPath(synctexFile);
    } catch {
      // SyncTeX file may not exist yet
      setRecords([]);
      setSynctexPath(null);
    }
  }, []);

  const getSourceLine = useCallback(
    (page: number, x: number, y: number): number | null => {
      return findSourcePosition(records, page, x, y);
    },
    [records]
  );

  return { loadSynctex, getSourceLine, synctexPath };
}
