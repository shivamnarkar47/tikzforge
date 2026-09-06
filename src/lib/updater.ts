import { invoke } from "@tauri-apps/api/core";

export interface UpdateInfo {
  available: boolean;
  version?: string;
  currentVersion?: string;
  body?: string;
  downloadUrl?: string;
}

/**
 * Compare two semantic version strings.
 * Returns 1 if a > b, -1 if a < b, 0 if equal.
 */
export function compareVersions(a: string, b: string): number {
  const partsA = a.split(".").map(Number);
  const partsB = b.split(".").map(Number);

  for (let i = 0; i < Math.max(partsA.length, partsB.length); i++) {
    const numA = partsA[i] || 0;
    const numB = partsB[i] || 0;
    if (numA > numB) return 1;
    if (numA < numB) return -1;
  }

  return 0;
}

/**
 * Check for application updates via Tauri updater.
 */
export async function checkForUpdates(): Promise<UpdateInfo> {
  try {
    const result = await invoke<UpdateInfo>("check_update");
    return result;
  } catch {
    return { available: false };
  }
}
