import { create } from "zustand";

export interface DocumentTab {
  id: string;
  filename: string;
  document: {
    filename: string;
    content: string;
    isDirty: boolean;
    pdfData: Uint8Array | null;
    isCompiling: boolean;
    compilationErrors: { file: string; line: number; message: string }[];
    compilationLog: string;
  };
}

export interface TabState {
  tabs: DocumentTab[];
  activeTabId: string | null;
  pendingCloseTabId: string | null;
  addTab: (filename: string, content: string) => string;
  removeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  updateTabDocument: (id: string, updates: Partial<DocumentTab["document"]>) => void;
  markTabSaved: (id: string, filename: string) => void;
  hasUnsavedChanges: () => boolean;
  getActiveTabDocument: () => DocumentTab["document"] | null;
  setPendingCloseTab: (id: string | null) => void;
  closeTab: (id: string) => void;
}

function generateTabId(): string {
  return `tab-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

export const useTabStore = create<TabState>((set, get) => ({
  tabs: [],
  activeTabId: null,
  pendingCloseTabId: null,

  addTab: (filename, content) => {
    const { tabs } = get();
    const existing = tabs.find((t) => t.filename === filename);
    if (existing) {
      set({ activeTabId: existing.id });
      return existing.id;
    }

    const id = generateTabId();
    const newTab: DocumentTab = {
      id,
      filename,
      document: {
        filename,
        content,
        isDirty: false,
        pdfData: null,
        isCompiling: false,
        compilationErrors: [],
        compilationLog: "",
      },
    };

    set((state) => ({
      tabs: [...state.tabs, newTab],
      activeTabId: id,
    }));

    return id;
  },

  removeTab: (id) => {
    const { tabs, activeTabId } = get();
    const index = tabs.findIndex((t) => t.id === id);
    const newTabs = tabs.filter((t) => t.id !== id);

    let newActiveTabId = activeTabId;
    if (activeTabId === id) {
      if (newTabs.length === 0) {
        newActiveTabId = null;
      } else {
        const newIndex = Math.min(index, newTabs.length - 1);
        newActiveTabId = newTabs[newIndex].id;
      }
    }

    set({ tabs: newTabs, activeTabId: newActiveTabId });
  },

  setActiveTab: (id) => {
    set({ activeTabId: id });
  },

  updateTabDocument: (id, updates) => {
    set((state) => ({
      tabs: state.tabs.map((tab) =>
        tab.id === id
          ? {
              ...tab,
              document: {
                ...tab.document,
                ...updates,
                isDirty: updates.content !== undefined && updates.content !== tab.document.content,
              },
            }
          : tab
      ),
    }));
  },

  markTabSaved: (id, filename) => {
    set((state) => ({
      tabs: state.tabs.map((tab) =>
        tab.id === id
          ? {
              ...tab,
              filename,
              document: { ...tab.document, filename, isDirty: false },
            }
          : tab
      ),
    }));
  },

  hasUnsavedChanges: () => {
    return get().tabs.some((tab) => tab.document.isDirty);
  },

  getActiveTabDocument: () => {
    const { tabs, activeTabId } = get();
    if (!activeTabId) return null;
    const tab = tabs.find((t) => t.id === activeTabId);
    return tab?.document || null;
  },

  setPendingCloseTab: (id) => {
    set({ pendingCloseTabId: id });
  },

  closeTab: (id) => {
    get().removeTab(id);
    set({ pendingCloseTabId: null });
  },
}));
