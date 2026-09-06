import { describe, it, expect, beforeEach, vi } from "vitest";
import { saveState, loadState } from "../lib/persistence";

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => { store[key] = value; }),
    removeItem: vi.fn((key: string) => { delete store[key]; }),
    clear: vi.fn(() => { store = {}; }),
  };
})();

Object.defineProperty(window, "localStorage", {
  value: localStorageMock,
});

describe("persistence", () => {
  beforeEach(() => {
    localStorageMock.clear();
    vi.clearAllMocks();
  });

  describe("saveState", () => {
    it("saves state to localStorage", () => {
      const state = { theme: "dark", windowWidth: 1024 };
      saveState("tikzforge_state", state);
      expect(localStorageMock.setItem).toHaveBeenCalledWith(
        "tikzforge_state",
        JSON.stringify(state)
      );
    });
  });

  describe("loadState", () => {
    it("loads state from localStorage", () => {
      const state = { theme: "light", windowWidth: 800 };
      localStorageMock.setItem("tikzforge_state", JSON.stringify(state));
      const loaded = loadState("tikzforge_state");
      expect(loaded).toEqual(state);
    });

    it("returns null when no state exists", () => {
      const loaded = loadState("nonexistent_key");
      expect(loaded).toBeNull();
    });

    it("returns null when JSON is invalid", () => {
      localStorageMock.setItem("bad_key", "not valid json{{{");
      const loaded = loadState("bad_key");
      expect(loaded).toBeNull();
    });
  });
});
