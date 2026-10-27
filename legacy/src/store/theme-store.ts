import { create } from "zustand";

export type Theme = "light" | "dark" | "system";

export interface ThemeState {
  theme: Theme;
  systemTheme: "light" | "dark";
  setTheme: (theme: Theme) => void;
  setSystemTheme: (systemTheme: "light" | "dark") => void;
  toggleTheme: () => void;
  resolvedTheme: () => "light" | "dark";
}

export const useThemeStore = create<ThemeState>((set, get) => ({
  theme: "system",
  systemTheme: "light",
  setTheme: (theme) => set({ theme }),
  setSystemTheme: (systemTheme) => set({ systemTheme }),
  toggleTheme: () => {
    const current = get().theme;
    set({ theme: current === "dark" ? "light" : "dark" });
  },
  resolvedTheme: () => {
    const { theme, systemTheme } = get();
    if (theme === "system") return systemTheme;
    return theme;
  },
}));
