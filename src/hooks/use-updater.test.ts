import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useUpdater } from "../hooks/use-updater";

vi.mock("../lib/updater", () => ({
  checkForUpdates: vi.fn(),
}));

import { checkForUpdates } from "../lib/updater";

describe("useUpdater", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("checks for updates on mount", async () => {
    vi.mocked(checkForUpdates).mockResolvedValue({
      available: false,
      currentVersion: "1.0.0",
    });

    const { result } = renderHook(() => useUpdater());

    await waitFor(() => {
      expect(result.current.checking).toBe(false);
    });

    expect(checkForUpdates).toHaveBeenCalled();
  });

  it("exposes update info when available", async () => {
    vi.mocked(checkForUpdates).mockResolvedValue({
      available: true,
      version: "1.1.0",
      currentVersion: "1.0.0",
      body: "Bug fixes",
    });

    const { result } = renderHook(() => useUpdater());

    await waitFor(() => {
      expect(result.current.updateInfo?.available).toBe(true);
      expect(result.current.updateInfo?.version).toBe("1.1.0");
    });
  });
});
