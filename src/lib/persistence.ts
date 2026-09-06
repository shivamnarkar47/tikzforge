export function saveState<T>(key: string, state: T): void {
  try {
    localStorage.setItem(key, JSON.stringify(state));
  } catch (e) {
    console.error("Failed to save state:", e);
  }
}

export function loadState<T>(key: string): T | null {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return null;
    return JSON.parse(raw) as T;
  } catch (e) {
    console.error("Failed to load state:", e);
    return null;
  }
}
