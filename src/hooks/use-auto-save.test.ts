import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useAutoSave } from "../hooks/use-auto-save";

describe("useAutoSave", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("calls onSave periodically", () => {
    const onSave = vi.fn();
    renderHook(() => useAutoSave({ onSave, intervalMs: 5000 }));

    act(() => {
      vi.advanceTimersByTime(15000);
    });

    expect(onSave).toHaveBeenCalledTimes(3);
  });

  it("does not call onSave before interval", () => {
    const onSave = vi.fn();
    renderHook(() => useAutoSave({ onSave, intervalMs: 5000 }));

    act(() => {
      vi.advanceTimersByTime(4999);
    });

    expect(onSave).not.toHaveBeenCalled();
  });

  it("clears interval on unmount", () => {
    const onSave = vi.fn();
    const { unmount } = renderHook(() => useAutoSave({ onSave, intervalMs: 5000 }));

    unmount();
    act(() => {
      vi.advanceTimersByTime(10000);
    });

    expect(onSave).not.toHaveBeenCalled();
  });
});
