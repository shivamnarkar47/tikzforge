import { describe, it, expect, beforeEach } from "vitest";
import { useThemeStore } from "../store/theme-store";

describe("ThemeStore", () => {
  beforeEach(() => {
    useThemeStore.setState({
      theme: "system",
      systemTheme: "light",
    });
  });

  it("has default initial state", () => {
    const { theme, systemTheme } = useThemeStore.getState();
    expect(theme).toBe("system");
    expect(systemTheme).toBe("light");
  });

  describe("setTheme", () => {
    it("sets the theme to dark", () => {
      useThemeStore.getState().setTheme("dark");
      expect(useThemeStore.getState().theme).toBe("dark");
    });

    it("sets the theme to light", () => {
      useThemeStore.getState().setTheme("light");
      expect(useThemeStore.getState().theme).toBe("light");
    });

    it("sets the theme to system", () => {
      useThemeStore.getState().setTheme("system");
      expect(useThemeStore.getState().theme).toBe("system");
    });
  });

  describe("setSystemTheme", () => {
    it("updates the system theme", () => {
      useThemeStore.getState().setSystemTheme("dark");
      expect(useThemeStore.getState().systemTheme).toBe("dark");
    });
  });

  describe("toggleTheme", () => {
    it("toggles from light to dark", () => {
      useThemeStore.getState().setTheme("light");
      useThemeStore.getState().toggleTheme();
      expect(useThemeStore.getState().theme).toBe("dark");
    });

    it("toggles from dark to light", () => {
      useThemeStore.getState().setTheme("dark");
      useThemeStore.getState().toggleTheme();
      expect(useThemeStore.getState().theme).toBe("light");
    });
  });

  describe("resolvedTheme", () => {
    it("returns 'dark' when theme is dark", () => {
      useThemeStore.getState().setTheme("dark");
      expect(useThemeStore.getState().resolvedTheme()).toBe("dark");
    });

    it("returns 'light' when theme is light", () => {
      useThemeStore.getState().setTheme("light");
      expect(useThemeStore.getState().resolvedTheme()).toBe("light");
    });

    it("returns system theme when theme is system", () => {
      useThemeStore.getState().setTheme("system");
      useThemeStore.getState().setSystemTheme("dark");
      expect(useThemeStore.getState().resolvedTheme()).toBe("dark");
    });
  });
});
