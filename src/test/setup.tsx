import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";
import React from "react";

// Mock react-resizable-panels — doesn't work in jsdom
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
