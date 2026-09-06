import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { FirstLaunchGate } from "../components/first-launch-gate";
import { useLatexEngineStore } from "../store/latex-engine-store";

describe("FirstLaunchGate", () => {
  beforeEach(() => {
    useLatexEngineStore.setState({
      status: "unknown",
      installProgress: 0,
      pdflatexPath: null,
      error: null,
    });
  });

  it("renders children when status is ready", () => {
    useLatexEngineStore.setState({ status: "ready", pdflatexPath: "/x/pdflatex" });

    render(
      <FirstLaunchGate>
        <div data-testid="app-content">Editor & PDF</div>
      </FirstLaunchGate>
    );

    expect(screen.getByTestId("app-content")).toBeInTheDocument();
    expect(screen.queryByText("LaTeX not found")).not.toBeInTheDocument();
  });

  it("shows an error card when status is error", () => {
    useLatexEngineStore.setState({
      status: "error",
      error: "No bundled TeX Live found.",
      pdflatexPath: "/app/texlive/pdflatex",
    });

    render(
      <FirstLaunchGate>
        <div data-testid="app-content">Editor & PDF</div>
      </FirstLaunchGate>
    );

    expect(screen.getByText("LaTeX not found")).toBeInTheDocument();
    expect(screen.getByText("No bundled TeX Live found.")).toBeInTheDocument();
    expect(screen.getByText(/Expected at:/)).toBeInTheDocument();
    expect(screen.getByText("Retry")).toBeInTheDocument();
    expect(screen.queryByTestId("app-content")).not.toBeInTheDocument();
  });

  it("shows a generic error when error message is null", () => {
    useLatexEngineStore.setState({ status: "error", error: null });

    render(
      <FirstLaunchGate>
        <div data-testid="app-content">Editor & PDF</div>
      </FirstLaunchGate>
    );

    expect(screen.getByText(/could not find its bundled TeX Live/i)).toBeInTheDocument();
  });

  it("shows an error card when stuck in detected state", () => {
    useLatexEngineStore.setState({ status: "detected" });

    render(
      <FirstLaunchGate>
        <div data-testid="app-content">Editor & PDF</div>
      </FirstLaunchGate>
    );

    expect(screen.getByText("LaTeX not found")).toBeInTheDocument();
    expect(screen.queryByTestId("app-content")).not.toBeInTheDocument();
  });
});
