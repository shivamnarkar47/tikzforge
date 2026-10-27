import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useAutoCompile } from "../hooks/use-auto-compile";

vi.mock("../hooks/use-compile", () => ({
  useCompile: () => ({ compile: vi.fn() }),
}));

describe("useAutoCompile", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("triggers compile after 2s debounce", () => {
    const onCompile = vi.fn();
    const { result } = renderHook(() => useAutoCompile({ onCompile, enabled: true, debounceMs: 2000 }));

    act(() => {
      result.current.trigger();
    });

    act(() => {
      vi.advanceTimersByTime(2000);
    });

    expect(onCompile).toHaveBeenCalledTimes(1);
  });

  it("resets debounce on each call", () => {
    const onCompile = vi.fn();
    const { result } = renderHook(() => useAutoCompile({ onCompile, enabled: true, debounceMs: 2000 }));

    act(() => {
      result.current.trigger();
      vi.advanceTimersByTime(1500);
      result.current.trigger();
      vi.advanceTimersByTime(1500);
    });

    expect(onCompile).not.toHaveBeenCalled();

    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(onCompile).toHaveBeenCalledTimes(1);
  });

  it("does not trigger when disabled", () => {
    const onCompile = vi.fn();
    renderHook(() => useAutoCompile({ onCompile, enabled: false, debounceMs: 2000 }));

    act(() => {
      vi.advanceTimersByTime(5000);
    });

    expect(onCompile).not.toHaveBeenCalled();
  });

  it("clears timer on unmount", () => {
    const onCompile = vi.fn();
    const { unmount } = renderHook(() => useAutoCompile({ onCompile, enabled: true, debounceMs: 2000 }));

    unmount();

    act(() => {
      vi.advanceTimersByTime(5000);
    });

    expect(onCompile).not.toHaveBeenCalled();
  });
});
