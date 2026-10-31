import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useKeyboardShortcuts } from "../hooks/use-keyboard-shortcuts";

describe("useKeyboardShortcuts", () => {
  const onOpen = vi.fn();
  const onSave = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("calls onSave when Ctrl+S is pressed", () => {
    renderHook(() => useKeyboardShortcuts({ onOpen, onSave }));

    act(() => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "s", ctrlKey: true, bubbles: true })
      );
    });

    expect(onSave).toHaveBeenCalledTimes(1);
  });

  it("calls onOpen when Ctrl+O is pressed", () => {
    renderHook(() => useKeyboardShortcuts({ onOpen, onSave }));

    act(() => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "o", ctrlKey: true, bubbles: true })
      );
    });

    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("does not trigger on non-shortcut keys", () => {
    renderHook(() => useKeyboardShortcuts({ onOpen, onSave }));

    act(() => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "a", bubbles: true })
      );
    });

    expect(onOpen).not.toHaveBeenCalled();
    expect(onSave).not.toHaveBeenCalled();
  });

  it("does not trigger when ctrl is not held", () => {
    renderHook(() => useKeyboardShortcuts({ onOpen, onSave }));

    act(() => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "s", bubbles: true })
      );
    });

    expect(onSave).not.toHaveBeenCalled();
  });
});
