import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
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
  });

  it("returns update info when available", async () => {
    vi.mocked(invoke).mockResolvedValue({
      available: true,
      version: "1.1.0",
      currentVersion: "1.0.0",
      body: "New features added",
    });

    const result = await checkForUpdates();

    expect(invoke).toHaveBeenCalledWith("check_update");
    expect(result.available).toBe(true);
    expect(result.version).toBe("1.1.0");
  });

  it("returns not available when no update", async () => {
    vi.mocked(invoke).mockResolvedValue({
      available: false,
      currentVersion: "1.0.0",
    });

    const result = await checkForUpdates();

    expect(result.available).toBe(false);
  });

  it("handles check failure gracefully", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("Network error"));

    const result = await checkForUpdates();

    expect(result.available).toBe(false);
  });
});
