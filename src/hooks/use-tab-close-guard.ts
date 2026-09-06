import { useCallback } from "react";
import { useTabStore } from "../store/tab-store";

export function useTabCloseGuard() {
  const requestClose = useCallback((id: string) => {
    const { tabs, setPendingCloseTab, removeTab } = useTabStore.getState();
    const tab = tabs.find((t) => t.id === id);
    
    if (tab?.document.isDirty) {
      setPendingCloseTab(id);
    } else {
      removeTab(id);
    }
  }, []);

  const forceClose = useCallback((id: string) => {
    const { removeTab, setPendingCloseTab } = useTabStore.getState();
    removeTab(id);
    setPendingCloseTab(null);
  }, []);

  const cancelClose = useCallback(() => {
    useTabStore.getState().setPendingCloseTab(null);
  }, []);

  return { requestClose, forceClose, cancelClose };
}
