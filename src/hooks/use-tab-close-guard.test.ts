import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useTabCloseGuard } from "../hooks/use-tab-close-guard";
import { useTabStore } from "../store/tab-store";

describe("useTabCloseGuard", () => {
  beforeEach(() => {
    useTabStore.setState({
      tabs: [],
      activeTabId: null,
    });
  });

  it("closes tab immediately if no unsaved changes", () => {
    const id = useTabStore.getState().addTab("clean.tex", "content");
    const { result } = renderHook(() => useTabCloseGuard());

    act(() => {
      result.current.requestClose(id);
    });

    expect(useTabStore.getState().tabs.length).toBe(0);
  });

  it("marks tab as pending close if has unsaved changes", () => {
    const id = useTabStore.getState().addTab("dirty.tex", "original");
    useTabStore.getState().updateTabDocument(id, { content: "changed" });

    const { result } = renderHook(() => useTabCloseGuard());

    act(() => {
      result.current.requestClose(id);
    });

    expect(useTabStore.getState().tabs.length).toBe(1);
    expect(useTabStore.getState().pendingCloseTabId).toBe(id);
  });

  it("forces close when force=true", () => {
    const id = useTabStore.getState().addTab("dirty.tex", "original");
    useTabStore.getState().updateTabDocument(id, { content: "changed" });

    const { result } = renderHook(() => useTabCloseGuard());

    act(() => {
      result.current.forceClose(id);
    });

    expect(useTabStore.getState().tabs.length).toBe(0);
  });

  it("cancels pending close", () => {
    const id = useTabStore.getState().addTab("dirty.tex", "original");
    useTabStore.getState().updateTabDocument(id, { content: "changed" });

    const { result } = renderHook(() => useTabCloseGuard());

    act(() => {
      result.current.requestClose(id);
    });

    act(() => {
      result.current.cancelClose();
    });

    expect(useTabStore.getState().tabs.length).toBe(1);
    expect(useTabStore.getState().pendingCloseTabId).toBeNull();
  });
});
