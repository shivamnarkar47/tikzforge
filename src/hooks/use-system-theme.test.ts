import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useSystemTheme } from "../hooks/use-system-theme";

describe("useSystemTheme", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("returns 'light' when system prefers light", () => {
    vi.stubGlobal(
      "matchMedia",
      vi.fn((query: string) => ({
        matches: query === "(prefers-color-scheme: light)",
        media: query,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      }))
    );

    const { result } = renderHook(() => useSystemTheme());
    expect(result.current).toBe("light");
  });

  it("returns 'dark' when system prefers dark", () => {
    vi.stubGlobal(
      "matchMedia",
      vi.fn((query: string) => ({
        matches: query === "(prefers-color-scheme: dark)",
        media: query,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      }))
    );

    const { result } = renderHook(() => useSystemTheme());
    expect(result.current).toBe("dark");
  });

  it("updates when system theme changes", () => {
    const listeners: Array<(e: { matches: boolean }) => void> = [];
    vi.stubGlobal(
      "matchMedia",
      vi.fn((query: string) => ({
        matches: false,
        media: query,
        addEventListener: vi.fn((_event: string, cb: any) => {
          listeners.push(cb);
        }),
        removeEventListener: vi.fn(),
      }))
    );

    const { result } = renderHook(() => useSystemTheme());
    expect(result.current).toBe("light");

    // Simulate theme change to dark
    act(() => {
      listeners.forEach((cb) => cb({ matches: true }));
    });

    expect(result.current).toBe("dark");
  });
});
