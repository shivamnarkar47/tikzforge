import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { detectInstallation } from "../lib/latex-engine";

describe("detectInstallation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("returns path when pdflatex is detected", async () => {
    vi.mocked(invoke).mockResolvedValue("/usr/bin/pdflatex");

    const result = await detectInstallation();

    expect(result).toEqual({ detected: true, path: "/usr/bin/pdflatex" });
    expect(invoke).toHaveBeenCalledWith("detect_engine");
  });

  it("returns detected=false when no pdflatex found", async () => {
    vi.mocked(invoke).mockResolvedValue(null);

    const result = await detectInstallation();

    expect(result).toEqual({ detected: false, path: null });
  });

  it("returns detected=false when invoke throws", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("Command not found"));

    const result = await detectInstallation();

    expect(result).toEqual({ detected: false, path: null });
  });
});
