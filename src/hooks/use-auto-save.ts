import { useEffect } from "react";

interface AutoSaveOptions {
  onSave: () => void;
  intervalMs?: number;
}

export function useAutoSave({ onSave, intervalMs = 30000 }: AutoSaveOptions) {
  useEffect(() => {
    const id = setInterval(onSave, intervalMs);
    return () => clearInterval(id);
  }, [onSave, intervalMs]);
}
