import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("./tauri", () => ({
  isTauri: vi.fn(() => true),
}));

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
}));

import { isTauri } from "./tauri";
import { check } from "@tauri-apps/plugin-updater";
import { checkForUpdates, compareVersions } from "../lib/updater";

describe("compareVersions", () => {
  it("returns 0 for equal versions", () => {
    expect(compareVersions("1.0.0", "1.0.0")).toBe(0);
  });

  it("returns 1 when first is newer", () => {
    expect(compareVersions("1.1.0", "1.0.0")).toBe(1);
    expect(compareVersions("2.0.0", "1.9.9")).toBe(1);
  });

  it("returns -1 when first is older", () => {
    expect(compareVersions("1.0.0", "1.1.0")).toBe(-1);
    expect(compareVersions("1.9.9", "2.0.0")).toBe(-1);
  });

  it("handles patch versions", () => {
    expect(compareVersions("1.0.1", "1.0.0")).toBe(1);
    expect(compareVersions("1.0.0", "1.0.1")).toBe(-1);
  });
});

describe("checkForUpdates", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(isTauri).mockReturnValue(true);
  });

  it("returns update info when available", async () => {
    vi.mocked(check).mockResolvedValue({
      version: "1.1.0",
      body: "New features added",
      downloadAndInstall: vi.fn(),
    } as any);

    const result = await checkForUpdates();

    expect(check).toHaveBeenCalled();
    expect(result.available).toBe(true);
    expect(result.version).toBe("1.1.0");
  });

  it("returns not available when no update", async () => {
    vi.mocked(check).mockResolvedValue(null);

    const result = await checkForUpdates();

    expect(result.available).toBe(false);
  });

  it("handles check failure gracefully", async () => {
    vi.mocked(check).mockRejectedValue(new Error("Network error"));

    const result = await checkForUpdates();

    expect(result.available).toBe(false);
  });

  it("returns not available outside Tauri", async () => {
    vi.mocked(isTauri).mockReturnValue(false);

    const result = await checkForUpdates();

    expect(check).not.toHaveBeenCalled();
    expect(result.available).toBe(false);
  });
});