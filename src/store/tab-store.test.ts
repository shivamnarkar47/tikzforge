import { describe, it, expect, beforeEach } from "vitest";
import { useTabStore } from "../store/tab-store";

describe("TabStore", () => {
  beforeEach(() => {
    useTabStore.setState({
      tabs: [],
      activeTabId: null,
    });
  });

  describe("addTab", () => {
    it("adds a new tab with document state", () => {
      const tabId = useTabStore.getState().addTab("untitled.tex", "");
      const currentTabs = useTabStore.getState().tabs;
      expect(currentTabs.length).toBe(1);
      expect(currentTabs[0].id).toBe(tabId);
      expect(currentTabs[0].filename).toBe("untitled.tex");
      expect(useTabStore.getState().activeTabId).toBe(tabId);
    });

    it("does not add duplicate tab for same file", () => {
      const id1 = useTabStore.getState().addTab("doc.tex", "content");
      const id2 = useTabStore.getState().addTab("doc.tex", "different");
      expect(id1).toBe(id2);
      expect(useTabStore.getState().tabs.length).toBe(1);
    });
  });

  describe("removeTab", () => {
    it("removes tab by id", () => {
      const id = useTabStore.getState().addTab("doc.tex", "content");
      useTabStore.getState().removeTab(id);
      expect(useTabStore.getState().tabs.length).toBe(0);
      expect(useTabStore.getState().activeTabId).toBeNull();
    });

    it("switches to another tab when active tab is removed", () => {
      const id1 = useTabStore.getState().addTab("a.tex", "a");
      const id2 = useTabStore.getState().addTab("b.tex", "b");
      useTabStore.getState().setActiveTab(id1);
      useTabStore.getState().removeTab(id1);
      expect(useTabStore.getState().activeTabId).toBe(id2);
    });
  });

  describe("setActiveTab", () => {
    it("sets the active tab", () => {
      const id = useTabStore.getState().addTab("doc.tex", "");
      useTabStore.getState().setActiveTab(id);
      expect(useTabStore.getState().activeTabId).toBe(id);
    });
  });

  describe("updateTabDocument", () => {
    it("updates document content for a tab", () => {
      const id = useTabStore.getState().addTab("doc.tex", "old");
      useTabStore.getState().updateTabDocument(id, { content: "new" });
      const tab = useTabStore.getState().tabs.find((t) => t.id === id);
      expect(tab?.document.content).toBe("new");
      expect(tab?.document.isDirty).toBe(true);
    });

    it("marks tab as dirty when content changes", () => {
      const id = useTabStore.getState().addTab("doc.tex", "initial");
      useTabStore.getState().updateTabDocument(id, { content: "modified" });
      const tab = useTabStore.getState().tabs.find((t) => t.id === id);
      expect(tab?.document.isDirty).toBe(true);
    });
  });

  describe("markTabSaved", () => {
    it("clears dirty flag and updates filename", () => {
      const id = useTabStore.getState().addTab("doc.tex", "content");
      useTabStore.getState().updateTabDocument(id, { content: "changed" });
      useTabStore.getState().markTabSaved(id, "/path/doc.tex");
      const tab = useTabStore.getState().tabs.find((t) => t.id === id);
      expect(tab?.document.isDirty).toBe(false);
      expect(tab?.filename).toBe("/path/doc.tex");
    });
  });

  describe("hasUnsavedChanges", () => {
    it("returns false when no tabs have unsaved changes", () => {
      useTabStore.getState().addTab("doc.tex", "content");
      expect(useTabStore.getState().hasUnsavedChanges()).toBe(false);
    });

    it("returns true when any tab has unsaved changes", () => {
      const id = useTabStore.getState().addTab("doc.tex", "content");
      useTabStore.getState().updateTabDocument(id, { content: "changed" });
      expect(useTabStore.getState().hasUnsavedChanges()).toBe(true);
    });
  });

  describe("getActiveTabDocument", () => {
    it("returns document state of active tab", () => {
      const id = useTabStore.getState().addTab("doc.tex", "hello");
      useTabStore.getState().setActiveTab(id);
      const doc = useTabStore.getState().getActiveTabDocument();
      expect(doc?.content).toBe("hello");
      expect(doc?.filename).toBe("doc.tex");
    });

    it("returns null when no active tab", () => {
      const doc = useTabStore.getState().getActiveTabDocument();
      expect(doc).toBeNull();
    });
  });
});
