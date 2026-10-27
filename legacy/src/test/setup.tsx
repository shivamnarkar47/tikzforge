import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";
import React from "react";

// Mock window.matchMedia — not available in jsdom
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn((query: string) => ({
    matches: false,
    media: query,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  })),
});

// Mock ResizeObserver — not available in jsdom (used by beUI tabs)
class ResizeObserverMock {
  observe() {}
  unobserve() {}
  disconnect() {}
}
Object.defineProperty(window, "ResizeObserver", {
  writable: true,
  value: ResizeObserverMock,
});
(globalThis as any).ResizeObserver = ResizeObserverMock;
vi.mock("react-resizable-panels", () => {
  const createMock = (defaultClassName?: string) =>
    ({ children, className, ...rest }: React.HTMLAttributes<HTMLDivElement> & { [key: string]: any }) => {
      return React.createElement("div", { className: className || defaultClassName, ...rest }, children);
    };

  return {
    Group: createMock(),
    Panel: createMock(),
    Separator: createMock(),
  };
});
