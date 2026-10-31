import { useEffect } from "react";

interface ShortcutHandlers {
  onOpen?: () => void;
  onSave?: () => void;
  onCompile?: () => void;
}

export function useKeyboardShortcuts({ onOpen, onSave, onCompile }: ShortcutHandlers) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (!e.ctrlKey && !e.metaKey) return;

      const key = e.key.toLowerCase();
      if (key === "s") {
        e.preventDefault();
        onSave?.();
      } else if (key === "o") {
        e.preventDefault();
        onOpen?.();
      } else if (key === "enter") {
        e.preventDefault();
        onCompile?.();
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onOpen, onSave, onCompile]);
}
