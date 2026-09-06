import { useEffect, useRef, useCallback } from "react";

interface AutoCompileOptions {
  onCompile: () => void;
  enabled: boolean;
  debounceMs?: number;
}

export function useAutoCompile({ onCompile, enabled, debounceMs = 2000 }: AutoCompileOptions) {
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const trigger = useCallback(() => {
    if (!enabled) return;
    
    if (timerRef.current) {
      clearTimeout(timerRef.current);
    }

    timerRef.current = setTimeout(() => {
      onCompile();
      timerRef.current = null;
    }, debounceMs);
  }, [enabled, debounceMs, onCompile]);

  // Clear on unmount
  useEffect(() => {
    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, []);

  return { trigger };
}
